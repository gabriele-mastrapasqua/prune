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

### Via Homebrew (Recommended)
```bash
brew tap gabriele-mastrapasqua/prune
brew install prune
```

### Via Makefile
```bash
git clone https://github.com/gabriele-mastrapasqua/prune.git
cd prune
make install
```

This will build and install to `~/.local/bin/prune` (no password required). If `~/.local/bin` is not in your PATH, add this to your `~/.zshrc` or `~/.bashrc`:
```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Manual
```bash
git clone https://github.com/gabriele-mastrapasqua/prune.git
cd prune
cargo build --release
mkdir -p ~/.local/bin
cp target/release/prune ~/.local/bin/
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
make test          # Run fmt, lint, and tests
make build         # Build in debug mode
make release       # Build release binary
make install       # Clean, build, install to /usr/local/bin
make uninstall     # Remove from /usr/local/bin
```

## 📄 License

Apache-2.0

## 🙏 Acknowledgments

- Fork of [dust](https://github.com/bootandy/dust) by bootandy
- Built with [ratatui](https://github.com/ratatui/ratatui) for TUI
- Uses [rayon](https://github.com/rayon-rs/rayon) for parallelism
