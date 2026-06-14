# 🧹 prune

**Interactive TUI Disk Cleanup Advisor** — Like `dust`, but tells you what to delete.

A fast, intelligent disk usage analyzer with an interactive TUI that helps you reclaim disk space by identifying cache files, old dependencies, unused package managers, and AI models. Works across macOS, Linux, and Windows.

![Rust](https://img.shields.io/badge/rust-1.81+-orange.svg)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue.svg)
![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)

## ✨ Features

### 🎯 Smart Analysis
- **Version Manager Scanner**: Detects all Node/Python/Rust versions (nvm, pyenv, volta, fnm, asdf, mise, conda, rustup)
  - Identifies EOL versions (Node 16, 18) safe to remove
  - Shows current vs old versions with proper semver resolution
  - Detects heavy ML libraries (torch, transformers, diffusers, onnxruntime)
- **Package Manager Detection**: Finds installed but unused tools
  - Scans for lockfiles to determine actual usage
  - Identifies cache bloat (npm, pnpm, yarn, bun, uv, poetry)
- **AI Model Scanner**: Locates large models and datasets
  - HuggingFace hub individual model/dataset breakdown
  - Ollama, LM Studio, Whisper model detection
  - CUDA toolkit detection (not needed on Mac)
- **Xcode Cleanup**: DerivedData, DeviceSupport, Archives, simulators
- **Docker Analysis**: Disk image size and prune suggestions
- **Download Scanning**: Old installers (DMG, PKG, ISO) in Downloads

### 🖥️ Interactive TUI (Default Mode)
Just run `prune` to launch the interactive interface:

```
╔═══════════════════════════════════════╗
║  🧹 PRUNE - Disk Cleanup Advisor   ║
╠═══════════════════════════════════════╣
║  [1] Full Scan (DEV mode: ON)         ║
║  [2] Scan without DEV                 ║
║  [3] Scan AI Models Only              ║
║  [4] View History                     ║
║  [5] Settings                         ║
║  [6] Help                             ║
╚═══════════════════════════════════════╝
```

**Navigation:**
- `↑/↓` or `j/k`: Navigate menu
- `Enter`: Select option
- `Space`: Toggle item for deletion
- `a`: Select all SAFE items
- `c`: Cleanup selected items
- `q`: Back/Quit

### 📊 Results View
```
📊 Scan Results - 92.6 GB recoverable
─────────────────────────────────────────
✓ 🐳  64.0 GB │ 🟠 MED │ Docker Desktop disk image
✓ 📦  10.0 GB │ ✅ SAFE│ uv package cache
✓ 🐍   3.6 GB │ 🟠 MED│ Python 3.10.0 (current) with ML libs
✓ 📦   3.1 GB │ ✅ SAFE│ npm package cache
✓ 🤖   1.6 GB │ 🔴 REV│ HuggingFace model cache
✓ 🛠️   1.1 GB │ ✅ SAFE│ Node.js v18.20.5 is EOL
```

### 📈 History Tracking
Every scan saves results to `~/.local/share/prune/history.json`. View trends with:

```bash
prune --history
```

Output:
```
Disk Usage Trend (recoverable space):
  ▁▂▃▅▇█▇▅▃▂▁▂▃▅▆▇█▇▅▃▂▁

  First scan: 2026-06-14 15:43 — 92.6 GB
  Last scan:  2026-06-14 16:20 — 85.2 GB
  Change:   -7.4 GB (3 scans)
```

### 🎨 Gradient Colors (Explorer Mode)
When using explorer mode, bars show gradient colors based on size:
- **Green**: Small (< 30%)
- **Yellow/Orange**: Medium (30-70%)
- **Red**: Large (> 70%)

```bash
prune ~/Library
```

## 🚀 Installation

### From Source
```bash
git clone https://github.com/gabrielemastrapasqua/prune.git
cd prune
cargo build --release
./target/release/prune
```

### Add to PATH
```bash
# Option 1: Copy binary
sudo cp target/release/prune /usr/local/bin/

# Option 2: Add to ~/.zshrc or ~/.bashrc
export PATH="$HOME/source/personal/macdump/prune/target/release:$PATH"
```

## 📖 Usage

### Interactive TUI (Recommended)
```bash
# Launch TUI (default)
prune

# Or explicitly
prune -i
prune --interactive
```

### Advisor Mode (Text Output)
```bash
# Full scan
prune -a
prune --advisor

# Only safe operations
prune -a --risk safe

# Only AI models
prune -a --category ai

# Exclude dev tools
prune -a --no-dev

# JSON output
prune -a --json

# Custom minimum size
prune -a --min-size 1G
```

### Explorer Mode (Like dust)
```bash
# Analyze specific directory
prune ~/Library

# Show top 10 largest
prune ~/Library -n 10

# Find largest files
prune ~/ -F

# With depth limit
prune ~/Library -d 2
```

### History
```bash
# View scan history sparkline
prune --history
```

## 🎯 Risk Levels

Every recommendation has a risk level:

| Badge | Level | Meaning |
|-------|-------|---------|
| ✅ SAFE | Safe | No risk, can delete freely |
| 🟡 LOW | Low | Low risk, usually safe |
| 🟠 MEDIUM | Medium | Review before deleting |
| 🔴 REVIEW | Review | Manual review required |
| ⛔ DANGER | Danger | Never auto-delete |

## 🔧 Configuration

### Minimum Size
Default: 100 MB. Override with:
```bash
prune -a --min-size 500M
prune -a --min-size 1G
```

### Risk Limit
Default: `review` (shows all). Limit with:
```bash
prune -a --risk safe      # Only safe items
prune -a --risk low       # Safe + low risk
prune -a --risk medium    # Safe + low + medium
```

### Categories
Filter by category:
```bash
prune -a --category cache
prune -a --category dev
prune -a --category ai
prune -a --category installer
```

## 📊 What It Finds

### Example Output (92.6 GB on test system)

| Category | Size | What |
|----------|------|------|
| 🐳 Docker | 64 GB | Docker.raw disk image |
| 📦 uv cache | 10 GB | Package cache |
| 🐍 Python | 3.6 GB | pyenv versions + ML libs |
| 📦 npm cache | 3.1 GB | npm _cacache |
| 🤖 HuggingFace | 1.6 GB | Models/datasets |
| 🛠️ Node versions | 2.5 GB | 7 versions (3 EOL) |
| 🔧 Rust | 1.1 GB | Toolchains |
| 🛠️ uv tools | 1 GB | vllm-mlx |
| 📦 Homebrew | 827 MB | Download cache |
| 📦 Bun cache | 129 MB | Install cache |

### Version Manager Detection

**Node.js** (nvm, fnm, volta, asdf, mise):
- Detects all installed versions
- Identifies current default via `~/.nvm/alias/default`
- Marks EOL versions (Node < 18) as safe to remove
- Suggests `nvm uninstall <version>`

**Python** (pyenv, conda):
- Scans all pyenv versions
- Detects ML libraries (torch, transformers, etc.)
- Shows size breakdown per version
- Suggests `pyenv uninstall <version>`

**Rust** (rustup):
- Lists all toolchains
- Identifies default toolchain
- Suggests `rustup toolchain uninstall <name>`

### Package Manager Detection

Detects installed but unused package managers:
- **npm**: `~/.npm/_cacache`, `~/.npm/_npx`
- **pnpm**: `~/Library/pnpm/store`, global packages
- **yarn**: `~/Library/Caches/Yarn`, `~/.yarn/berry/cache`
- **bun**: `~/.bun/install/cache`
- **uv**: `~/.cache/uv`, `~/.local/share/uv/tools`
- **poetry**: `~/.cache/pypoetry`

### AI Model Detection

**HuggingFace Hub** (`~/.cache/huggingface/hub`):
- Individual model/dataset breakdown
- Shows size per model
- Detects unused models (> 90 days)

**Other AI Tools**:
- Ollama models (`~/.ollama/models`)
- LM Studio (`~/.lmstudio/models`)
- Whisper (`~/.cache/whisper`)
- PyTorch cache (`~/.cache/torch`)
- CUDA toolkit (`/usr/local/cuda*`)

## 🛡️ Safety Features

### Safe Deletion
- **Whitelist**: Only suggests known cache/temp directories
- **Blacklist**: Never touches `/System`, `/usr`, `.git/objects`
- **Current versions**: Never suggests deleting current Node/Python/Rust default
- **Confirmation**: Always asks before executing deletions
- **Native commands**: Uses `nvm uninstall`, `pyenv uninstall`, etc.

### Risk Assessment
- **EOL detection**: Node 16, 18 marked as EOL
- **Last accessed**: Shows days since last use
- **Size thresholds**: Configurable minimum size
- **Category filtering**: Skip dev/AI if not needed

## 🔍 How It Works

### Parallel Scanning
Uses **rayon** for parallel directory traversal:
- Scans multiple directories simultaneously
- Utilizes all CPU cores (M1: 8 cores)
- Typical scan time: 13-15 seconds for full ~/

### Known Paths Database
Maintains a comprehensive list of macOS cache locations:
- `~/Library/Caches/*`
- `~/.npm`, `~/.cache/*`
- `~/Library/Developer/Xcode/*`
- And 50+ more locations

### Rule-Based Analysis
Evaluates each path against rules:
- Size thresholds
- Age (last accessed)
- Category (cache, dev, AI, etc.)
- Risk level (safe, low, medium, review)

### Heuristic Analysis
Detects patterns:
- Old large files in Movies/Desktop
- Unused version manager installations
- Heavy ML libraries in site-packages
- Dead simulator containers

## 📝 Examples

### Clean up old Node versions
```bash
# See what's installed
prune -a --category dev | grep Node

# Output shows:
# Node.js v16.13.2 is EOL (End of Life)
# Node.js v18.15.0 is EOL (End of Life)
# Node.js v20.20.2 (current default)

# In TUI, select EOL versions and press 'c' to cleanup
# Executes: nvm uninstall 16.13.2
```

### Remove unused Python ML libraries
```bash
# Scan for AI models
prune -a --category ai

# Shows:
# Python 3.10.0 with ML libs: torch (306.8 MB), transformers (85.3 MB)
# HuggingFace dataset 'Aniemore/resd': 463.0 MB

# Review and selectively delete
```

### Clean Docker
```bash
# Check Docker size
prune -a | grep Docker

# Output:
# Docker Desktop disk image: 64.0 GB

# Suggested command:
# docker system prune -af --volumes
```

### Quick cache cleanup
```bash
# TUI mode
prune

# Select "Full Scan"
# Press 'a' to select all SAFE items
# Press 'c' to cleanup
# Confirm with 'y'
```

## 🧪 Development

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
```

### Run without install
```bash
cargo run --release
cargo run --release -- -a
cargo run --release -- ~/Library
```

## 🤝 Contributing

Contributions welcome! Areas for improvement:
- [ ] Linux support (currently macOS-focused)
- [ ] More package manager detection
- [ ] Duplicate file detection
- [ ] Snapshot/Time Machine cleanup
- [ ] Custom rules engine
- [ ] Plugin system

## 📄 License

Apache-2.0

## 🙏 Acknowledgments

- Fork of [dust](https://github.com/bootandy/dust) by bootandy
- Built with [ratatui](https://github.com/ratatui/ratatui) for TUI
- Uses [rayon](https://github.com/rayon-rs/rayon) for parallelism

## 🐛 Known Issues

- Gradient colors in explorer mode may not render correctly in all terminals
- History tracking requires write access to `~/.local/share/prune/`
- Some operations require manual command execution (shown in output)

## 🔮 Future Plans

- [ ] Gradient colors for TUI bars
- [ ] Interactive file browser
- [ ] Scheduled cleanup reminders
- [ ] Export reports (PDF, HTML)
- [ ] Cloud storage integration
- [ ] Cross-platform support (Linux, Windows)

---

**Made with ❤️ for macOS developers who hate wasted disk space.**
