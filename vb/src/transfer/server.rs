use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub struct QrServer {
    pub port: u16,
    pub url: String,
}

pub async fn start_server(
    filepath: String,
    session_id: String,
    auth_code: Option<String>,
) -> Result<QrServer> {
    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();
    let ip = crate::ui::qr::get_local_ip().unwrap_or_else(|| "127.0.0.1".into());
    let url = format!("http://{}:{}/dl/{}", ip, port, session_id);

    let filepath = Arc::new(filepath);
    let auth_code = Arc::new(auth_code);

    tokio::spawn(async move {
        loop {
            let (mut stream, _) = match listener.accept().await {
                Ok(s) => s,
                Err(_) => break,
            };
            let filepath = filepath.clone();
            let auth_code = auth_code.clone();
            let sid = session_id.clone();

            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    return;
                }
                let request = String::from_utf8_lossy(&buf[..n]);
                let path = request
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("/")
                    .to_string();

                let expected_prefix = format!("/dl/{}", sid);

                if path.starts_with(&expected_prefix) || path == expected_prefix {
                    if let Some(ref code) = *auth_code {
                        let query_code = path.split("?code=").nth(1).unwrap_or("");
                        if query_code.is_empty() {
                            let html = html_code_page(&sid, Some(code));
                            let resp = make_text_response(200, "text/html", &html);
                            let _ = stream.write_all(resp.as_bytes()).await;
                            return;
                        }
                        if query_code != *code {
                            let resp = make_text_response(403, "text/plain", "Invalid code");
                            let _ = stream.write_all(resp.as_bytes()).await;
                            return;
                        }
                    }
                    if let Err(e) = stream_file(&mut stream, &filepath).await {
                        eprintln!("  [!] QR serve error: {}", e);
                    }
                } else {
                    let resp = make_text_response(404, "text/plain", "Not found");
                    let _ = stream.write_all(resp.as_bytes()).await;
                }
            });
        }
    });

    Ok(QrServer { port, url })
}

fn html_code_page(session_id: &str, auth_code: Option<&str>) -> String {
    let is_alpha = auth_code.map_or(false, |c| c.len() == 4 && !c.chars().all(|ch| ch.is_ascii_digit()));
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

async fn stream_file(stream: &mut tokio::net::TcpStream, filepath: &str) -> Result<()> {
    let data = tokio::fs::read(filepath).await?;
    let filename = filepath.split('\\').last().unwrap_or(filepath).split('/').last().unwrap_or(filepath);
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        data.len(), filename
    );
    stream.write_all(header.as_bytes()).await?;
    stream.write_all(&data).await?;
    Ok(())
}

fn make_text_response(status: u16, content_type: &str, body: &str) -> String {
    let reason = match status {
        200 => "OK", 403 => "Forbidden", 404 => "Not Found",
        _ => "Error",
    };
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        status, reason, content_type, body.len(), body
    )
}
