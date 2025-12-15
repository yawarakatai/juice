# 🧃 juice - A modern battery status CLI for Linux.

## 🚀 Usage
```bash
juice        # Simple output
juice -v     # Detailed output

# Daemon (background data collection)
juice daemon              # Start with 30s interval
juice daemon -i 60        # Custom interval

# Status
juice status              # Show database info

# Export
juice export              # CSV to stdout
juice export -o data.csv  # Save to file
juice export --from 2025-12-13 --to 2025-12-14
```

## 📸 Example Output

### Normal mode
```
BAT0   ██████░░░░  63% ↓  5.1W  5h55m
```

### Verbose mode (`-v`)
```
BAT0 ████████████░░░░░░░░ Discharging
  Power:       5.5 W
  Remaining:   5h27m
  Capacity:    63 %
  Energy:      30.1 / 48.0 Wh
  Cycle count: 66
  Health:      96.8 %
  Technology:  Li-poly
```

## 📦 Installation

### From crates.io
```bash
cargo install juice-cli
```

### From source
```bash
git clone https://github.com/yawarakatai/juice
cd juice
cargo build --release
```

### Nix
```bash
nix run github:yawarakatai/juice
```

### Systemd service
Create a service file to ~/.config/systemd/user/juice-daemon.service

```juice.service
[Unit]
Description=Juice battery history daemon
Documentation=https://github.com/yawarakatai/juice
After=default.target

[Service]
Type=simple
ExecStart=%h/.cargo/bin/juice daemon
Restart=on-failure
RestartSec=30

[Install]
WantedBy=default.target
```

and then, enable it

```bash
systemctl --user daemon-reload
systemctl --user enable juice-daemon
systemctl --user start juice-daemon
```

## ✨ Features

- Simple, clean output with progress bar and colors
- Multiple battery support (ThinkPad, etc.)
- Detailed view with battery health, cycle count, and more
- Background daemon for continuous battery history logging
- SQLite database storage with CSV export

## 🔧 Compatibility

juice reads battery information from `/sys/class/power_supply/` and supports:

- Standard `energy_*` interfaces (most laptops)
- `charge_*` interfaces (some older hardware)
- `current_now` + `voltage_now` fallback for power calculation
- Multiple batteries (BAT0, BAT1, CMB0, etc.)

Tested on:

- NixOS
- Arch Linux

## 📄 License

MIT
