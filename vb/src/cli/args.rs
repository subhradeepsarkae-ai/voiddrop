use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "vb",
    about = "Drop files to anyone, instantly — no setup, no accounts",
    version,
    long_about = concat!(
        "voiddrop — peer-to-peer file transfer over the terminal.\n\n",
        "Three modes:\n",
        "  --fast         Direct P2P transfer, no encryption. Just filename and go.\n",
        "  --secure       AES-256-GCM encrypted. Receiver needs a 4-digit code.\n",
        "  --secure-blast AES-256-GCM encrypted + filename + alphanumeric code.\n\n",
        "Quick start:\n",
        "  vb send photo.jpg --fast\n",
        "  vb receive photo.jpg\n\n",
        "All transfers use a signalling server at zephyr.proxy.rlwy.net:12963\n",
        "to coordinate the connection, then send data directly P2P.\n",
        "Files never touch the server."
    )
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Send a file to another machine
    #[command(
        after_help = concat!(
            "EXAMPLES:\n",
            "  vb send video.mp4 --fast                    basic fast transfer\n",
            "  vb send docs.zip --secure                   encrypted with 4-digit code\n",
            "  vb send secret.pdf --secure-blast            encrypted + filename + code\n",
            "  vb send photo.jpg --fast --qr               QR code for phone download\n",
            "  vb send file.bin --fast --relay 1.2.3.4:9876 custom relay address\n\n",
            "MODES:\n",
            "  --fast           No encryption, fastest option.\n",
            "                   Receiver just needs the filename.\n\n",
            "  --secure         AES-256-GCM encrypted.\n",
            "                   Receiver needs the 4-digit code shown on your screen.\n",
            "                   Code is auto-copied to clipboard.\n\n",
            "  --secure-blast   AES-256-GCM encrypted with a 4-char alphanumeric code.\n",
            "                   Both filename and code are required to receive.\n",
            "                   Safer against filename guessing.\n\n",
            "QR (--qr):\n",
            "  Starts a temporary HTTP server on your machine.\n",
            "  Scan the QR with your phone → download directly over WiFi.\n",
            "  Works with --fast and --secure modes.\n\n",
            "RELAY:\n",
            "  Default: zephyr.proxy.rlwy.net:12963\n",
            "  Only change this if you're running your own signalling server."
        )
    )]
    Send {
        #[arg(help = "Path to the file you want to send")]
        file: String,

        #[arg(long, help = "Fast mode — instant P2P transfer, no encryption")]
        fast: bool,

        #[arg(long, help = "Secure mode — AES-256-GCM encrypted with 4-digit code")]
        secure: bool,

        #[arg(long, help = "Secure-Blast mode — encrypted + filename + alphanumeric code")]
        secure_blast: bool,

        #[arg(long, short = 'q', help = "Generate QR code for mobile download over WiFi")]
        qr: bool,

        #[arg(
            long,
            default_value = "zephyr.proxy.rlwy.net:12963",
            help = "Signalling server address"
        )]
        relay: String,
    },

    /// Receive a file from another machine
    #[command(
        after_help = concat!(
            "EXAMPLES:\n",
            "  vb receive photo.jpg                        fast mode (just filename)\n",
            "  vb receive 4829                             secure mode (4-digit code)\n",
            "  vb receive plans.zip X9P1                   secure-blast mode (filename + code)\n",
            "  vb receive photo.jpg --relay 1.2.3.4:9876   custom relay address\n\n",
            "MODE DETECTION:\n",
            "  1 argument, not 4 digits  → Fast mode (filename)\n",
            "  1 argument, 4 digits       → Secure mode (code)\n",
            "  2 arguments                → Secure-Blast mode (filename + code)\n\n",
            "HINT:\n",
            "  The sender will tell you which mode and what to type.\n",
            "  Codes are shown on the sender's screen and auto-copied."
        )
    )]
    Receive {
        #[arg(help = "Filename (fast mode) or session code (secure mode)")]
        identifier: String,

        #[arg(help = "Blast code (required for --secure-blast mode)")]
        code: Option<String>,

        #[arg(
            long,
            default_value = "zephyr.proxy.rlwy.net:12963",
            help = "Signalling server address"
        )]
        relay: String,
    },
}
