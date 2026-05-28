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
