# WinDebloat for Linux

An advanced Linux debloating and cleaning tool with a beautiful TUI (Terminal User Interface) built with Rust and Ratatui.

## Features

- **Multi-distro support**: Debian/Ubuntu, Arch/Manjaro, Fedora/RHEL, openSUSE, Alpine
- **7+ cleaning modules**:
  - 📦 Packages: Cache, orphaned packages, old kernels
  - 🧹 System: Logs, journal, tmp, trash, caches, broken symlinks
  - 💻 Applications: Browser caches, Flatpak, Snap, Docker, dev tools
  - 🔒 Privacy: Shell history, recent files, clipboard data
  - ⚙ Services: Disable unnecessary services (bluetooth, cups, etc.)
  - 🗑 Duplicates: Find and remove duplicate files (SHA-256)
  - 💾 Disk Usage: Find large files consuming disk space
- **Beautiful TUI**: Panel-based interface with sidebar and content areas
- **Full rollback system**: Every clean operation is backed up for easy undo
- **Safety first**: Safe mode, dry-run, confirmations, whitelist/blacklist
- **Configurable**: TOML-based configuration with defaults
- **Multi-language**: English interface

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd WinDebloat-ForLinux

# Build
cargo build --release

# Install (optional)
sudo cp target/release/windebloat /usr/local/bin/
```

## Usage

### Interactive TUI (Recommended)
```bash
windebloat
```

### CLI Commands
```bash
# Show system information
windebloat status

# Scan for cleanable items (dry-run)
windebloat scan --all
windebloat scan --system

# Clean selected items
windebloat clean --all
windebloat clean --system --yes

# Manage backups
windebloat undo list
windebloat undo --id <backup-id>

# Configuration
windebloat config show
windebloat config edit
```

## Configuration

Default configuration is stored at `~/.config/windebloat/config.toml`:

```toml
[safe]
enabled = true
confirm_all = true

[whitelist]
paths = []

[blacklist]
paths = []

[modules]
packages = true
system = true
apps = true
privacy = false
services = false
duplicates = false

[backup]
enabled = true
location = "~/.local/share/windebloat/backups"
retention_days = 30
max_backups = 10

[ui]
theme = "auto"
sidebar_width = 20
show_icons = true
```

## Keyboard Shortcuts (TUI)

**Navigation:**
- `↑`/`↓`: Navigate items
- `←`/`→`: Switch categories
- `PageUp`/`PageDown`: Scroll by page
- `Home`/`End`: Jump to first/last item

**Selection:**
- `Space`: Toggle item selection
- `Enter`: Toggle selection (alternative)
- `T`: Toggle select all
- `R`: Select safe items only

**Actions:**
- `Tab`: Toggle focus (list/details)
- `A`: Analyze/scan current category
- `S`: Scan all categories
- `C`: Clean selected items
- `U`: Undo / view backups
- `/`: Search/filter items

**Other:**
- `W`: Disk Cleaning Wizard
- `Q`/`Esc`: Quit
- `?`/`H`: Toggle help

## Safety Features

- **Dry-run mode**: Always see what will be cleaned before proceeding
- **Full backup**: Every file is backed up before removal
- **Rollback**: Use `windebloat undo` to restore from backups
- **Whitelist/Blacklist**: Control what gets cleaned
- **Safe mode**: Only clean known-safe items by default
- **Confirmations**: Interactive confirmation before cleaning

## License

MIT

## Credits

Built with:
- [Ratatui](https://github.com/ratatui/ratatui) - TUI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Terminal handling
- [Clap](https://github.com/clap-rs/clap) - Argument parsing
- [Serde](https://serde.rs/) - Configuration serialization
