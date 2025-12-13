# 🧃 juice - A modern battery status CLI for Linux.

## 🚀 Usage
```bash
juice        # Simple output
juice -v     # Detailed output
juice --help # Show help
```

## 📸 Example Output

### Normal mode
```
BAT0 ████████░░  84%  11.2W ↓  2h34m
```

### With multiple batteries
```
BAT0 ████████░░  84%   8.2W ↓  3h12m
BAT1 ██████░░░░  62%   3.0W ↓  2h58m
```

### Verbose mode (`-v`)
```
BAT0 ████████░░  84% Discharging
  Power:        11.2W
  Remaining:    2h34m
  Energy:       45.2 /  54.0 Wh
  Cycle count:  142
  Health:       92.3%
  Technology:   Li-ion
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

Or add to your flake inputs:
```nix
{
  inputs.juice.url = "github:yawarakatai/juice";
}
```

## ✨ Features

- Simple, clean output with progress bar and colors
- Multiple battery support (ThinkPad, etc.)
- Detailed view with battery health, cycle count, and more
- No dependencies other than sysfs (works on minimal systems)

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
