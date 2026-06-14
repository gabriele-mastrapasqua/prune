# 🧹 prune

**Interactive TUI Disk Cleanup Advisor** — Like `dust`, but tells you what to delete.

A fast, intelligent disk usage analyzer that helps you reclaim disk space by identifying cache files, old dependencies, unused package managers, and AI models.

![Rust](https://img.shields.io/badge/rust-1.96+-orange.svg)
![Platform](https://img.shields.io/badge/platform-macOS-blue.svg)
![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)

## ✨ Features

- **Interactive TUI**: Run `prune` for a guided cleanup experience
- **Smart Analysis**: Detects caches, old dependencies, AI models, Docker bloat
- **Version Managers**: Scans nvm, pyenv, rustup, volta, fnm, asdf, mise, conda
- **Package Managers**: Finds unused tools (npm, pnpm, yarn, bun, uv, poetry)
- **AI Models**: Locates HuggingFace, Ollama, LM Studio, Whisper models
- **Risk Levels**: Every recommendation rated (Safe → Danger)
- **History**: Track cleanup trends over time

## 🚀 Installation

```bash
git clone https://github.com/gabrielemastrapasqua/prune.git
cd prune
cargo build --release
./target/release/prune
```

Or add to PATH:
```bash
sudo cp target/release/prune /usr/local/bin/
```

## 📖 Usage

### Interactive TUI (Recommended)
```bash
prune              # Launch TUI
prune -i           # Same, explicit
```

### Advisor Mode (Text Output)
```bash
prune -a           # Full scan
prune -a --risk safe      # Only safe items
prune -a --category ai    # Only AI models
prune -a --no-dev         # Skip dev tools
prune -a --json           # JSON output
```

### Explorer Mode (Like dust)
```bash
prune ~/Library    # Analyze directory
prune ~/Library -n 10     # Top 10 largest
prune ~/ -F               # Find largest files
```

### History
```bash
prune --history    # View scan history sparkline
```

## 🎯 Risk Levels

| Badge | Level | Meaning |
|-------|-------|---------|
| ✅ SAFE | Safe | No risk, can delete freely |
| 🟡 LOW | Low | Low risk, usually safe |
| 🟠 MEDIUM | Medium | Review before deleting |
| 🔴 REVIEW | Review | Manual review required |
| ⛔ DANGER | Danger | Never auto-delete |

## 🔧 Configuration

```bash
prune -a --min-size 500M    # Default: 100 MB
prune -a --risk low         # Default: review (shows all)
prune -a --category cache   # Filter by category
```

## 🧪 Development

```bash
cargo build --release
cargo test
cargo run --release -- -a
```

## 📄 License

Apache-2.0

## 🙏 Acknowledgments

- Fork of [dust](https://github.com/bootandy/dust) by bootandy
- Built with [ratatui](https://github.com/ratatui/ratatui) for TUI
- Uses [rayon](https://github.com/rayon-rs/rayon) for parallelism
