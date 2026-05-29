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
            clip,
            global_qr,
            relay,
        } => {
            send::handle_send(file, clip, fast, secure, secure_blast, qr, global_qr, &relay).await
        }
        Commands::Receive {
            identifier,
            code,
            clip,
            relay,
        } => {
            receive::handle_receive(identifier, code, clip, &relay).await
        }
    };

    if let Err(e) = result {
        println!();
        println!("  {} {}", "✖ Error:".red().bold(), e.to_string().red());
        println!();
        std::process::exit(1);
    }
}
