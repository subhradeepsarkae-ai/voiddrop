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

### Session 3 — 2026-05-28 (Final — Ship)
- AWS → Railway: Deployed `voiddrop-server` to Railway (free tier, no wait time)
- Dockerfile + `railway.json` created for Railway deploy
- Railway TCP proxy configured: `zephyr.proxy.rlwy.net:12963`
- **GitHub Release v0.1.0**: `vb.exe` uploaded with install scripts
- `install.ps1`: Auto-downloads `vb.exe` to `~/.voiddrop/`, adds to PATH
- `uninstall.ps1`: Clean removal of vb.exe, folder, and PATH entry
- `vb.exe` rebuilt with `zephyr.proxy.rlwy.net:12963` as default relay (no `--relay` flag needed)
- **Rich `--help`**: Full process guides for every flag in `vb --help`, clean minimal output in `vb send --help` and `vb receive --help`
- Repository made public for raw.githubusercontent.com access
- All modes tested and working: Fast, Secure, Secure-Blast, QR
- PATH configured: `vb` works from any directory

### Session 4 — 2026-05-29 (v0.2.0 — Clip + Global QR)
- **Phase A: `--clip` flag** — `vb/src/util/clipboard.rs` created with `ClipboardFile` struct + `read_clipboard_file()` using `arboard` (file_list + text fallback)
- `--clip` on `send`: reads clipboard → validates path → shows detection UI → uses resolved path
- `--clip` on `receive`: reads clipboard → extracts filename → forces fast mode
- `file` / `identifier` made `Option<String>` in clap; validation ensures either file path or `--clip`
- **Phase B: `--global-qr` flag** — Relay-based QR that works from anywhere (not just LAN)
- Protocol extended: `Create.worldwide_qr`, `UploadStart`, `UploadChunk`, `UploadComplete` messages
- `voiddrop-server` now handles HTTP requests on the same TCP port (protocol detection via first-byte peek)
- HTTP endpoint `GET /dl/<session-id>` serves buffered file; secure mode shows HTML code page
- File upload via base64-encoded chunks, buffered in memory only
- `base64` crate added to both `vb` and `voiddrop-server`
- End-to-end test passed: Fast mode send/receive over localhost relay
- Server verified: HTTP detection + signalling protocol multiplexed on single port
- Railway TCP proxy configured: `zephyr.proxy.rlwy.net:12963`
- `install.ps1`: Auto-downloads `vb.exe` to `~/.voiddrop/`, adds to PATH
- `uninstall.ps1`: Clean removal of vb.exe, folder, and PATH entry
- `vb.exe` rebuilt with `zephyr.proxy.rlwy.net:12963` as default relay (no `--relay` flag needed)
- **Rich `--help`**: Full process guides for every flag in `vb --help`, clean minimal output in `vb send --help` and `vb receive --help`
- Repository made public for raw.githubusercontent.com access
- All modes tested and working: Fast, Secure, Secure-Blast, QR
- PATH configured: `vb` works from any directory

### Session 5 — 2026-05-29 (v0.1.1 — Relay download fix)
- **Critical bug: relay download timed out** — diagnosed from user screenshots
- **Root cause 1**: Server sent `UploadComplete` immediately on `UploadStart`, before any chunks were uploaded
- **Root cause 2**: Sender sent `"done"` after relay upload, deleting the session before the receiver could HTTP download
- **Fix 1** (`voiddrop-server/src/session.rs`): Removed premature `UploadComplete` from `UploadStart` handler — server now only sends it on the last chunk
- **Fix 2** (`vb/src/commands/send.rs`): Removed `send("done")` after global-QR upload — session stays alive
- **Fix 3** (`vb/src/transfer/pipeline.rs`): Removed `send("done")` from `relay_upload` — same reason
- Server redeployed to Railway via API (CLI auth issues worked around via GraphQL API)
- **GitHub Release v0.1.1**: `vb.exe` with relay fixes, auto-downloaded by existing `install.ps1`

---

## Architecture (Current)

| Decision | Choice |
|---|---|
| Binary name | `vb` (e.g. `vb send`, `vb receive`) |
| Transfer mechanism | **Signalling server + P2P direct** with relay fallback |
| QR mobile bridge | Temp HTTP server on sender's machine |
| Platform target | Cross-platform (Windows/macOS/Linux) |
| Server packaging | Separate crate `voiddrop-server` in same workspace |
| Hosting | **Railway** (free tier) |
| Relay address | `zephyr.proxy.rlwy.net:12963` |
| TCP proxy | Railway TCP proxy on `proxy.rlwy.net` |

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
├── Dockerfile                        # multistage Rust build for Railway
├── railway.json                       # Railway deploy config
├── .gitignore
├── install.ps1                        # one-click install + PATH setup
├── uninstall.ps1                      # one-click uninstall + PATH cleanup
├── vb/                               # client binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                   # entry, command routing, styled error handler
│       ├── cli/
│       │   ├── mod.rs
│       │   └── args.rs               # clap: send/receive with rich help docs
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
└── voiddrop-server/                  # server binary (deployed to Railway)
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
base64 = "0.22"
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
base64 = "0.22"
```

---

## Command Reference

### Send
```
vb send [<FILE> | --clip] [OPTIONS]

Options:
  --fast             Fast mode — no encryption, receiver just needs filename
  --secure           Secure mode — AES-256-GCM encrypted, 4-digit code
  --secure-blast     Secure-Blast mode — encrypted + filename + alphanumeric code
  -q, --qr           Generate QR code for mobile download over WiFi (local)
  --global-qr        Global QR — relay serves file, phone downloads from anywhere
  -c, --clip         Read file path from clipboard instead of typing
  --relay <ADDR>     Signalling server [default: zephyr.proxy.rlwy.net:12963]

Examples:
  vb send photo.jpg --fast
  vb send docs.zip --secure
  vb send secret.zip --secure-blast
  vb send video.mp4 --fast --qr
  vb send --clip --fast
  vb send --clip --secure
  vb send photo.jpg --fast --global-qr
  vb send --clip --fast --global-qr
```

### Receive
```
vb receive [<IDENTIFIER> | --clip] [CODE] [OPTIONS]

Options:
  -c, --clip         Read filename from clipboard instead of typing
  --relay <ADDR>     Signalling server [default: zephyr.proxy.rlwy.net:12963]

Mode Detection:
  1 arg, not 4 digits      → Fast mode:   vb receive photo.jpg
  1 arg, exactly 4 digits   → Secure mode:  vb receive 4829
  2 args                    → Blast mode:   vb receive plans.zip X9P1
  --clip (no arg)           → Fast mode:   vb receive --clip
```

### QR Support

| Mode | QR | Phone experience |
|---|---|---|
| Fast + `--qr` | ✅ Local | Scan QR → instant download (same WiFi) |
| Secure + `--qr` | ✅ Local | Scan QR → webpage → enter code → download (same WiFi) |
| Blast + `--qr` | ❌ | Not available |
| Fast + `--global-qr` | ✅ Global | Scan QR → instant download (anywhere) |
| Secure + `--global-qr` | ✅ Global | Scan QR → webpage → enter code → download (anywhere) |
| Blast + `--global-qr` | ❌ | Not available |

---

## Phase Status

- [x] **Phase 1.1 — CLI Foundation** — Workspace, args, banner, colors, progress stubs
- [x] **Phase 1.2 — Fast Mode** — Signalling protocol, P2P file streaming, progress bar
- [x] **Phase 1.3 — Secure Mode** — AES-256-GCM encrypt/decrypt, 4-digit codes
- [x] **Phase 1.4 — Secure-Blast Mode** — Alphanumeric codes, server-side validation
- [x] **Phase 1.5 — QR System** — Terminal QR rendering, temp HTTP server, mobile webpage
- [x] **Phase 1.6 — Beauty & Atmosphere** — Spinners, clipboard, summary boxes, styled errors
- [x] **Deploy to Railway** — TCP proxy on `zephyr.proxy.rlwy.net:12963`
- [x] **Install Script** — `install.ps1` with auto-PATH, `uninstall.ps1` for cleanup
- [x] **Rich --help docs** — Full process guides in `vb --help`, clean output in subcommands
- [x] **Phase A: --clip flag** — Send files from clipboard without typing paths
- [x] **Phase B: --global-qr flag** — Relay-based QR works from anywhere (not just LAN)
- [x] **Relay download fix v0.1.1** — Fixes "Relay download timed out" when P2P is blocked by NAT

---

## End-to-End Test Results

- Fast mode: send/receive over localhost ✅
- Fast mode: send/receive over Railway relay ✅
- Signalling server: session create/join/P2P ready ✅
- Progress bar: real-time speed/ETA display ✅
- Clipboard auto-copy: code copied automatically ✅
- Summary box: Transfer Complete / Download Complete displayed ✅
- Styled errors: `✖ Error:` in red (no raw panics) ✅
- QR: URL generation + terminal rendering ✅
- Help text: full process guide in `vb --help`, clean in subcommands ✅
- PATH: `vb` works from any directory ✅
- Railway TCP proxy: `zephyr.proxy.rlwy.net:12963` ✅
- One-liner install: `iwr -useb .../install.ps1 | iex` ✅
- One-liner uninstall: `iwr -useb .../uninstall.ps1 | iex` ✅
- **--clip flag**: send/receive with clipboard detection ✅
- **--global-qr**: relay-based file upload + HTTP endpoint ✅
- **Protocol multiplexing**: HTTP + signalling on single TCP port ✅
- **Relay download fix**: relay fallback no longer times out ✅

---

## Install / Uninstall (for users)

### Install (one-liner, Windows):
```powershell
iwr -useb https://raw.githubusercontent.com/subhradeepsarkae-ai/voiddrop/master/install.ps1 | iex
```

### Uninstall (one-liner, Windows):
```powershell
iwr -useb https://raw.githubusercontent.com/subhradeepsarkae-ai/voiddrop/master/uninstall.ps1 | iex
```

---

## GitHub Release

- Release: `v0.1.0 — voiddrop P2P File Transfer`
- Link: https://github.com/subhradeepsarkae-ai/voiddrop/releases/tag/v0.1.0
- Binary: `vb.exe` (Windows x64, ~1.9 MB)
- Platform targets: Windows (Linux/macOS: build from source with `cargo build --release --bin vb`)

- Release: `v0.1.1 — Relay download fix`
- Link: https://github.com/subhradeepsarkae-ai/voiddrop/releases/tag/v0.1.1
- Binary: `vb.exe` (Windows x64, ~2.0 MB)
- Fixes relay download timeout when P2P is blocked by NAT
- `voiddrop-server` redeployed to Railway with HTTP endpoint

---

## Upcoming (v0.3.0)

- [ ] **HTTPS for relay QR** — Configure Railway HTTP proxy so `--global-qr` uses `https://` instead of `http://`
- [ ] **Drag-and-drop** — Let users drag a file onto the terminal window for `vb send`
- [ ] **Receive path flag** — `vb receive --out <dir>` to specify download directory
- [ ] **--clip on receive secure** — `vb receive --clip <code>` (filename from clipboard, code from arg)

## Do NOT add

- accounts
- cloud storage
- GUI app
- AI
- P2P complexity
- multi-user systems
- social systems
- plugins
