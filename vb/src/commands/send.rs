use crate::transfer::crypto::derive_key;
use crate::transfer::pipeline;
use crate::transfer::server::start_server;
use crate::transfer::session::SignallingClient;
use crate::ui::banner;
use crate::ui::progress::new_spinner;
use crate::ui::qr::print_qr;
use crate::util::helpers::{copy_to_clipboard, format_size, format_speed, generate_blast_code, generate_code};
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

pub async fn handle_send(
    file: &str,
    _fast: bool,
    secure: bool,
    secure_blast: bool,
    qr: bool,
    relay: &str,
) -> Result<()> {
    let path = Path::new(file);
    let filesize = tokio::fs::metadata(file).await?.len();
    let filename = path.file_name().unwrap().to_string_lossy().to_string();

    let mode = if secure_blast {
        "blast"
    } else if secure {
        "secure"
    } else {
        "fast"
    };

    banner::print_mode(mode);

    if secure_blast && qr {
        println!("  {} QR not available for blast mode (Phase 1)\n", "⚠".yellow());
    }

    println!(
        "  📦 Preparing {} ({})",
        filename,
        format_size(filesize)
    );

    let spinner = new_spinner("📡 Connecting to signalling server...");

    let mut client = SignallingClient::connect(relay).await?;

    let (session_key, display_code) = match mode {
        "blast" => {
            let code = generate_blast_code();
            spinner.finish_with_message("  📡 Connected to signalling server".to_string());
            println!("  💥 Blast Code: {}", code.yellow().bold());
            println!("  📄 Receiver must verify: filename + code match\n");
            copy_to_clipboard(&format!("vb receive {} {}", filename, code));
            (filename.clone(), code)
        }
        "secure" => {
            let code = generate_code();
            spinner.finish_with_message("  📡 Connected to signalling server".to_string());
            println!("  🎟 Code: {}", code.yellow().bold());
            println!("  ⏳ Expires in 10 minutes\n");
            copy_to_clipboard(&code);
            (code.clone(), code)
        }
        _ => {
            spinner.finish_with_message("  📡 Connected to signalling server".to_string());
            println!("  🎟 Session: {}", filename.yellow().bold());
            println!("  📡 Receiver runs: vb receive {}\n", filename);
            (filename.clone(), filename.clone())
        }
    };

    client
        .send(&crate::transfer::session::ClientMessage::Create {
            mode: mode.to_string(),
            filename: filename.clone(),
            filesize,
            code: session_key.clone(),
        })
        .await?;

    let created = client
        .recv_until(|m| matches!(m, crate::transfer::session::ServerMessage::Created { .. }))
        .await?;
    let session_id =
        if let crate::transfer::session::ServerMessage::Created { session_id } = &created {
            session_id.clone()
        } else {
            unreachable!()
        };

    if qr && mode != "blast" {
        let auth_code = if mode == "secure" {
            Some(display_code.clone())
        } else {
            None
        };
        let server = start_server(file.to_string(), session_id.clone(), auth_code).await?;
        println!("  📱 QR server on port {}\n", server.port);
        print_qr(&server.url);
    }

    let encryption_key = match mode {
        "blast" => Some(derive_key(&display_code)),
        "secure" => Some(derive_key(&session_key)),
        _ => None,
    };

    let stats = pipeline::send_file(&mut client, &session_id, file, encryption_key).await?;

    let speed = format_speed(stats.filesize, stats.elapsed.as_secs_f64());

    println!();
    println!("  {}", "┌────────────────────────────────┐".cyan());
    println!("  {} {:^30} {}", "│".cyan(), "Transfer Complete".green().bold(), "│".cyan());
    println!("  {}", "├────────────────────────────────┤".cyan());
    println!("  │  Mode:        {:<18} │", match mode { "blast" => "Secure-Blast".red(), "secure" => "Secure".cyan(), _ => "Fast".green() });
    println!("  │  File:        {:<18} │", stats.filename);
    println!("  │  Size:        {:<18} │", format_size(stats.filesize));
    println!("  │  Time:        {:<18} │", format!("{:.1}s", stats.elapsed.as_secs_f64()));
    println!("  │  Speed:       {:<18} │", speed);
    println!("  {}", "└────────────────────────────────┘".cyan());
    println!();

    Ok(())
}
