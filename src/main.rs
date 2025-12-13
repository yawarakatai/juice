mod db;

use clap::{Parser, Subcommand};
use colored::*;
use db::{default_db_path, Database};
use std::fs;
use std::path::Path;

const MICRO: f32 = 1e-6;
const PICO: f32 = 1e-12;

const POWER_SUPPLY_PATH: &str = "/sys/class/power_supply";

#[derive(Parser)]
#[command(name = "juice")]
#[command(about = "Battery status and history for Linux")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    // Show detailed information
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Daemon {
        #[arg(short, long, default_value = "30")]
        interval: u64,
    },
    Status,
}

struct BatteryInfo {
    name: String,
    status: String,
    capacity: Option<u32>,
    cycle_count: Option<u32>,
    power_now: Option<f32>,
    energy_now: Option<f32>,
    energy_full: Option<f32>,
    energy_full_design: Option<f32>,
    technology: Option<String>,
}

impl BatteryInfo {
    fn bar(&self) -> ColoredString {
        self.capacity
            .map(|n| progress_bar(n, 10))
            .unwrap_or("None".to_string().white())
    }

    fn capacity_str(&self) -> String {
        self.capacity
            .map(|n| format!("{:3}%", n))
            .unwrap_or_else(|| "  --%".to_string())
    }

    fn power_str(&self) -> String {
        self.power_now
            .map(|n| format!("{:5.1}W", n))
            .unwrap_or_else(|| "  --W".to_string())
    }

    fn calc_time(&self) -> Option<(u32, u32)> {
        let power = self.power_now?;
        if power <= 0.0 {
            return None;
        }

        let energy_now = self.energy_now?;

        let energy = if self.status == "Charging" {
            self.energy_full? - energy_now
        } else {
            energy_now
        };

        let hours = energy / power;
        let minutes = hours.fract() * 60.0;
        Some((hours as u32, minutes as u32))
    }

    fn remaining_str(&self) -> String {
        self.calc_time()
            .map(|(h, m)| format!("{:2}h{:02}m", h, m))
            .unwrap_or(" --:--".to_string())
    }
}

fn read_sysfs(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path.as_ref())
        .ok()
        .map(|s| s.trim().to_string())
}

fn find_batteries() -> Vec<String> {
    let power_supply = Path::new(POWER_SUPPLY_PATH);

    let entries = match fs::read_dir(power_supply) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Warning: {}:{}", power_supply.display(), e);
            return Vec::new();
        }
    };

    entries
        .flatten()
        .filter(|entry| {
            fs::read_to_string(entry.path().join("type"))
                .map(|t| t.trim() == "Battery")
                .unwrap_or(false)
        })
        .map(|entry| entry.path().to_string_lossy().to_string())
        .collect()
}

// power_supply class has several types of expressions
// For example:
// power_now / current_now
// energy_now / charge_now
// energy_full / charge_full
// energy_full_design / charge_full_design

fn read_power(path: &str) -> Option<f32> {
    if let Some(power) =
        read_sysfs(format!("{}/power_now", path)).and_then(|s| s.parse::<f32>().ok())
    {
        return Some(power * MICRO);
    }

    let current =
        read_sysfs(format!("{}/current_now", path)).and_then(|s| s.parse::<f32>().ok())?;
    let voltage =
        read_sysfs(format!("{}/voltage_now", path)).and_then(|s| s.parse::<f32>().ok())?;

    Some(current * voltage * PICO)
}

fn read_energy_or_charge(path: &str, class_name: &str) -> Option<f32> {
    read_sysfs(format!("{}/energy_{}", path, class_name))
        .or_else(|| read_sysfs(format!("{}/charge_{}", path, class_name)))
        .and_then(|s| s.parse::<f32>().ok())
        .map(|p| p * MICRO)
}

fn get_battery_info(path: &str) -> BatteryInfo {
    let name: String = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let status = read_sysfs(format!("{}/status", path)).unwrap_or_else(|| "Unknown".to_string());

    let capacity: Option<u32> =
        read_sysfs(format!("{}/capacity", path)).and_then(|s| s.parse().ok());

    let cycle_count: Option<u32> =
        read_sysfs(format!("{}/cycle_count", path)).and_then(|s| s.parse().ok());

    let power_now: Option<f32> = read_power(path);

    let energy_now: Option<f32> = read_energy_or_charge(path, "now");
    let energy_full: Option<f32> = read_energy_or_charge(path, "full");
    let energy_full_design: Option<f32> = read_energy_or_charge(path, "full_design");

    let technology: Option<String> = read_sysfs(format!("{}/technology", path));

    BatteryInfo {
        name,
        status,
        capacity,
        cycle_count,
        power_now,
        energy_now,
        energy_full,
        energy_full_design,
        technology,
    }
}

fn progress_bar(percent: u32, width: u32) -> ColoredString {
    let filled = (percent * width / 100) as usize;
    let empty = (width as usize) - filled;

    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    match percent {
        0..=20 => bar.red(),
        21..=50 => bar.yellow(),
        _ => bar.green(),
    }
}

fn calc_health(info: &BatteryInfo) -> Option<f32> {
    let current_full = info.energy_full?;
    let design_full = info.energy_full_design?;

    Some(current_full / design_full * 100.0)
}

fn print_normal(info: &BatteryInfo) {
    let charging_symbol = match info.status.as_str() {
        "Charging" => "↑".yellow(),
        "Discharging" | "Not charging" => "↓".cyan(),
        "Full" => "→".green(),
        _ => "?".white(),
    };

    println!(
        "{} {} {} {} {} {}",
        info.name,
        info.bar(),
        info.capacity_str(),
        info.power_str(),
        charging_symbol,
        info.remaining_str(),
    );
}

fn print_verbose(info: &BatteryInfo) {
    let bar = info
        .capacity
        .map(|n| progress_bar(n, 10))
        .unwrap_or("None".to_string().white());

    let energy_str = info
        .energy_now
        .zip(info.energy_full)
        .map(|(now, full)| format!("{:5.1} / {:5.1} Wh", now, full))
        .unwrap_or_else(|| " -- /  -- Wh".to_string());

    let cycle_count_str = info
        .cycle_count
        .map(|n| n.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let health_str = calc_health(info)
        .map(|n| format!("{:5.1}%", n))
        .unwrap_or_else(|| " --%".to_string());

    println!(
        "{} {} {} {}",
        info.name,
        bar,
        info.capacity_str(),
        info.status
    );
    println!("  Power:       {}", info.power_str());
    println!("  Remaining:   {}", info.remaining_str());
    println!("  Energy:      {}", energy_str);
    println!("  Cycle count: {}", cycle_count_str);
    println!("  Health:      {}", health_str);
    println!(
        "  Technology:  {}",
        info.technology.as_deref().unwrap_or("Unknown")
    );
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let battery_paths = find_batteries();

            if battery_paths.is_empty() {
                println!("No battery found");
                return;
            }

            for path in battery_paths {
                let battery_info = get_battery_info(&path);
                if cli.verbose {
                    print_verbose(&battery_info);
                } else {
                    print_normal(&battery_info);
                }
            }
        }
        Some(Commands::Daemon { interval }) => {
            println!("Starting daemon with {}s interval...", interval);
        }
        Some(Commands::Status) => {
            let db_path = default_db_path();
            println!("Database: {}", db_path.display());

            match Database::open(&db_path) {
                Ok(db) => {
                    db.init_scheme().expect("Failed to init schema");
                    let count = db.count_readings().unwrap_or(0);
                    println!("Total readings: {}", count);
                }
                Err(e) => {
                    println!("Database error: {}", e);
                }
            }
        }
    }
}
