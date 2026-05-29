use crate::transfer::crypto::derive_key;
use crate::transfer::pipeline;
use crate::transfer::session::SignallingClient;
use crate::ui::banner;
use crate::ui::progress::new_spinner;
use crate::util::clipboard;
use crate::util::helpers::{format_size, format_speed};
use anyhow::Result;
use colored::Colorize;

pub async fn handle_receive(
    identifier: Option<String>,
    code: Option<String>,
    clip: bool,
    relay: &str,
) -> Result<()> {
    let (effective_id, effective_code, mode) = if clip {
        if identifier.is_some() {
            anyhow::bail!("Cannot use --clip with an identifier argument");
        }
        let clip_file = clipboard::read_clipboard_file()?;
        clipboard::print_clipboard_detected(&clip_file);
        (clip_file.filename, None, "fast")
    } else {
        let id = identifier.ok_or_else(|| anyhow::anyhow!(
            "Provide an identifier (filename or code) or use --clip"
        ))?;
        let mode = if code.is_some() {
            "blast"
        } else if id.len() == 4 && id.chars().all(|c| c.is_ascii_digit()) {
            "secure"
        } else {
            "fast"
        };
        (id, code, mode)
    };

    banner::print_mode(mode);

    let spinner = new_spinner("📡 Connecting to signalling server...");
    let mut client = SignallingClient::connect(relay).await?;

    match mode {
        "blast" => {
            let blast_code = effective_code.as_deref().unwrap();
            spinner.finish_with_message("  📡 Connected to signalling server".to_string());
            println!("  💥 Joining blast session: {} with code {}", effective_id.yellow().bold(), blast_code.yellow().bold());
            println!();

            client
                .send(&crate::transfer::session::ClientMessage::Join {
                    session_id: effective_id.clone(),
                    code: Some(blast_code.to_string()),
                })
                .await?;

            let encryption_key = Some(derive_key(blast_code));
            let stats = pipeline::receive_file(&mut client, &effective_id, &effective_id, encryption_key, relay, effective_code.as_deref()).await?;
            let speed = format_speed(stats.filesize, stats.elapsed.as_secs_f64());
            print_summary("Secure-Blast".red(), &stats, &speed);
        }
        "secure" => {
            spinner.finish_with_message("  📡 Connected to signalling server".to_string());
            println!("  🎟 Joining secure session: {}\n", effective_id.yellow().bold());

            client
                .send(&crate::transfer::session::ClientMessage::Join {
                    session_id: effective_id.clone(),
                    code: None,
                })
                .await?;

            let encryption_key = Some(derive_key(&effective_id));
            let stats = pipeline::receive_file(&mut client, &effective_id, &effective_id, encryption_key, relay, Some(&effective_id)).await?;
            let speed = format_speed(stats.filesize, stats.elapsed.as_secs_f64());
            print_summary("Secure".cyan(), &stats, &speed);
        }
        _ => {
            spinner.finish_with_message("  📡 Connected to signalling server".to_string());
            println!("  🎟 Joining fast session: {}\n", effective_id.yellow().bold());

            client
                .send(&crate::transfer::session::ClientMessage::Join {
                    session_id: effective_id.clone(),
                    code: None,
                })
                .await?;

            let stats = pipeline::receive_file(&mut client, &effective_id, &effective_id, None, relay, None).await?;
            let speed = format_speed(stats.filesize, stats.elapsed.as_secs_f64());
            print_summary("Fast".green(), &stats, &speed);
        }
    }

    Ok(())
}

fn print_summary(mode_display: colored::ColoredString, stats: &pipeline::TransferStats, speed: &str) {
    println!();
    println!("  {}", "┌────────────────────────────────┐".cyan());
    println!("  {} {:^30} {}", "│".cyan(), "Download Complete".green().bold(), "│".cyan());
    println!("  {}", "├────────────────────────────────┤".cyan());
    println!("  │  Mode:        {:<18} │", mode_display);
    println!("  │  File:        {:<18} │", stats.filename);
    println!("  │  Size:        {:<18} │", format_size(stats.filesize));
    println!("  │  Time:        {:<18} │", format!("{:.1}s", stats.elapsed.as_secs_f64()));
    println!("  │  Speed:       {:<18} │", speed);
    println!("  {}", "└────────────────────────────────┘".cyan());
    println!();
}
