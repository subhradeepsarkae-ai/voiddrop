# ⚡ VoidDrop

**Drop files to anyone, instantly — no setup, no accounts.**

Peer-to-peer file transfer over the terminal. Files stream directly between machines. The relay only coordinates.

```
vb send photo.jpg --fast
vb receive photo.jpg
```

---

## Install (Windows)

```powershell
iwr -useb https://raw.githubusercontent.com/subhradeepsarkae-ai/voiddrop/master/install.ps1 | iex
```

Installs `vb.exe` to `~/.voiddrop/` and adds it to PATH. Then just type `vb` from anywhere.

---

## Quick Start

### Fast Mode — no encryption
```bash
# Sender
vb send song.mp3 --fast

# Receiver (needs the filename)
vb receive song.mp3
```

### Secure Mode — AES-256-GCM encrypted
```bash
# Sender (4-digit code auto-copied to clipboard)
vb send report.pdf --secure

# Receiver (enter the code)
vb receive 4829
```

### Secure-Blast Mode — encrypted + filename + code
```bash
# Sender
vb send plans.zip --secure-blast

# Receiver (needs both filename and code)
vb receive plans.zip X9P1
```

---

## 📋 New: `--clip` Flag

Send files directly from your clipboard — no typing paths.

```bash
# Copy a file in Explorer (Ctrl+C), then:
vb send --clip --fast

# Or with encryption:
vb send --clip --secure

# Receiver gets filename from clipboard too:
vb receive --clip
```

**How it works:**
1. Ctrl+C a file in Explorer / Finder
2. `vb send --clip --fast`
3. VoidDrop reads the file path from clipboard, shows detected file + size
4. Transfer starts instantly

Works with copied files (file manager) or copied path text.

---

## 🌍 New: `--global-qr` Flag

QR that works from **anywhere** — phone doesn't need to be on same WiFi.

```
vb send photo.jpg --fast --global-qr
vb send --clip --secure --global-qr
```

The file uploads through the relay (in memory, no disk). Phone scans QR and downloads directly from the relay.

| Mode | Local QR (`--qr`) | Global QR (`--global-qr`) |
|---|---|---|
| Fast | Scan → instant download (same WiFi) | Scan → instant download (anywhere) |
| Secure | Scan → webpage → enter code → download | Same, but works from anywhere |
| Blast | ❌ Not available | ❌ Not available |

---

## 🔳 Local QR (`--qr`)

Scan with your phone to download over WiFi (same network required).

```bash
vb send video.mp4 --fast --qr
vb send document.pdf --secure --qr
```

Phone just needs a browser — no app required. For secure mode, the phone shows a code entry page.

---

## 📖 Full Command Reference

### Send
```
vb send [<FILE> | --clip] [OPTIONS]

Options:
  --fast             Fast mode — no encryption, receiver just needs filename
  --secure           Secure mode — AES-256-GCM encrypted, 4-digit code
  --secure-blast     Secure-Blast mode — encrypted + filename + alphanumeric code
  -q, --qr           Local QR — phone downloads over WiFi (same network)
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

---

## 🏗 Architecture

```
SENDER ───signalling──→ RELAY ←──signalling─── RECEIVER
  │                       │
  └─────── P2P TCP ──────→┘     (direct connection)
         (or via relay for global QR)
```

- **Signalling server**: Coordinates session creation/joining (Railway)
- **P2P transfer**: File streams directly between machines
- **Relay fallback**: Global QR uploads through relay (in-memory buffer)

---

## 🚀 Build from Source

```bash
git clone https://github.com/subhradeepsarkae-ai/voiddrop.git
cd voiddrop
cargo build --release --bin vb
./target/release/vb send photo.jpg --fast
```

---

## 📦 Releases

| Version | Features |
|---|---|
| v0.1.0 | Fast, Secure, Blast modes; Local QR; Install scripts |
| v0.2.0 | `--clip` flag, `--global-qr` flag |

---

## Rules

- ❌ No accounts
- ❌ No cloud storage
- ❌ No GUI
- ❌ No AI
- ❌ No plugins
