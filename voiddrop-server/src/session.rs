use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

fn percent_decode(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                out.push(byte as char);
            } else {
                out.push('%');
                out.push_str(&hex);
            }
        } else {
            out.push(c);
        }
    }
    out
}

const SESSION_TTL_SECS: i64 = 600;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub worldwide_qr: bool,
    pub upload_ready: bool,
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    file_buffers: HashMap<String, Vec<u8>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { sessions: HashMap::new(), file_buffers: HashMap::new() }
    }

    pub fn create(&mut self, mode: String, filename: String, filesize: u64, code: String, worldwide_qr: bool) -> String {
        self.cleanup_expired();
        let id = code.clone();
        let session = Session {
            id: id.clone(),
            mode,
            filename,
            filesize,
            code: id.clone(),
            sender_addr: None,
            created_at: Utc::now().timestamp(),
            worldwide_qr,
            upload_ready: false,
        };
        self.sessions.insert(id.clone(), session);
        id
    }

    pub fn join(&mut self, session_id: &str) -> Option<Session> {
        self.cleanup_expired();
        self.sessions.get(session_id).cloned()
    }
}

pub async fn handle_connection(
    stream: TcpStream,
    sessions: Arc<Mutex<SessionManager>>,
) -> Result<()> {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);

    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let trimmed = line.trim().to_string();
    if trimmed.is_empty() {
        return Ok(());
    }

    if trimmed.starts_with("GET") || trimmed.starts_with("POST") {
        handle_http(trimmed, reader, &mut writer, sessions).await?;
        return Ok(());
    }

    let msg: ClientMessage = match serde_json::from_str(&trimmed) {
        Ok(m) => m,
        Err(_) => {
            send(&mut writer, &ServerMessage::Error { message: "invalid JSON".into() }).await?;
            return Ok(());
        }
    };

    dispatch(msg, &mut reader, &mut writer, sessions).await
}

async fn handle_http(
    first_line: String,
    mut reader: BufReader<tokio::io::ReadHalf<TcpStream>>,
    writer: &mut tokio::io::WriteHalf<TcpStream>,
    sessions: Arc<Mutex<SessionManager>>,
) -> Result<()> {
    let mut content_length: usize = 0;

    loop {
        let mut hdr = String::new();
        if reader.read_line(&mut hdr).await? == 0 { break; }
        let trimmed = hdr.trim();
        if trimmed.is_empty() { break; }
        if let Some(val) = trimmed.strip_prefix("Content-Length:").or_else(|| trimmed.strip_prefix("content-length:")) {
            content_length = val.trim().parse().unwrap_or(0);
        }
    }

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let method = parts.first().unwrap_or(&"");
    let path = parts.get(1).unwrap_or(&"/");

    if *method == "POST" {
        let session_id = percent_decode(path.strip_prefix("/upload/").unwrap_or(""));
        let mut sm = sessions.lock().await;
        if !sm.upload_start(&session_id) {
            drop(sm);
            let resp = make_http_header(404, "text/plain", 26);
            writer.write_all(resp.as_bytes()).await?;
            writer.write_all(b"Session not found for upload").await?;
            return Ok(());
        }
        drop(sm);

        let mut body = vec![0u8; content_length];
        let mut read_total = 0;
        while read_total < content_length {
            let n = reader.read(&mut body[read_total..]).await?;
            if n == 0 { break; }
            read_total += n;
        }
        body.truncate(read_total);

        let mut sm = sessions.lock().await;
        sm.store_file(&session_id, &body);

        let resp = make_http_header(200, "text/plain", 2);
        writer.write_all(resp.as_bytes()).await?;
        writer.write_all(b"OK").await?;
        return Ok(());
    }

    let session_id = percent_decode(path.strip_prefix("/dl/").and_then(|s| s.split('?').next()).unwrap_or(""));
    let query_code = path.split("?code=").nth(1).unwrap_or("");

    let mut sm = sessions.lock().await;
    let session = sm.join(&session_id).filter(|s| s.upload_ready);

    match session {
        Some(sess) => {
            if sess.mode == "secure" && query_code.is_empty() {
                drop(sm);
                let html = html_code_page(&session_id, &sess.code);
                let resp = make_http_header(200, "text/html; charset=utf-8", html.len());
                writer.write_all(resp.as_bytes()).await?;
                writer.write_all(html.as_bytes()).await?;
                return Ok(());
            }
            if sess.mode == "secure" && query_code != sess.code {
                drop(sm);
                let resp = make_http_header(403, "text/plain", 11);
                writer.write_all(resp.as_bytes()).await?;
                writer.write_all(b"Invalid code").await?;
                return Ok(());
            }
            let file_data = sm.get_file(&session_id);
            drop(sm);

            if let Some(data) = file_data {
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                    data.len(), sess.filename
                );
                writer.write_all(header.as_bytes()).await?;
                writer.write_all(&data).await?;
            } else {
                let resp = make_http_header(404, "text/plain", 14);
                writer.write_all(resp.as_bytes()).await?;
                writer.write_all(b"File not found").await?;
            }
        }
        None => {
            drop(sm);
            let resp = make_http_header(404, "text/plain", 19);
            writer.write_all(resp.as_bytes()).await?;
            writer.write_all(b"Session not found").await?;
        }
    }

    Ok(())
}

fn html_code_page(session_id: &str, code: &str) -> String {
    let is_alpha = code.len() == 4 && !code.chars().all(|c| c.is_ascii_digit());
    let label = if is_alpha { "4-character" } else { "4-digit" };
    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>VoidDrop</title><style>
body{{font-family:sans-serif;background:#0a0a0f;color:#fff;display:flex;justify-content:center;align-items:center;height:100vh;margin:0}}
.card{{text-align:center;padding:2rem;border:1px solid #6c63ff;border-radius:12px;background:#12121a;max-width:320px}}
h1{{color:#6c63ff;margin:0 0 0.5rem}}p{{color:#888;margin:0 0 1.5rem;font-size:0.9rem}}
input{{padding:0.75rem;font-size:1.5rem;text-align:center;width:100px;border:2px solid #6c63ff;border-radius:8px;background:#1a1a2e;color:#fff;outline:none;letter-spacing:4px}}
input:focus{{border-color:#00d4ff}}
button{{padding:0.75rem 2rem;font-size:1rem;background:#6c63ff;color:#fff;border:none;border-radius:8px;cursor:pointer;margin-top:1rem;font-weight:bold}}
button:hover{{background:#5a52e0}}
.hint{{color:#555;font-size:0.8rem;margin-top:1rem}}
</style></head>
<body><div class="card"><h1>⚡ VoidDrop</h1><p>Enter the {} code</p>
<input type="text" id="code" maxlength="4" autofocus{}/>
<br/><button onclick="location.href='/dl/{}?code='+encodeURIComponent(code.value)">Download</button>
<div class="hint">Code is shown on the sender's terminal</div></div></body></html>"#,
        label,
        if is_alpha { r#" oninput="this.value=this.value.toUpperCase()""# } else { "" },
        session_id
    )
}

fn make_http_header(status: u16, content_type: &str, content_len: usize) -> String {
    let reason = match status {
        200 => "OK", 403 => "Forbidden", 404 => "Not Found",
        _ => "Error",
    };
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        status, reason, content_type, content_len
    )
}

async fn dispatch(
    msg: ClientMessage,
    reader: &mut BufReader<tokio::io::ReadHalf<TcpStream>>,
    writer: &mut tokio::io::WriteHalf<TcpStream>,
    sessions: Arc<Mutex<SessionManager>>,
) -> Result<()> {
    handle_msg(msg, writer, &sessions).await?;

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
                send(writer, &ServerMessage::Error { message: "invalid JSON".into() }).await?;
                continue;
            }
        };
        handle_msg(msg, writer, &sessions).await?;
    }

    Ok(())
}

async fn handle_msg(
    msg: ClientMessage,
    writer: &mut tokio::io::WriteHalf<TcpStream>,
    sessions: &Arc<Mutex<SessionManager>>,
) -> Result<()> {
    match msg {
        ClientMessage::Create { mode, filename, filesize, code, worldwide_qr } => {
            let ww = worldwide_qr.unwrap_or(false);
            let mut sm = sessions.lock().await;
            let session_id = sm.create(mode, filename, filesize, code, ww);
            send(writer, &ServerMessage::Created { session_id }).await?;
        }

        ClientMessage::Join { session_id, code } => {
            let mut sm = sessions.lock().await;
            if let Some(session) = sm.join(&session_id) {
                if session.mode == "blast" {
                    if let Some(ref c) = code {
                        if c != &session.code {
                            send(writer, &ServerMessage::Error {
                                message: format!("invalid blast code: expected {}, got {}", session.code, c),
                            }).await?;
                            return Ok(());
                        }
                    } else {
                        send(writer, &ServerMessage::Error {
                            message: "blast mode requires a code".into(),
                        }).await?;
                        return Ok(());
                    }
                }
                let sender_addr = session.sender_addr.clone();
                send(writer, &ServerMessage::Joined {
                    mode: session.mode,
                    filename: session.filename,
                    filesize: session.filesize,
                    sender_addr,
                }).await?;
            } else {
                send(writer, &ServerMessage::Error {
                    message: "session not found or expired".into(),
                }).await?;
            }
        }

        ClientMessage::P2pReady { session_id, addr } => {
            let mut sm = sessions.lock().await;
            if sm.set_sender_addr(&session_id, addr) {
                send(writer, &ServerMessage::P2pReadyAck { }).await?;
            } else {
                send(writer, &ServerMessage::Error {
                    message: "session not found".into(),
                }).await?;
            }
        }

        ClientMessage::Data { session_id, payload } => {
            if payload == "done" {
                let mut sm = sessions.lock().await;
                sm.remove(&session_id);
                send(writer, &ServerMessage::SessionClosed { reason: "transfer complete".into() }).await?;
            }
        }
    }

    Ok(())
}

impl SessionManager {
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
        self.file_buffers.remove(session_id);
    }

    pub fn upload_start(&mut self, session_id: &str) -> bool {
        if self.sessions.contains_key(session_id) {
            self.file_buffers.insert(session_id.to_string(), Vec::new());
            true
        } else {
            false
        }
    }

    pub fn store_file(&mut self, session_id: &str, data: &[u8]) {
        self.file_buffers.insert(session_id.to_string(), data.to_vec());
        if let Some(s) = self.sessions.get_mut(session_id) {
            s.upload_ready = true;
        }
    }

    pub fn get_file(&self, session_id: &str) -> Option<Vec<u8>> {
        self.file_buffers.get(session_id).cloned()
    }

    fn cleanup_expired(&mut self) {
        let now = Utc::now().timestamp();
        self.sessions.retain(|_, s| now - s.created_at < SESSION_TTL_SECS);
        self.file_buffers.retain(|id, _| self.sessions.contains_key(id));
    }
}

async fn send(writer: &mut (impl AsyncWriteExt + Unpin), msg: &ServerMessage) -> Result<()> {
    let json = serde_json::to_string(msg)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    Ok(())
}
