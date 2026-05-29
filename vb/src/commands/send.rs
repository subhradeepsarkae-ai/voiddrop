use crate::transfer::crypto::derive_key;
use crate::transfer::pipeline;
use crate::transfer::server::start_server;
use crate::transfer::session::SignallingClient;
use crate::ui::banner;
use crate::ui::progress::new_spinner;
use crate::ui::qr::print_qr;
use crate::util::clipboard;
use crate::util::helpers::{copy_to_clipboard, format_size, format_speed, generate_blast_code, generate_code};
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

pub async fn handle_send(
    file: Option<String>,
    clip: bool,
    _fast: bool,
    secure: bool,
    secure_blast: bool,
    qr: bool,
    global_qr: bool,
    relay: &str,
) -> Result<()> {
    let (resolved_file, filesize, filename) = {
        if clip {
            let clip_file = clipboard::read_clipboard_file()?;
            clipboard::print_clipboard_detected(&clip_file);
            (clip_file.path.to_string_lossy().to_string(), clip_file.size, clip_file.filename)
        } else if let Some(f) = file {
            let path = Path::new(&f);
            let sz = tokio::fs::metadata(&f).await?.len();
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            (f, sz, name)
        } else {
            anyhow::bail!("Either provide a file path or use --clip")
        }
    };

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
    if secure_blast && global_qr {
        println!("  {} Global QR not available for blast mode\n", "⚠".yellow());
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
            worldwide_qr: if global_qr { Some(true) } else { None },
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

    if global_qr && mode != "blast" {
        let upload_spinner = new_spinner("  🌍 Uploading file to relay...");

        let stats = crate::transfer::pipeline::http_upload(relay, &session_id, &resolved_file).await?;

        upload_spinner.finish_with_message(format!(
            "  ✅ Uploaded to relay ({})",
            format_size(stats.filesize)
        ));

        let qr_url = format!("http://{}/dl/{}", relay, session_id);
        println!();
        print_qr(&qr_url);

        if mode == "secure" {
            println!("  {} Receiver scans QR, enters code on phone", "📱".to_string());
        } else {
            println!("  {} Scan QR to download from anywhere", "📱".to_string());
        }

        let speed = format_speed(stats.filesize, stats.elapsed.as_secs_f64());

        println!();
        println!("  {}", "┌────────────────────────────────┐".cyan());
        println!("  {} {:^30} {}", "│".cyan(), "Transfer Complete".green().bold(), "│".cyan());
        println!("  {}", "├────────────────────────────────┤".cyan());
        println!("  │  Mode:        {:<18} │", match mode { "secure" => "Secure QR".cyan(), _ => "Global QR".green() });
        println!("  │  File:        {:<18} │", stats.filename);
        println!("  │  Size:        {:<18} │", format_size(stats.filesize));
        println!("  │  Time:        {:<18} │", format!("{:.1}s", stats.elapsed.as_secs_f64()));
        println!("  │  Speed:       {:<18} │", speed);
        println!("  {}", "└────────────────────────────────┘".cyan());
        println!();

        return Ok(());
    }

    if !global_qr && qr && mode != "blast" {
        let auth_code = if mode == "secure" {
            Some(display_code.clone())
        } else {
            None
        };
        let server = start_server(resolved_file.clone(), session_id.clone(), auth_code).await?;
        println!("  📱 QR server on port {}\n", server.port);
        print_qr(&server.url);
    }

    let encryption_key = match mode {
        "blast" => Some(derive_key(&display_code)),
        "secure" => Some(derive_key(&session_key)),
        _ => None,
    };

    let stats = pipeline::send_file(&mut client, &session_id, &resolved_file, encryption_key, relay).await?;

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
