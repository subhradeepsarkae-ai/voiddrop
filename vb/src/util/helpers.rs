use rand::Rng;

pub fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen_range(1000..=9999);
    num.to_string()
}

pub fn generate_blast_code() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    (0..4).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

pub fn format_speed(bytes: u64, elapsed_secs: f64) -> String {
    if elapsed_secs <= 0.0 {
        return "∞".to_string();
    }
    let bps = bytes as f64 / elapsed_secs;
    format_size(bps as u64) + "/s"
}

pub fn copy_to_clipboard(text: &str) {
    match arboard::Clipboard::new() {
        Ok(mut ctx) => {
            if ctx.set_text(text).is_ok() {
                println!("  {} Code copied to clipboard", "📋".to_string());
            }
        }
        Err(_) => {}
    }
}
