# 🧅 OnionDownOda

A beautiful, high-performance TUI application for downloading files from `.onion` URLs via the Tor network.

Built with **Rust** — featuring parallel chunk downloading (up to 100 connections), pause/resume support, and a stunning cyberpunk interface.

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🎨 **Cyberpunk TUI** | Neon pink/green/purple aesthetic powered by `ratatui` |
| 🧅 **Tor Native** | SOCKS5 proxy support for `.onion` URLs |
| ⚡ **Parallel Downloads** | Up to 100 concurrent connections for maximum speed |
| ⏸️ **Pause/Resume** | Press `Space` to pause and resume downloads |
| 📊 **Live Progress** | Real-time progress bars with speed, ETA, and status |
| 📋 **Activity Log** | Timestamped events with colored indicators |
| ⚙️ **Configurable** | CLI args + optional TOML config file |
| 🔗 **Social Links** | Quick access to developer social profiles |

---

## 📦 Installation

### Prerequisites

- **Rust** 1.70+ — [install via rustup](https://rustup.rs/)
- **Tor** — must be running on your system

#### Installing Tor

```bash
# Arch Linux / Manjaro
sudo pacman -S tor
sudo systemctl start tor
sudo systemctl enable tor

# Ubuntu / Debian / Linux Mint
sudo apt install tor
sudo systemctl start tor
sudo systemctl enable tor

# Fedora / RHEL
sudo dnf install tor
sudo systemctl start tor
sudo systemctl enable tor

# macOS (with Homebrew)
brew install tor
brew services start tor

# Windows
# Download Tor Browser or run Tor Expert Bundle
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/amigoDcyber/OnionDownOda.git
cd OnionDownOda

# Build optimized release binary
cargo build --release

# The binary will be at:
./target/release/oniondownoda

# Optional: Install to system
sudo cp ./target/release/oniondownoda /usr/local/bin/
```

---

## 🚀 Usage

### Quick Start

```bash
# Run with defaults (Tor at 127.0.0.1:9050)
./target/release/oniondownoda

# Custom Tor proxy (e.g., Tor Browser)
./target/release/oniondownoda --proxy socks5h://127.0.0.1:9150

# Custom output directory
./target/release/oniondownoda --output-dir ~/Downloads/onion

# With verbose logging
./target/release/oniondownoda --verbose
```

### Keyboard Shortcuts

| Key | Action |
|:---:|:---|
| `Enter` | Start download from URL input |
| `Tab` | Switch focus (Input ↔ Downloads) |
| `Space` | **Pause/Resume** download (in Downloads focus) |
| `↑` / `↓` | Scroll downloads list |
| `Esc` | Quit application |
| `Ctrl+C` | Force quit |

#### Social Media Shortcuts (Global)

Press these keys **anywhere** to open the developer's profiles:

| Key | Platform | URL |
|:---:|:---|:---|
| `1` | 📷 Instagram | instagram.com/amigo.d.cyber |
| `2` | 📘 Facebook | facebook.com/amigo.d.cyber |
| `3` | 📺 YouTube | youtube.com/@CyberMafiaX |
| `4` | 🎵 TikTok | tiktok.com/@amigo.d.cyber |
| `5` | 🐦 X (Twitter) | x.com/MafiaCyberX |
| `6` | 👻 Snapchat | snapchat.com/add/amigo-cyber |
| `7` | 📌 Pinterest | pinterest.com/amigodcyber |
| `8` | 🐙 GitHub | github.com/amigoDcyber |
| `0` | 🌳 Linktree | linktr.ee/Amigo.D.Cyber |

---

## 🖥️ Interface Guide

```
┌─────────────────────────────────────────────────────────┐
│  🧅 ONIONDOWNODA  ─  High-Speed Tor Downloader         │  ← Banner
├─────────────────────────────────────────────────────────┤
│  🧅 Tor Status: ✅ Connected (127.0.0.1:9050)        │  ← Tor Status
├─────────────────────────────────────────────────────────┤
│  🔗 URL Input                                            │  ← Input Box
│  http://example.onion/file.zip ▉                       │
├─────────────────────────────────────────────────────────┤
│  📥 Downloads                                          │  ← Downloads Panel
│  ⏳ file.zip                                          │
│    ████████████░░░░░░░ 45%  2.5 MB/s  ETA 12s         │
├─────────────────────────────────────────────────────────┤
│  📋 Log                                               │  ← Activity Log
│  [12:34:56] 🧅 Welcome to OnionDownOda               │
│  [12:35:01] 📥 Starting: file.zip (10 MB)            │
├─────────────────────────────────────────────────────────┤
│  👤 Developer & Socials                                │  ← Credits Panel
│  GitHub: @amigoDcyber  |  Press: [1-8] for socials   │
│  [1]📷 IG  [2]📘 FB  [3]📺 YT  [4]🎵 TT  [5]🐦 X   │
├─────────────────────────────────────────────────────────┤
│  [Enter] Download  [Space] Pause  [Tab] Focus  [Esc] Quit│  ← Help Bar
└─────────────────────────────────────────────────────────┘
```

---

## ⚙️ Configuration

OnionDownOda looks for a config file at:
- Linux/macOS: `~/.config/oniondownoda/config.toml`
- Windows: `%APPDATA%\oniondownoda\config.toml`

### Example `config.toml`

```toml
# Tor SOCKS5 proxy address
proxy = "socks5h://127.0.0.1:9050"

# Where to save downloaded files
output_dir = "./downloads"

# Enable verbose logging
verbose = true
```

**Priority order:** CLI args > Config file > Defaults

---

## 🔧 Advanced Usage

### Download States

| State | Indicator | Description |
|-------|:---------:|:------------|
| In Progress | ⏳ | Active downloading with progress bar |
| Paused | ⏸️ | Download paused (Space to resume) |
| Completed | ✅ | Download finished successfully |
| Failed | ❌ | Error occurred during download |

### Parallel Downloading

The app automatically uses parallel chunk downloading when:
1. Server supports HTTP Range requests
2. File size is ≥ 1 MB

Up to **100 concurrent connections** are used for maximum throughput over Tor.

### Tor Connection Troubleshooting

If you see `⚠ Tor proxy not responding`:

```bash
# Check if Tor is running
sudo systemctl status tor

# Test proxy connectivity
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org/api/ip

# Restart Tor
sudo systemctl restart tor
```

---

## 👤 Developer

**Amigo D. Cyber** — Cybersecurity enthusiast & Rust developer

- 🐙 GitHub: [@amigoDcyber](https://github.com/amigoDcyber)
- 🌳 All Links: [linktr.ee/Amigo.D.Cyber](https://linktr.ee/Amigo.D.Cyber)

### Social Media

| Platform | Handle | Link |
|----------|--------|------|
| Instagram | @amigo.d.cyber | [instagram.com/amigo.d.cyber](https://www.instagram.com/amigo.d.cyber) |
| Facebook | @amigo.d.cyber | [facebook.com/amigo.d.cyber](https://www.facebook.com/amigo.d.cyber) |
| YouTube | @CyberMafiaX | [youtube.com/@CyberMafiaX](https://www.youtube.com/@CyberMafiaX) |
| TikTok | @amigo.d.cyber | [tiktok.com/@amigo.d.cyber](https://www.tiktok.com/@amigo.d.cyber) |
| X/Twitter | @MafiaCyberX | [x.com/MafiaCyberX](https://x.com/MafiaCyberX) |
| Snapchat | amigo-cyber | [snapchat.com/add/amigo-cyber](https://www.snapchat.com/add/amigo-cyber) |
| Pinterest | amigodcyber | [pinterest.com/amigodcyber](https://www.pinterest.com/amigodcyber/) |

---

## 🏗️ Architecture

| Module | Purpose |
|--------|---------|
| `main.rs` | Entry point, terminal setup, async event loop |
| `app.rs` | App state machine, input handling, download tracking, pause/resume logic |
| `ui.rs` | TUI rendering — banner, panels, progress bars, credits panel |
| `banner.rs` | ASCII art branding |
| `downloader.rs` | Parallel HTTP download engine with chunk support |
| `tor.rs` | SOCKS5 connectivity check and reqwest client builder |
| `config.rs` | CLI args (clap) + TOML config merging |
| `error.rs` | Error types with user-friendly messages |

---

## � License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

---

## 🤝 Contributing

Contributions are welcome! Feel free to:
- Open an issue for bugs or feature requests
- Submit a pull request
- Fork and customize for your needs

---

## ⚠️ Disclaimer

This tool is for educational and research purposes. Users are responsible for complying with local laws and service terms when downloading content via Tor. The developer assumes no liability for misuse.
