mod cli;
mod commands;
mod transfer;
mod ui;
mod util;

use clap::Parser;
use cli::args::{Cli, Commands};
use colored::Colorize;
use commands::{receive, send};
use ui::banner;

#[tokio::main]
async fn main() {
    banner::print_banner();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Send {
            file,
            fast,
            secure,
            secure_blast,
            qr,
            relay,
        } => {
            send::handle_send(&file, fast, secure, secure_blast, qr, &relay).await
        }
        Commands::Receive { identifier, code, relay } => {
            receive::handle_receive(&identifier, code.as_deref(), &relay).await
        }
    };

    if let Err(e) = result {
        println!();
        println!("  {} {}", "✖ Error:".red().bold(), e.to_string().red());
        println!();
        std::process::exit(1);
    }
}
