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
    relay: &str,
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
            pb.finish_with_message("  ⚠ P2P blocked by NAT — uploading via relay...".to_string());
            return relay_upload(client, session_id, filepath, filesize, filename, relay).await;
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

pub fn http_base_from_relay(relay: &str) -> String {
    let host = relay.split(':').next().unwrap_or(relay);
    let port = relay.split(':').nth(1).unwrap_or("");
    if host.ends_with("fly.dev") || host.ends_with("fly.io") {
        format!("https://{}", host)
    } else if host == "127.0.0.1" || host == "localhost" {
        format!("http://{}:{}", host, port)
    } else {
        format!("http://{}", relay)
    }
}

pub async fn http_upload(relay: &str, session_id: &str, filepath: &str) -> Result<TransferStats> {
    let path = std::path::Path::new(filepath);
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let url = format!("{}/upload/{}", http_base_from_relay(relay), session_id);

    let file_data = tokio::fs::read(filepath).await?;
    let filesize = file_data.len() as u64;

    let start = Instant::now();
    let rclient = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    let resp = rclient.post(&url)
        .body(file_data)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Relay upload failed: {}", e))?;

    if !resp.status().is_success() {
        anyhow::bail!("Relay upload failed: HTTP {}", resp.status());
    }

    let elapsed = start.elapsed();
    Ok(TransferStats { filename, filesize, elapsed })
}

async fn relay_upload(
    _client: &mut SignallingClient,
    session_id: &str,
    filepath: &str,
    _filesize: u64,
    _filename: String,
    relay: &str,
) -> Result<TransferStats> {
    http_upload(relay, session_id, filepath).await
}

pub async fn receive_file(
    client: &mut SignallingClient,
    session_id: &str,
    _identifier: &str,
    encryption_key: Option<[u8; 32]>,
    relay: &str,
    code: Option<&str>,
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
            pb.finish_with_message("  ⚠ P2P blocked by NAT — downloading via relay...".to_string());
            return relay_download(client, session_id, &filename, filesize, relay, code).await;
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

async fn relay_download(
    client: &mut SignallingClient,
    session_id: &str,
    filename: &str,
    _filesize: u64,
    relay: &str,
    code: Option<&str>,
) -> Result<TransferStats> {
    let base_url = http_base_from_relay(relay);
    let code_param = code.map(|c| format!("?code={}", c)).unwrap_or_default();
    let url = format!("{}/dl/{}{}", base_url, session_id, code_param);
    let outpath = format!("received_{}", filename);

    let rclient = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    let start = Instant::now();

    loop {
        if start.elapsed() > Duration::from_secs(120) {
            return Err(anyhow::anyhow!("Relay download timed out"));
        }

        match rclient.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let bytes = resp.bytes().await?;
                    let total = bytes.len() as u64;
                    tokio::fs::write(&outpath, &bytes).await?;

                    let elapsed = start.elapsed();
                    client
                        .send(&super::session::ClientMessage::Data {
                            session_id: session_id.to_string(),
                            payload: "done".into(),
                        })
                        .await?;

                    return Ok(TransferStats { filename: outpath, filesize: total, elapsed });
                } else if resp.status().as_u16() == 404 {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                } else {
                    anyhow::bail!("Relay download failed: HTTP {}", resp.status());
                }
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        }
    }
}
