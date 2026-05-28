use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "vb",
    about = "Drop files to anyone, instantly — no setup, no accounts",
    version,
    long_about = concat!(
        "voiddrop — peer-to-peer file transfer over the terminal.\n",
        "Files stream directly between machines. The relay only coordinates.\n\n",
        "USAGE:\n",
        "  vb send <FILE> [FLAGS]\n",
        "  vb receive <IDENTIFIER> [CODE]\n\n",
        "3 MODES (pick one with --fast, --secure, or --secure-blast):\n",
        "  --fast         No encryption. Receiver just needs the filename.\n",
        "  --secure       AES-256-GCM encrypted. Receiver needs a 4-digit code.\n",
        "  --secure-blast AES-256-GCM encrypted. Receiver needs filename + 4-char code.\n\n",
        "QUICK START:\n",
        "  vb send photo.jpg --fast\n",
        "  vb receive photo.jpg\n\n",
        "HOW TRANSFERS WORK:\n",
        "  Step 1 — Sender connects to relay, creates a session (keyed by filename,\n",
        "           code, or both depending on mode)\n",
        "  Step 2 — Sender starts a P2P TCP listener, tells relay its address\n",
        "  Step 3 — Receiver connects to relay, joins the session\n",
        "  Step 4 — Relay gives receiver the sender's P2P address\n",
        "  Step 5 — Receiver connects directly to sender\n",
        "  Step 6 — File streams P2P with live progress bar + speed + ETA\n",
        "  Step 7 — Summary box shows filename, size, duration, speed\n\n",
        "ONE-LINER INSTALL:\n",
        "  iwr https://github.com/subhradeepsarkae-ai/voiddrop/releases/latest/download/vb.exe -o vb.exe\n\n",
        "DEFAULT RELAY:\n",
        "  zephyr.proxy.rlwy.net:12963"
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
            "━━━ MODE: --fast ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            "No encryption, instant transfer.\n",
            "Receiver only needs the filename.\n\n",
            "SENDER:\n",
            "  vb send song.mp3 --fast\n",
            "  → Connects to relay, creates session \"song.mp3\"\n",
            "  → Starts P2P listener, waits for receiver\n",
            "  → Progress bar appears once receiver connects\n\n",
            "RECEIVER:\n",
            "  vb receive song.mp3\n",
            "  → Joins session by filename\n",
            "  → Connects directly to sender\n",
            "  → File downloads to current folder\n\n",
            "━━━ MODE: --secure ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            "AES-256-GCM encrypted. Safe over any network.\n",
            "Receiver enters a 4-digit code to decrypt.\n\n",
            "SENDER:\n",
            "  vb send report.pdf --secure\n",
            "  → Generates random 4-digit code (e.g. 4829)\n",
            "  → Code auto-copied to your clipboard\n",
            "  → Creates encrypted session keyed by code\n",
            "  → SHARE the code with receiver (chat, call, etc.)\n",
            "  → Once receiver joins, encrypted P2P stream starts\n\n",
            "RECEIVER:\n",
            "  vb receive 4829\n",
            "  → Joins session by code\n",
            "  → Derives same AES-256 key from code (SHA-256)\n",
            "  → Connects and receives encrypted stream\n",
            "  → File decrypted automatically on receipt\n\n",
            "━━━ MODE: --secure-blast ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            "Encrypted + filename + alphanumeric code.\n",
            "Harder to intercept — needs BOTH filename and code.\n\n",
            "SENDER:\n",
            "  vb send plans.zip --secure-blast\n",
            "  → Generates 4-char alphanumeric code (e.g. X9P1)\n",
            "  → Code auto-copied to clipboard\n",
            "  → Session keyed by filename + code (double validation)\n",
            "  → SHARE filename AND code with receiver\n\n",
            "RECEIVER:\n",
            "  vb receive plans.zip X9P1\n",
            "  → Both filename and code required\n",
            "  → Joins session, derives key, files streams\n\n",
            "━━━ FLAG: --qr ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            "Scan QR with your phone to download over WiFi.\n",
            "Works with --fast and --secure (not --secure-blast).\n\n",
            "HOW IT WORKS:\n",
            "  Step 1 — Sender runs:  vb send file.mp4 --fast --qr\n",
            "  Step 2 — vb starts a temporary HTTP server on sender's machine\n",
            "  Step 3 — QR code renders in terminal with the download URL\n",
            "  Step 4 — Receiver scans QR with phone camera\n",
            "  Step 5 — Phone opens webpage → file downloads directly from sender\n",
            "  Step 6 — HTTP server stops automatically after transfer\n\n",
            "REQUIREMENTS:\n",
            "  Sender and phone must be on the same WiFi network (or port forwarding)\n",
            "  Phone just needs a browser — no app required\n\n",
            "━━━ FLAG: --relay ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            "Use a custom signalling server instead of the default.\n\n",
            "DEFAULT:\n",
            "  zephyr.proxy.rlwy.net:12963\n\n",
            "WHEN TO USE:\n",
            "  --relay 127.0.0.1:9876        Local testing (self-hosted relay)\n",
            "  --relay myrelay.com:9876      Your own server\n\n",
            "━━━ TRANSFER LIFE CYCLE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            "Every transfer goes through these stages:\n\n",
            "  [1/4] Connecting to relay  — establishes TCP to signalling server\n",
            "  [2/4] Creating/Joining     — session setup with mode-specific keys\n",
            "  [3/4] P2P handshake       — receiver gets sender's address, connects\n",
            "  [4/4] Streaming           — file transfers with progress bar + speed\n\n",
            "After transfer, a summary is printed:\n",
            "  ┌─────────────────────────────────┐\n",
            "  │  Transfer Complete               │\n",
            "  │  File:    report.pdf             │\n",
            "  │  Size:    2.4 MB                 │\n",
            "  │  Time:    3.2 sec                │\n",
            "  │  Speed:   750 KB/s               │\n",
            "  └─────────────────────────────────┘"
        )
    )]
    Send {
        #[arg(help = "Path to the file you want to send")]
        file: String,

        #[arg(long, help = "Fast mode — No encryption. Receiver just needs the filename. Fastest option.")]
        fast: bool,

        #[arg(long, help = "Secure mode — AES-256-GCM encrypted. Receiver needs a 4-digit code shown on your screen.")]
        secure: bool,

        #[arg(long, help = "Secure-Blast mode — AES-256-GCM encrypted. Receiver needs both filename and the alphanumeric code.")]
        secure_blast: bool,

        #[arg(long, short = 'q', help = "QR mode — Starts a temp HTTP server. Scan QR with phone to download over WiFi.")]
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
            "━━━ MODE DETECTION ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            "vb auto-detects the mode based on what you type:\n\n",
            "  1 arg, has file extension or >4 chars\n",
            "    → FAST MODE: join by filename\n",
            "    Example:  vb receive song.mp3\n\n",
            "  1 arg, exactly 4 digits (e.g. 4829)\n",
            "    → SECURE MODE: join by code, derive decryption key\n",
            "    Example:  vb receive 4829\n\n",
            "  2 args (e.g. plans.zip X9P1)\n",
            "    → BLAST MODE: join by filename + code\n",
            "    Example:  vb receive plans.zip X9P1\n\n",
            "━━━ TRANSFER PROCESS ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            "  Step 1 — Connect to relay\n",
            "  Step 2 — Join session (by filename, code, or both)\n",
            "  Step 3 — Receive sender's P2P address from relay\n",
            "  Step 4 — Connect directly to sender\n",
            "  Step 5 — File streams in with progress bar + speed\n\n",
            "━━━ FLAG: --relay ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            "  --relay 127.0.0.1:9876    Local testing\n",
            "  --relay myrelay.com:9876  Custom server\n\n",
            "DEFAULT:\n",
            "  zephyr.proxy.rlwy.net:12963\n\n",
            "━━━ AFTER TRANSFER ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
            "File is saved in your current directory with its original name.\n",
            "A summary shows: file size, transfer time, average speed.\n",
            "Progress bar shows real-time speed and ETA during transfer.\n\n",
            "TIPS:\n",
            "  • The sender will tell you which mode they're using\n",
            "  • For secure modes, the code is displayed on the sender's screen\n",
            "  • The sender's code is automatically copied to their clipboard\n",
            "  • If a port is blocked, try another mode or use --relay"
        )
    )]
    Receive {
        #[arg(help = "Filename (fast mode) or session code (secure mode) — sender tells you which")]
        identifier: String,

        #[arg(help = "Blast code (required for --secure-blast mode). Sender provides this alongside the filename")]
        code: Option<String>,

        #[arg(
            long,
            default_value = "zephyr.proxy.rlwy.net:12963",
            help = "Signalling server address"
        )]
        relay: String,
    },
}
