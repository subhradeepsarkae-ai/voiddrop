use crate::transfer::crypto::derive_key;
use crate::transfer::pipeline;
use crate::transfer::session::SignallingClient;
use crate::ui::banner;
use crate::ui::progress::new_spinner;
use crate::util::helpers::{format_size, format_speed};
use anyhow::Result;
use colored::Colorize;

pub async fn handle_receive(identifier: &str, code: Option<&str>, relay: &str) -> Result<()> {
    let mode = if code.is_some() {
        "blast"
    } else if identifier.len() == 4 && identifier.chars().all(|c| c.is_ascii_digit()) {
        "secure"
    } else {
        "fast"
    };

    banner::print_mode(mode);

    let spinner = new_spinner("📡 Connecting to signalling server...");
    let mut client = SignallingClient::connect(relay).await?;

    match mode {
        "blast" => {
            let filename = identifier;
            let blast_code = code.unwrap();
            spinner.finish_with_message("  📡 Connected to signalling server".to_string());
            println!("  💥 Joining blast session: {} with code {}", filename.yellow().bold(), blast_code.yellow().bold());
            println!();

            client
                .send(&crate::transfer::session::ClientMessage::Join {
                    session_id: filename.to_string(),
                    code: Some(blast_code.to_string()),
                })
                .await?;

            let encryption_key = Some(derive_key(blast_code));
            let stats = pipeline::receive_file(&mut client, filename, filename, encryption_key).await?;
            let speed = format_speed(stats.filesize, stats.elapsed.as_secs_f64());
            print_summary("Secure-Blast".red(), &stats, &speed);
        }
        "secure" => {
            spinner.finish_with_message("  📡 Connected to signalling server".to_string());
            println!("  🎟 Joining secure session: {}\n", identifier.yellow().bold());

            client
                .send(&crate::transfer::session::ClientMessage::Join {
                    session_id: identifier.to_string(),
                    code: None,
                })
                .await?;

            let encryption_key = Some(derive_key(identifier));
            let stats = pipeline::receive_file(&mut client, identifier, identifier, encryption_key).await?;
            let speed = format_speed(stats.filesize, stats.elapsed.as_secs_f64());
            print_summary("Secure".cyan(), &stats, &speed);
        }
        _ => {
            spinner.finish_with_message("  📡 Connected to signalling server".to_string());
            println!("  🎟 Joining fast session: {}\n", identifier.yellow().bold());

            client
                .send(&crate::transfer::session::ClientMessage::Join {
                    session_id: identifier.to_string(),
                    code: None,
                })
                .await?;

            let stats = pipeline::receive_file(&mut client, identifier, identifier, None).await?;
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
