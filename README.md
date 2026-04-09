# 🧅 OnionDownOda

> A beautiful, high-performance TUI for downloading files from `.onion` URLs over Tor — built in Rust from Kigali 🇷🇼

[![crates.io](https://img.shields.io/crates/v/oniondownoda.svg)](https://crates.io/crates/oniondownoda)
[![AUR](https://img.shields.io/aur/version/oniondownoda)](https://aur.archlinux.org/packages/oniondownoda)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/amigoDcyber/OnionDownOda)](https://github.com/amigoDcyber/OnionDownOda)

---

## 🖼️ Preview

```
╔══════════════════════════════════════════════════════════════╗
║  [1] Instagram          🧅 ONION          [5] X/Twitter     ║
║  instagram.com/...       DownOda          x.com/...         ║
║  [2] Facebook                             [6] Snapchat      ║
║  facebook.com/...                         snapchat.com/...  ║
╠══════════════════════════════════════════════════════════════╣
║  🧅 Tor Status: ● Connected (socks5h://127.0.0.1:9050)     ║
╠══════════════════════════════════════════════════════════════╣
║  📎 Paste .onion URL                                        ║
║  http://example.onion/file.zip▉                             ║
╠══════════════════════════════════════════════════════════════╣
║  📥 Downloads                                               ║
║  ⏳ file.zip                                                ║
║    ████████████░░░░░░ 45%  2.5 MB/s  ETA 12s               ║
╠══════════════════════════════════════════════════════════════╣
║  📋 Log (scroll ↑↓ PgUp/PgDn)                              ║
║  [22:28:39] 🧅 Welcome to OnionDownOda                     ║
╠══════════════════════════════════════════════════════════════╣
║  ⚠️  Developer is NOT responsible for unauthorized downloads ║
║  [Enter] Download  [Space] Pause  [Tab] Focus  [Esc] Quit   ║
╚══════════════════════════════════════════════════════════════╝
```

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🎨 **Cyberpunk TUI** | Neon magenta/cyan/green aesthetic powered by `ratatui` |
| 🧅 **Tor Native** | SOCKS5 proxy support for `.onion` URLs via `reqwest` |
| ⚡ **Parallel Downloads** | Up to 100 concurrent connections for maximum speed |
| ⏸️ **Pause/Resume** | Press `Space` to pause and resume any active download |
| 📊 **Live Progress** | Real-time progress bars with speed, ETA, and byte count |
| 📋 **Activity Log** | Timestamped events with colored status indicators |
| ⚙️ **Configurable** | CLI args + optional TOML config file |
| 🔗 **Social Links** | Press `1-8` anywhere to open developer profiles in browser |
| 📱 **Responsive** | Adapts layout to wide, medium, and narrow terminal sizes |

---

## 📦 Installation

### Prerequisites

- **Tor** — must be running on your system before launching the app

#### Install & Start Tor

```bash
# Arch Linux / Garuda / Manjaro
sudo pacman -S tor
sudo systemctl enable --now tor

# Ubuntu / Debian
sudo apt install tor
sudo systemctl enable --now tor

# Fedora
sudo dnf install tor
sudo systemctl enable --now tor

# macOS
brew install tor
brew services start tor
```

---

### Via crates.io *(Recommended)*

```bash
cargo install oniondownoda
```

> Requires Rust 1.70+ — install via [rustup.rs](https://rustup.rs/)

---

### Via AUR *(Arch / Garuda / Manjaro)*

```bash
yay -S oniondownoda
# or
paru -S oniondownoda
```

---

### Via GitHub Releases *(No Rust required)*

Download the pre-built binary for your platform from the [Releases page](https://github.com/amigoDcyber/OnionDownOda/releases/latest):

```bash
# After downloading, make it executable
chmod +x oniondownoda
./oniondownoda
```

---

### Build from Source

```bash
git clone https://github.com/amigoDcyber/OnionDownOda.git
cd OnionDownOda
cargo build --release
./target/release/oniondownoda
```

---

## 🚀 Usage

```bash
# Run with defaults (Tor at 127.0.0.1:9050)
oniondownoda

# Custom Tor proxy (e.g., Tor Browser on port 9150)
oniondownoda --proxy socks5h://127.0.0.1:9150

# Custom output directory
oniondownoda --output-dir ~/Downloads/onion

# Verbose logging
oniondownoda --verbose
```

---

## ⌨️ Keyboard Shortcuts

| Key | Action |
|:---:|--------|
| `Enter` | Start download from URL input |
| `Tab` | Switch focus between Input and Downloads |
| `Space` | Pause / Resume selected download |
| `↑` / `↓` | Scroll downloads or log |
| `PgUp` / `PgDn` | Scroll log faster |
| `Esc` | Quit |
| `Ctrl+C` | Force quit |

### Social Shortcuts *(press anywhere)*

| Key | Platform | URL |
|:---:|----------|-----|
| `1` | 📷 Instagram | [instagram.com/amigo.d.cyber](https://www.instagram.com/amigo.d.cyber) |
| `2` | 📘 Facebook | [facebook.com/amigo.d.cyber](https://www.facebook.com/amigo.d.cyber) |
| `3` | 📺 YouTube | [youtube.com/@CyberMafiaX](https://www.youtube.com/@CyberMafiaX) |
| `4` | 🎵 TikTok | [tiktok.com/@amigo.d.cyber](https://www.tiktok.com/@amigo.d.cyber) |
| `5` | 🐦 X/Twitter | [x.com/MafiaCyberX](https://x.com/MafiaCyberX) |
| `6` | 👻 Snapchat | [snapchat.com/add/amigo-cyber](https://www.snapchat.com/add/amigo-cyber) |
| `7` | 📌 Pinterest | [pinterest.com/amigodcyber](https://www.pinterest.com/amigodcyber/) |
| `8` | 🐙 GitHub | [github.com/amigoDcyber](https://github.com/amigoDcyber) |
| `0` | 🌳 Linktree | [linktr.ee/Amigo.D.Cyber](https://linktr.ee/Amigo.D.Cyber) |

---

## ⚙️ Configuration

Config file location:
- **Linux/macOS:** `~/.config/oniondownoda/config.toml`
- **Windows:** `%APPDATA%\oniondownoda\config.toml`

```toml
# Tor SOCKS5 proxy
proxy = "socks5h://127.0.0.1:9050"

# Output directory for downloads
output_dir = "./downloads"

# Enable verbose logging
verbose = true
```

**Priority:** CLI args → Config file → Defaults

---

## 🔧 Download States

| State | Icon | Description |
|-------|:----:|-------------|
| In Progress | ⏳ | Actively downloading with live progress bar |
| Paused | ⏸️ | Paused — press `Space` to resume |
| Completed | ✅ | Finished successfully |
| Failed | ❌ | Error occurred — check log for details |

---

## 🛠️ Tor Troubleshooting

If you see `● Disconnected` in the status bar:

```bash
# Check if Tor is running
sudo systemctl status tor

# Test connectivity
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org/api/ip

# Restart Tor
sudo systemctl restart tor
```

---

## 🏗️ Architecture

| Module | Purpose |
|--------|---------|
| `main.rs` | Entry point, terminal setup, async event loop |
| `app.rs` | App state machine, input handling, download tracking |
| `ui.rs` | TUI rendering — banner, panels, progress bars, socials |
| `banner.rs` | ASCII art branding |
| `downloader.rs` | Parallel HTTP download engine with chunk support |
| `tor.rs` | SOCKS5 connectivity check and reqwest client builder |
| `config.rs` | CLI args (clap) + TOML config merging |
| `error.rs` | Error types with user-friendly messages |

---

## 👤 Developer

**Amigo D. Cyber** — Cybersecurity enthusiast & Rust developer from Kigali, Rwanda 🇷🇼

| Platform | Link |
|----------|------|
| 🐙 GitHub | [@amigoDcyber](https://github.com/amigoDcyber) |
| 📷 Instagram | [@amigo.d.cyber](https://www.instagram.com/amigo.d.cyber) |
| 📺 YouTube | [@CyberMafiaX](https://www.youtube.com/@CyberMafiaX) |
| 🎵 TikTok | [@amigo.d.cyber](https://www.tiktok.com/@amigo.d.cyber) |
| 🐦 X/Twitter | [@MafiaCyberX](https://x.com/MafiaCyberX) |
| 👻 Snapchat | [amigo-cyber](https://www.snapchat.com/add/amigo-cyber) |
| 📌 Pinterest | [amigodcyber](https://www.pinterest.com/amigodcyber/) |
| 🌳 All Links | [linktr.ee/Amigo.D.Cyber](https://linktr.ee/Amigo.D.Cyber) |

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

## 🤝 Contributing

PRs, issues, and forks are welcome. If you find a bug or want a feature, open an issue on GitHub.

---

## ⚠️ Disclaimer

This tool is for **educational and research purposes only**. Users are solely responsible for complying with local laws and the terms of service of any site accessed via Tor. **The developer assumes no liability for any files downloaded without proper authorization.**
