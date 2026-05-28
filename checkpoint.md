# Checkpoint Log

> Maintained by opencode — every session, every instruction, logged here so nothing is forgotten.

---

## Sessions

### Session 1 — 2026-05-27
- Created `checkpoint.md` with full build plan context
- Discussed architecture: signalling server + P2P vs relay
- AWS free tier chosen for hosting
- Domain: `relay.opendev.website` (Cloudflare)

### Session 2 — 2026-05-28
- **Phase 1.1** — CLI Foundation: `cargo init` workspace (`vb` + `voiddrop-server`), clap args, banner, colors, progress stubs, helpers
- **Phase 1.2** — Fast Mode: voiddrop-server TCP signalling, P2P pipeline, session create/join protocol, file streaming with progress bar
- **Phase 1.3** — Secure Mode: AES-256-GCM encrypt/decrypt via `aes-gcm` crate, 4-digit code with SHA-256 key derivation, encrypted framing
- **Phase 1.4** — Secure-Blast Mode: 4-char alphanumeric codes, server-side code validation on join, filename + code required
- **Phase 1.5** — QR System: terminal QR rendering via `qrcode` crate, temp HTTP server (tokio-based, no framework), inline HTML code entry page for secure QR
- **Phase 1.6** — Beauty & Atmosphere: spinners for connecting, clipboard auto-copy via `arboard`, summary boxes (`┌──┐`), styled errors, transfer timing + speed
- Fixed: Fast mode no longer uses codes (uses filename directly)
- Fixed: `vb receive` optional second arg for blast codes
- All helps populated (`--help` descriptions on all flags and args)
- End-to-end test passed: Fast mode send/receive over localhost

---

## Architecture (Current)

| Decision | Choice |
|---|---|
| Binary name | `vb` (e.g. `vb send`, `vb receive`) |
| Transfer mechanism | **Signalling server + P2P direct** with relay fallback |
| QR mobile bridge | Temp HTTP server on sender's machine |
| Platform target | Cross-platform (Windows/macOS/Linux) |
| Server packaging | Separate crate `voiddrop-server` in same workspace |
| Default server address | `relay.opendev.website:9876` |
| Default local test | `127.0.0.1:9876` |

### How it works
1. Sender connects to signalling server, creates session (keyed by filename for fast, code for secure/blast)
2. Receiver connects to signalling server, joins session
3. Sender starts P2P TCP listener, sends address via signalling
4. Receiver connects directly to sender (P2P)
5. File streams over direct connection with optional encryption
6. QR: sender starts temp HTTP server, serves file for mobile download

---

## Project Structure (Current)

```
voiddrop/
├── Cargo.toml                        # workspace root
├── checkpoint.md
├── vb/                               # client binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                   # entry, command routing, styled error handler
│       ├── cli/
│       │   ├── mod.rs
│       │   └── args.rs               # clap: send/receive with all flags + help text
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── send.rs               # send handler (all 3 modes + QR + clipboard)
│       │   └── receive.rs            # receive handler (3-mode detection + summary)
│       ├── transfer/
│       │   ├── mod.rs
│       │   ├── session.rs            # signalling protocol (JSON lines over TCP)
│       │   ├── pipeline.rs           # P2P file streaming + encrypted framing + stats
│       │   ├── crypto.rs             # AES-256-GCM encrypt/decrypt + key derivation
│       │   └── server.rs             # temp HTTP server for QR mobile downloads
│       ├── ui/
│       │   ├── mod.rs
│       │   ├── banner.rs             # splash + mode headers
│       │   ├── colors.rs             # color role definitions
│       │   ├── progress.rs           # indicatif wrappers (bar + spinner)
│       │   └── qr.rs                 # QR terminal rendering + local IP detection
│       └── util/
│           ├── mod.rs
│           └── helpers.rs            # code gen, size/duration/speed format, clipboard
└── voiddrop-server/                  # server binary (deployed to AWS)
    ├── Cargo.toml
    └── src/
        ├── main.rs                   # TCP listener, connection handler
        └── session.rs                # session manager, JSON protocol handler
```

---

## Dependencies (vb)

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
colored = "2"
indicatif = "0.17"
qrcode = "0.14"
image = "0.24"
aes-gcm = "0.10"
sha2 = "0.10"
rand = "0.8"
uuid = { version = "1", features = ["v4"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
arboard = "3"
```

## Dependencies (voiddrop-server)

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
anyhow = "1"
```

---

## Command Reference

```
vb send <FILE> [OPTIONS]
  --fast             Fast mode — instant P2P transfer, no encryption
  --secure           Secure mode — AES-256-GCM encrypted with 4-digit code
  --secure-blast     Blast mode — encrypted + filename + alphanumeric code
  --qr               Generate QR code for mobile download
  --relay <ADDR>     Signalling server address (default: relay.opendev.website:9876)

vb receive <IDENTIFIER> [CODE]
  IDENTIFIER         Filename (fast) or session code (secure)
  CODE               Blast code (required for --secure-blast mode)
  --relay <ADDR>     Signalling server address (default: relay.opendev.website:9876)
```

### Mode Detection (Receive)

| Arguments | Mode | Example |
|---|---|---|
| 1 arg, not 4 digits | Fast | `vb receive photo.jpg` |
| 1 arg, 4 digits | Secure | `vb receive 4829` |
| 2 args | Blast | `vb receive plans.zip X9P1` |

### QR Support

| Mode | QR | Phone experience |
|---|---|---|
| Fast + `--qr` | ✅ | Scan QR → instant download |
| Secure + `--qr` | ✅ | Scan QR → webpage → enter code → download |
| Blast + `--qr` | ❌ | Warning: not available in Phase 1 |

---

## Phase Status

- [x] **Phase 1.1 — CLI Foundation** — Workspace, args, banner, colors, progress stubs
- [x] **Phase 1.2 — Fast Mode** — Signalling protocol, P2P file streaming, progress bar
- [x] **Phase 1.3 — Secure Mode** — AES-256-GCM encrypt/decrypt, 4-digit codes
- [x] **Phase 1.4 — Secure-Blast Mode** — Alphanumeric codes, server-side validation
- [x] **Phase 1.5 — QR System** — Terminal QR rendering, temp HTTP server, mobile webpage
- [x] **Phase 1.6 — Beauty & Atmosphere** — Spinners, clipboard, summary boxes, styled errors
- [ ] **Deploy to AWS** — t2.micro (free tier), Ubuntu 24.04, port 9876, Cloudflare DNS

---

## End-to-End Test Results

- Fast mode: send/receive over localhost ✅
- Signalling server: session create/join/P2P ready ✅
- Progress bar: real-time speed/ETA display ✅
- Clipboard auto-copy: code copied automatically ✅
- Summary box: Transfer Complete / Download Complete displayed ✅
- Styled errors: `✖ Error:` in red (no raw panics) ✅
- QR: URL generation + terminal rendering ✅
- Help text: all flags/args documented ✅

---

## AWS Deployment Plan

1. Launch t2.micro (free tier), Ubuntu 24.04
2. Security group: port 22 (SSH) + port 9876 (signalling)
3. SSH in, install Rust, `cargo build --release --bin voiddrop-server`
4. Create systemd service for persistence
5. Cloudflare DNS: `relay.opendev.website` → A record → EC2 public IP

---

## Do NOT add

- accounts
- cloud storage
- GUI app
- AI
- P2P complexity
- multi-user systems
- social systems
- plugins
