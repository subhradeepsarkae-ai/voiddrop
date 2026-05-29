use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::io::ReadHalf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "create")]
    Create { mode: String, filename: String, filesize: u64, code: String, worldwide_qr: Option<bool> },
    #[serde(rename = "join")]
    Join { session_id: String, code: Option<String> },
    #[serde(rename = "p2p_ready")]
    P2pReady { session_id: String, addr: String },
    #[serde(rename = "data")]
    Data { session_id: String, payload: String },
    #[serde(rename = "upload_start")]
    UploadStart { session_id: String, filesize: u64 },
    #[serde(rename = "upload_chunk")]
    UploadChunk { session_id: String, data: String, #[serde(rename = "final")] is_last: bool },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "created")]
    Created { session_id: String },
    #[serde(rename = "joined")]
    Joined { mode: String, filename: String, filesize: u64, sender_addr: Option<String> },
    #[serde(rename = "p2p_ready_ack")]
    P2pReadyAck { },
    #[serde(rename = "session_closed")]
    SessionClosed { reason: String },
    #[serde(rename = "upload_complete")]
    UploadComplete { session_id: String },
    #[serde(rename = "error")]
    Error { message: String },
}

pub struct SignallingClient {
    pub reader: BufReader<ReadHalf<TcpStream>>,
    pub writer: tokio::io::WriteHalf<TcpStream>,
}

impl SignallingClient {
    pub async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let (reader, writer) = tokio::io::split(stream);
        let reader = BufReader::new(reader);
        Ok(Self { reader, writer })
    }

    pub async fn send(&mut self, msg: &ClientMessage) -> Result<()> {
        let mut json = serde_json::to_string(msg)?;
        json.push('\n');
        self.writer.write_all(json.as_bytes()).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<ServerMessage> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await?;
        if line.is_empty() {
            bail!("connection closed");
        }
        let msg: ServerMessage = serde_json::from_str(line.trim())?;
        Ok(msg)
    }

    pub async fn recv_until<P>(&mut self, predicate: P) -> Result<ServerMessage>
    where
        P: Fn(&ServerMessage) -> bool,
    {
        use tokio::time::timeout as timeout_fn;
        use std::time::Duration;
        loop {
            let msg = timeout_fn(Duration::from_secs(60), self.recv()).await
                .map_err(|_| anyhow::anyhow!("Timed out waiting for server response (60s)"))??;
            if let ServerMessage::Error { message } = &msg {
                anyhow::bail!("Server error: {}", message);
            }
            if predicate(&msg) {
                return Ok(msg);
            }
        }
    }
}
