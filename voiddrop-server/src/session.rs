use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

const SESSION_TTL_SECS: i64 = 600;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "create")]
    Create { mode: String, filename: String, filesize: u64, code: String },
    #[serde(rename = "join")]
    Join { session_id: String, code: Option<String> },
    #[serde(rename = "p2p_ready")]
    P2pReady { session_id: String, addr: String },
    #[serde(rename = "data")]
    Data { session_id: String, payload: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub mode: String,
    pub filename: String,
    pub filesize: u64,
    pub code: String,
    pub sender_addr: Option<String>,
    pub created_at: i64,
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { sessions: HashMap::new() }
    }

    pub fn create(&mut self, mode: String, filename: String, filesize: u64, code: String) -> String {
        self.cleanup_expired();
        let id = code;
        let session = Session {
            id: id.clone(),
            mode,
            filename,
            filesize,
            code: id.clone(),
            sender_addr: None,
            created_at: Utc::now().timestamp(),
        };
        self.sessions.insert(id.clone(), session);
        id
    }

    pub fn join(&mut self, session_id: &str) -> Option<Session> {
        self.cleanup_expired();
        self.sessions.get(session_id).cloned()
    }

    pub fn set_sender_addr(&mut self, session_id: &str, addr: String) -> bool {
        if let Some(s) = self.sessions.get_mut(session_id) {
            s.sender_addr = Some(addr);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    fn cleanup_expired(&mut self) {
        let now = Utc::now().timestamp();
        self.sessions.retain(|_, s| now - s.created_at < SESSION_TTL_SECS);
    }
}

pub async fn handle_connection(
    mut stream: TcpStream,
    sessions: Arc<Mutex<SessionManager>>,
) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let msg: ClientMessage = match serde_json::from_str(line.trim()) {
            Ok(m) => m,
            Err(_) => {
                send(&mut writer, &ServerMessage::Error { message: "invalid JSON".into() }).await?;
                continue;
            }
        };

        match msg {
            ClientMessage::Create { mode, filename, filesize, code } => {
                let mut sm = sessions.lock().await;
                let session_id = sm.create(mode, filename, filesize, code);
                send(&mut writer, &ServerMessage::Created { session_id }).await?;
            }

            ClientMessage::Join { session_id, code } => {
                let mut sm = sessions.lock().await;
                if let Some(session) = sm.join(&session_id) {
                    if session.mode == "blast" {
                        if let Some(ref c) = code {
                            if c != &session.code {
                                send(&mut writer, &ServerMessage::Error {
                                    message: format!("invalid blast code: expected {}, got {}", session.code, c),
                                }).await?;
                                continue;
                            }
                        } else {
                            send(&mut writer, &ServerMessage::Error {
                                message: "blast mode requires a code".into(),
                            }).await?;
                            continue;
                        }
                    }
                    let sender_addr = session.sender_addr.clone();
                    send(&mut writer, &ServerMessage::Joined {
                        mode: session.mode,
                        filename: session.filename,
                        filesize: session.filesize,
                        sender_addr,
                    }).await?;
                } else {
                    send(&mut writer, &ServerMessage::Error {
                        message: "session not found or expired".into(),
                    }).await?;
                }
            }

            ClientMessage::P2pReady { session_id, addr } => {
                let mut sm = sessions.lock().await;
                if sm.set_sender_addr(&session_id, addr) {
                    send(&mut writer, &ServerMessage::P2pReadyAck { }).await?;
                } else {
                    send(&mut writer, &ServerMessage::Error {
                        message: "session not found".into(),
                    }).await?;
                }
            }

            ClientMessage::Data { session_id, payload } => {
                if payload == "done" {
                    let mut sm = sessions.lock().await;
                    sm.remove(&session_id);
                    send(&mut writer, &ServerMessage::SessionClosed { reason: "transfer complete".into() }).await?;
                }
            }
        }
    }

    Ok(())
}

async fn send(writer: &mut (impl AsyncWriteExt + Unpin), msg: &ServerMessage) -> Result<()> {
    let json = serde_json::to_string(msg)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    Ok(())
}
