use colored::Colorize;
use qrcode::QrCode;
use qrcode::render::unicode;

pub fn print_qr(url: &str) {
    let code = QrCode::new(url).unwrap();
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .build();
    println!("  {}", "📱 Scan QR on mobile".cyan().bold());
    println!();
    for line in image.lines() {
        println!("  {}", line);
    }
    println!();
    println!("  {}", url.cyan().underline());
    println!();
}

pub fn get_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local = socket.local_addr().ok()?;
    Some(local.ip().to_string())
}

pub async fn get_public_ip() -> Option<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect("api.ipify.org:80").await.ok()?;
    let req = "GET / HTTP/1.1\r\nHost: api.ipify.org\r\nConnection: close\r\n\r\n";
    stream.write_all(req.as_bytes()).await.ok()?;
    let mut buf = vec![0u8; 512];
    let n = stream.read(&mut buf).await.ok()?;
    let resp = String::from_utf8_lossy(&buf[..n]);
    let ip = resp.split("\r\n\r\n").nth(1)?.trim();
    if !ip.is_empty() { Some(ip.to_string()) } else { None }
}
