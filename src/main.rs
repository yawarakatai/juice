use clap::Parser;
use colored::*;
use std::fs;
use std::path::Path;

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
    status: String,
    capacity: Option<u32>,
    cycle_count: Option<u32>,
    power_now: Option<f32>,
    energy_now: Option<f32>,
    energy_full: Option<f32>,
    energy_full_design: Option<f32>,
    technology: Option<String>,
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

            // This code can be shorter with if, but it obstructs rustfmt
            match fs::read_to_string(&type_path) {
                Ok(t) if t.trim() == "Battery" => {
                    batteries.push(entry.path().to_string_lossy().to_string());
                }
                _ => {}
            }
        }
    }
    batteries
}

fn get_battery_info(path: &str) -> BatteryInfo {
    let name: String = Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let status =
        read_sysfs(format!("{}/{}", path, "status")).unwrap_or_else(|| "Unknown".to_string());

    let capacity: Option<u32> =
        read_sysfs(format!("{}/{}", path, "capacity")).and_then(|s| s.parse().ok());

    let cycle_count: Option<u32> =
        read_sysfs(format!("{}/{}", path, "cycle_count")).and_then(|s| s.parse().ok());

    let power_now: Option<f32> = read_sysfs(format!("{}/power_now", path))
        .and_then(|s| s.parse::<f32>().ok())
        .map(|p| p * 1e-6);

    let energy_now: Option<f32> = read_sysfs(format!("{}/energy_now", path))
        .and_then(|s| s.parse::<f32>().ok())
        .map(|p| p * 1e-6);

    let energy_full: Option<f32> = read_sysfs(format!("{}/energy_full", path))
        .and_then(|s| s.parse::<f32>().ok())
        .map(|p| p * 1e-6);

    let energy_full_design: Option<f32> = read_sysfs(format!("{}/energy_full_design", path))
        .and_then(|s| s.parse::<f32>().ok())
        .map(|p| p * 1e-6);

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

fn calc_remaining(energy: f32, power: f32) -> (u32, u32) {
    if power <= 0.0 {
        return (0, 0);
    }
    let hours = energy / power;
    let minutes = hours.fract() * 60.0;
    (hours as u32, minutes as u32)
}

fn calc_time(info: &BatteryInfo) -> Option<(u32, u32)> {
    let power = info.power_now?;
    let energy_now = info.energy_now?;

    let energy = if info.status == "Charging" {
        info.energy_full? - energy_now
    } else {
        energy_now
    };

    Some(calc_remaining(energy, power))
}

fn calc_health(info: &BatteryInfo) -> Option<f32> {
    let current_full = info.energy_full?;
    let design_full = info.energy_full_design?;

    Some(current_full / design_full * 100.0)
}

fn print_normal(info: &BatteryInfo) {
    let bar = info
        .capacity
        .map(|n| progress_bar(n, 10))
        .unwrap_or("None".to_string().white());

    let capacity_str = info
        .capacity
        .map(|n| format!("{:3}%", n))
        .unwrap_or_else(|| "  --%".to_string());

    let power_str = info
        .power_now
        .map(|n| format!("{:5.1}W", n))
        .unwrap_or_else(|| "  --W".to_string());

    let charging_symbol = match info.status.as_str() {
        "Charging" => "↑".yellow(),
        "Discharging" | "Not charging" => "↓".cyan(),
        "Full" => "→".green(),
        _ => "?".white(),
    };

    let remaining_time_str = calc_time(info)
        .map(|(h, m)| format!("{:2}h{:02}m", h, m))
        .unwrap_or(" --:--".to_string());

    println!(
        "{} {} {} {} {} {}",
        info.name, bar, capacity_str, power_str, charging_symbol, remaining_time_str
    );
}

fn print_verbose(info: &BatteryInfo) {
    let bar = info
        .capacity
        .map(|n| progress_bar(n, 10))
        .unwrap_or("None".to_string().white());

    let capacity_str = info
        .capacity
        .map(|n| format!("{:3}%", n))
        .unwrap_or_else(|| "  --%".to_string());

    let power_str = info
        .power_now
        .map(|n| format!("{:5.1}W", n))
        .unwrap_or_else(|| "  --W".to_string());

    let remaining_time_str = calc_time(info)
        .map(|(h, m)| format!("{:2}h{:02}m", h, m))
        .unwrap_or(" --:--".to_string());

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
        .map(|n| format!("{:3}%", n))
        .unwrap_or_else(|| " --%".to_string());

    println!("{} {} {} {}", info.name, bar, capacity_str, info.status);
    println!("  Power:       {}", power_str);
    println!("  Remaining:   {}", remaining_time_str);
    println!("  Energy:      {}", energy_str);
    println!("  Cycle count: {}", cycle_count_str);
    println!("  Health:      {}", health_str);
    println!(
        "  Technology:  {}",
        info.technology.as_deref().unwrap_or("Unknown")
    );
}

fn main() {
    let args = Args::parse();
    let battery_paths = find_batteries();

    if battery_paths.is_empty() {
        println!("No battery found");
        return;
    }

    for path in battery_paths {
        let battery_info = get_battery_info(&path);
        if args.verbose {
            print_verbose(&battery_info);
        } else {
            print_normal(&battery_info);
        }
    }
}
