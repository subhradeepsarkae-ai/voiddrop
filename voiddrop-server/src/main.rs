mod session;

use anyhow::Result;
use session::SessionManager;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "9876".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("voiddrop-server listening on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    let sessions = Arc::new(Mutex::new(SessionManager::new()));

    loop {
        let (stream, peer) = listener.accept().await?;
        let sessions = sessions.clone();
        println!("  [+] connection from {}", peer);

        tokio::spawn(async move {
            if let Err(e) = session::handle_connection(stream, sessions).await {
                eprintln!("  [!] {}: {}", peer, e);
            }
            println!("  [-] {} disconnected", peer);
        });
    }
}
