use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vb", about = "Secure Terminal Transfer", version, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Send a file
    Send {
        #[arg(help = "File to send")]
        file: String,

        #[arg(long, help = "Fast mode — instant P2P transfer, no encryption")]
        fast: bool,

        #[arg(long, help = "Secure mode — AES-256-GCM encrypted with 4-digit code")]
        secure: bool,

        #[arg(long, help = "Secure-Blast mode — encrypted + filename + alphanumeric code")]
        secure_blast: bool,

        #[arg(long, help = "Generate QR code for mobile download")]
        qr: bool,

        #[arg(long, default_value = "relay.opendev.website:9876", help = "Signalling server address")]
        relay: String,
    },

    /// Receive a file
    Receive {
        #[arg(help = "Filename (fast mode) or session code (secure mode)")]
        identifier: String,

        #[arg(help = "Blast code (required for --secure-blast mode)")]
        code: Option<String>,

        #[arg(long, default_value = "relay.opendev.website:9876", help = "Signalling server address")]
        relay: String,
    },
}
