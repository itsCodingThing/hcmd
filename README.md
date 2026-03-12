# hcmd

Enhanced terminal commands.

## About

hcmd is a Rust-based CLI tool that provides enhanced terminal utilities including:

- **Pomodoro Timer** - Create, manage, and run customizable timer sessions
- **File Discovery (fd)** - Interactive TUI-based fuzzy file finder with preview

## Commands

### Pomodoro Timer

```bash
# Create a quick timer (duration in minutes)
hcmd pomo create --name "Focus" --duration 25

# Add a timer to storage
hcmd pomo add --name "Break" --duration 5

# Run a stored timer by name
hcmd pomo run --name "Focus"

# List all stored timers
hcmd pomo list

# Remove a stored timer by name
hcmd pomo remove --name "Focus"

# Remove all stored timers
hcmd pomo nuke
```

### File Discovery (fd)

```bash
# Launch interactive file browser
hcmd fd
```

**Keybindings in fd:**
- `j` / `k` - Navigate up/down
- `Enter` - Toggle selection
- `a` - Create new file/directory
- `r` - Rename
- `d` - Delete
- `q` - Quit

## Build

```bash
# Build the project
cargo build

# Run in development
cargo run

# Build release version
cargo build --release

# Install using cargo (requires Rust installed)
cargo install --git https://github.com/yourusername/hcmd.git
```
