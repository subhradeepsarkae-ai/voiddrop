use crate::transfer::session::SignallingClient;
use crate::ui::progress::new_progress_bar;
use crate::ui::qr::get_public_ip;
use crate::util::helpers::format_size;
use anyhow::Result;
use std::path::Path;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::Duration;

use super::crypto::{decrypt_chunk, encrypt_chunk};
use super::session::ServerMessage;

const CHUNK_SIZE: usize = 64 * 1024;

pub struct TransferStats {
    pub filename: String,
    pub filesize: u64,
    pub elapsed: Duration,
}

pub async fn send_file(
    client: &mut SignallingClient,
    session_id: &str,
    filepath: &str,
    encryption_key: Option<[u8; 32]>,
) -> Result<TransferStats> {
    let path = Path::new(filepath);
    let filesize = tokio::fs::metadata(filepath).await?.len();
    let filename = path.file_name().unwrap().to_string_lossy().to_string();

    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let p2p_port = listener.local_addr()?.port();
    let public_ip = get_public_ip().await.unwrap_or_else(|| "127.0.0.1".into());
    let external_addr = format!("{}:{}", public_ip, p2p_port);

    client
        .send(&super::session::ClientMessage::P2pReady {
            session_id: session_id.to_string(),
            addr: external_addr.clone(),
        })
        .await?;

    client
        .recv_until(|m| matches!(m, ServerMessage::P2pReadyAck { .. }))
        .await?;

    let pb = new_progress_bar(filesize, "  📡 Waiting for receiver to connect...");

    let accept = tokio::time::timeout(Duration::from_secs(60), listener.accept()).await;
    let mut stream = match accept {
        Ok(Ok((stream, peer))) => {
            pb.set_message(format!("  🤝 Connected to {}", peer));
            stream
        }
        _ => {
            pb.finish_with_message("  ❌ Receiver did not connect".to_string());
            return Err(anyhow::anyhow!("Receiver connection timed out"));
        }
    };

    let mut file = tokio::fs::File::open(filepath).await?;
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut total: u64 = 0;
    let start = Instant::now();

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        if let Some(ref key) = encryption_key {
            let frame = encrypt_chunk(key, &buf[..n])?;
            let len = (frame.len() as u32).to_le_bytes();
            stream.write_all(&len).await?;
            stream.write_all(&frame).await?;
        } else {
            stream.write_all(&buf[..n]).await?;
        }
        total += n as u64;
        pb.set_position(total);
    }

    let elapsed = start.elapsed();
    pb.finish_with_message(format!(
        "  ✅ Sent {} ({}) in {:.1}s",
        filename,
        format_size(filesize),
        elapsed.as_secs_f64()
    ));

    client
        .send(&super::session::ClientMessage::Data {
            session_id: session_id.to_string(),
            payload: "done".into(),
        })
        .await?;

    Ok(TransferStats { filename, filesize, elapsed })
}

pub async fn receive_file(
    client: &mut SignallingClient,
    _session_id: &str,
    _identifier: &str,
    encryption_key: Option<[u8; 32]>,
) -> Result<TransferStats> {
    let joined = client
        .recv_until(|m| matches!(m, ServerMessage::Joined { .. }))
        .await?;

    let (mode, filename, filesize, sender_addr) =
        if let ServerMessage::Joined { mode, filename, filesize, sender_addr } = &joined {
            (mode.clone(), filename.clone(), *filesize, sender_addr.clone())
        } else {
            unreachable!()
        };

    let is_secure = mode == "secure" || mode == "blast";

    let pb = new_progress_bar(filesize, &format!("  ⚡ Receiving {}...", filename));
    let sender_addr = sender_addr.ok_or_else(|| anyhow::anyhow!("Sender not ready yet"))?;

    pb.set_message(format!("  🔗 Connecting to sender at {}...", sender_addr));

    let conn = tokio::time::timeout(Duration::from_secs(15), TcpStream::connect(&sender_addr)).await;
    let mut stream = match conn {
        Ok(Ok(stream)) => {
            pb.set_message("  🔗 P2P connected".to_string());
            stream
        }
        _ => {
            pb.finish_with_message("  ❌ Could not connect to sender".to_string());
            return Err(anyhow::anyhow!("Could not connect to sender"));
        }
    };

    let outpath = format!("received_{}", filename);
    let mut file = tokio::fs::File::create(&outpath).await?;
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut total: u64 = 0;
    let start = Instant::now();

    if is_secure {
        let key = encryption_key
            .ok_or_else(|| anyhow::anyhow!("encryption key required for secure mode"))?;
        loop {
            let mut len_buf = [0u8; 4];
            if stream.read_exact(&mut len_buf).await.is_err() {
                break;
            }
            let frame_len = u32::from_le_bytes(len_buf) as usize;
            let mut frame = vec![0u8; frame_len];
            stream.read_exact(&mut frame).await?;
            let plaintext = decrypt_chunk(&key, &frame)?;
            file.write_all(&plaintext).await?;
            total += plaintext.len() as u64;
            pb.set_position(total);
        }
    } else {
        loop {
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n]).await?;
            total += n as u64;
            pb.set_position(total);
        }
    }

    let elapsed = start.elapsed();
    pb.finish_with_message(format!(
        "  ✅ Received {} ({}) in {:.1}s",
        outpath,
        format_size(filesize),
        elapsed.as_secs_f64()
    ));

    Ok(TransferStats { filename: outpath, filesize, elapsed })
}
