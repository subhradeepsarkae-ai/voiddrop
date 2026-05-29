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
         "  vb send [<FILE> | --clip] [--fast | --secure | --secure-blast] [--qr | --global-qr] [--relay ADDR]\n",
         "  vb receive [<IDENTIFIER> | --clip] [CODE] [--relay ADDR]\n\n",
        "━━━ MODE: --fast ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        "No encryption, instant transfer. Receiver just needs the filename.\n\n",
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
        "━━━ MODE: --secure ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
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
        "━━━ MODE: --secure-blast ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
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
        "━━━ FLAG: --qr ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
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
        "  Sender and phone must be on the same WiFi network.\n",
        "  Phone just needs a browser — no app required.\n\n",
        "━━━ FLAG: --relay ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        "Use a custom signalling server instead of the default.\n\n",
          "  --relay 127.0.0.1:9876        Local testing (self-hosted relay)\n",
         "  --relay myrelay.com:9876      Your own server\n\n",
          "DEFAULT: voiddrop.fly.dev:9876\n\n",
         "━━━ FLAG: --clip ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
         "Send files directly from clipboard — no typing needed.\n\n",
         "SENDER:\n",
         "  vb send --clip --fast\n",
         "  → Reads file path from clipboard (copied file or path text)\n",
         "  → Shows detected file name and size\n",
         "  → Starts transfer in chosen mode\n",
         "  → Works with --fast, --secure, and --qr\n\n",
         "RECEIVER:\n",
         "  vb receive --clip\n",
         "  → Reads filename from clipboard\n",
         "  → Joins fast-mode session by filename\n\n",
         "HOW TO COPY A FILE:\n",
         "  Windows: Ctrl+C the file in File Explorer\n",
         "  macOS:   Cmd+C the file in Finder\n",
         "  Linux:   Ctrl+C the file in file manager\n",
         "  Or copy a file path as text from anywhere\n\n",
         "━━━ FLAG: --global-qr ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
         "QR that works from anywhere — phone doesn't need to be on same WiFi.\n\n",
         "  File is uploaded through the relay server (in memory only, no disk).\n",
         "  Phone scans QR and downloads directly from the relay.\n\n",
         "  vb send photo.jpg --fast --global-qr\n",
         "  vb send --clip --secure --global-qr\n\n",
         "  Supports --fast and --secure (not --secure-blast).\n",
         "  When --global-qr is set, --qr is ignored.\n\n",
         "━━━ RECEIVE MODE DETECTION ─────────────────────────────────────\n",
        "vb auto-detects the mode based on what you type:\n\n",
        "  1 arg, has file extension or >4 chars\n",
        "    → FAST MODE:  vb receive song.mp3\n\n",
        "  1 arg, exactly 4 digits (e.g. 4829)\n",
        "    → SECURE MODE:  vb receive 4829\n\n",
        "  2 args (e.g. plans.zip X9P1)\n",
        "    → BLAST MODE:  vb receive plans.zip X9P1\n\n",
        "━━━ TRANSFER LIFE CYCLE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n",
        "  [1/4] Connecting to relay  — establishes TCP to signalling server\n",
        "  [2/4] Creating / Joining    — session setup with mode-specific keys\n",
        "  [3/4] P2P handshake        — receiver gets sender's address, connects\n",
        "  [4/4] Streaming            — file transfers with progress bar + speed\n\n",
        "After transfer, a summary is printed:\n",
        "  ┌─────────────────────────────────┐\n",
        "  │  Transfer Complete               │\n",
        "  │  File:    report.pdf             │\n",
        "  │  Size:    2.4 MB                 │\n",
        "  │  Time:    3.2 sec                │\n",
        "  │  Speed:   750 KB/s               │\n",
        "  └─────────────────────────────────┘\n\n",
        "ONE-LINER INSTALL:\n",
        "  iwr https://github.com/subhradeepsarkae-ai/voiddrop/releases/latest/download/vb.exe -o vb.exe"
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
            "  vb send photo.jpg --fast                  no encryption, just filename\n",
            "  vb send docs.zip --secure                 AES-256 encrypted, 4-digit code\n",
            "  vb send secret.zip --secure-blast          encrypted, filename + alphanumeric code\n",
            "  vb send video.mp4 --fast --qr              QR for phone download over WiFi\n",
            "  vb send --clip --fast                     send file from clipboard (fast)\n",
            "  vb send --clip --secure                   send file from clipboard (encrypted)\n",
            "  vb send photo.jpg --fast --global-qr      QR via relay (anywhere)\n",
            "  vb send --clip --fast --global-qr          clipboard + global QR\n\n",
            "For full details on every mode and flag, run:  vb --help"
        )
    )]
    Send {
        #[arg(help = "Path to the file you want to send (omit when using --clip)")]
        file: Option<String>,

        #[arg(long, help = "Fast mode — No encryption. Receiver just needs the filename.")]
        fast: bool,

        #[arg(long, help = "Secure mode — AES-256-GCM encrypted. Receiver needs a 4-digit code.")]
        secure: bool,

        #[arg(long, help = "Secure-Blast mode — AES-256-GCM encrypted. Receiver needs filename + code.")]
        secure_blast: bool,

        #[arg(long, short = 'q', help = "QR mode — Starts a temp HTTP server. Scan QR with phone to download over WiFi.")]
        qr: bool,

        #[arg(
            long,
            short = 'c',
            help = "Read file path from clipboard instead of typing it"
        )]
        clip: bool,

        #[arg(
            long,
            help = "Global QR — Relay serves the file. Phone downloads from anywhere (not just LAN)"
        )]
        global_qr: bool,

        #[arg(
            long,
            default_value = "voiddrop.fly.dev:9876",
            help = "Signalling server address"
        )]
        relay: String,
    },

    /// Receive a file from another machine
    #[command(
        after_help = concat!(
            "MODE DETECTION (auto-detected):\n",
            "  1 arg, has extension or >4 chars  → FAST mode:  vb receive photo.jpg\n",
            "  1 arg, exactly 4 digits            → SECURE mode:  vb receive 4829\n",
            "  2 args                             → BLAST mode:  vb receive plans.zip X9P1\n",
            "  --clip (no arg)                    → FAST mode:  vb receive --clip\n\n",
            "For full details on modes and flags, run:  vb --help"
        )
    )]
    Receive {
        #[arg(help = "Filename (fast mode) or session code (secure mode) — omit when using --clip")]
        identifier: Option<String>,

        #[arg(help = "Blast code (required for --secure-blast mode). Sender provides this alongside the filename")]
        code: Option<String>,

        #[arg(
            long,
            short = 'c',
            help = "Read filename from clipboard instead of typing it"
        )]
        clip: bool,

        #[arg(
            long,
            default_value = "voiddrop.fly.dev:9876",
            help = "Signalling server address"
        )]
        relay: String,
    },
}
