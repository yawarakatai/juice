use colored::*;
use std::fs;
use std::path::Path;
use clap::Parser;

#[derive(Parser)]
#[command(name = "juice")]
#[command(about = "Battery status for Linux")]
struct Args {
    // Show detailed information
    #[arg(short, long)]   
    verbose: bool,
}

struct BatteryInfo {
    name: String,
    capacity: u32,
    status: String,
    power_now: f32,
    energy_now: f32,
    energy_full: f32,
}

fn read_sysfs(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path.as_ref())
        .ok()
        .map(|s| s.trim().to_string())
}

fn find_batteries() -> Vec<String> {
    let mut batteries = Vec::new();
    let power_supply = Path::new("/sys/class/power_supply");

    if let Ok(entries) = fs::read_dir(power_supply) {
        for entry in entries.flatten() {
            let type_path = entry.path().join("type");
            if let Ok(t) = fs::read_to_string(&type_path) && t.trim() == "Battery" {
                batteries.push(entry.path().to_string_lossy().to_string());
            }
        }
    }
    batteries
}

fn get_battery_info(path: &str) -> Option<BatteryInfo>{
    let name: String = Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let capacity: u32 = read_sysfs(format!("{}/{}", path, "capacity"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let status =
        read_sysfs(format!("{}/{}", path, "status")).unwrap_or_else(|| "Unknown".to_string());

    let power_now: f32 = read_sysfs(format!("{}/{}", path, "power_now"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)* 1e-6;

    let energy_now: f32 = read_sysfs(format!("{}/{}", path, "energy_now"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)* 1e-6;

    let energy_full: f32 = read_sysfs(format!("{}/{}", path, "energy_full"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)* 1e-6;

    Some(BatteryInfo {name , capacity, status, power_now, energy_now, energy_full})
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

fn calc_remaining(energy: f32, power: f32) -> (u32, u32) {
    if power <= 0.0 {
        return (0, 0);
    }
    let hours = energy / power;
    let minutes = hours.fract() * 60.0;
    (hours as u32, minutes as u32)
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        todo!();
    }

    let battery_paths = find_batteries();

    if battery_paths.is_empty() {
        println!("No battery found");
        return;
    }

    for path in battery_paths {
        let battery_info = get_battery_info(&path);
        let battery_name: &str = Path::new(&path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let capacity: u32 = read_sysfs(format!("{}/{}", path, "capacity"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let status =
            read_sysfs(format!("{}/{}", path, "status")).unwrap_or_else(|| "Unknown".to_string());
        let charging_symbol = match status.as_str() {
            "Charging" => "↑".yellow(),
            "Discharging" => "↓".cyan(),
            _ => "?".red(),
        };
        let power_now: f32 = read_sysfs(format!("{}/{}", path, "power_now"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let power_watt = power_now / 1e6;
        let energy_now: f32 = read_sysfs(format!("{}/{}", path, "energy_now"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let energy_full: f32 = read_sysfs(format!("{}/{}", path, "energy_full"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let (h, m) = if status == "Charging" {
            calc_remaining(energy_full - energy_now, power_now) // Time to fully charge
        } else {
            calc_remaining(energy_now, power_now) // Time to complete discharge
        };

        let time = format!("{:2}h{:2}m", h, m);

        let bar = progress_bar(capacity, 10);

        println!(
            "{} {} {:3}% {:2.1}W {} {}",
            battery_name, bar, capacity, power_watt, charging_symbol, time
        );
    }
}
