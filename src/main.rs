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

    let status = read_sysfs(format!("{}/{}", path, "status")).unwrap_or_else(|| "Unknown".to_string());

    let capacity: Option<u32> = read_sysfs(format!("{}/{}", path, "capacity"))
        .and_then(|s| s.parse().ok());

    let cycle_count: Option<u32> = read_sysfs(format!("{}/{}", path, "cycle_count"))
        .and_then(|s| s.parse().ok());

    let power_now: Option<f32>= read_sysfs(format!("{}/{}", path, "power_now"))
        .and_then(|s| s.parse::<f32>().ok())
        .map(|p| p * 1e-6);

    let energy_now: Option<f32> = read_sysfs(format!("{}/{}", path, "energy_now"))
        .and_then(|s| s.parse::<f32>().ok())
        .map(|p| p * 1e-6);

    let energy_full: Option<f32> = read_sysfs(format!("{}/{}", path, "energy_full"))
        .and_then(|s| s.parse::<f32>().ok())
        .map(|p| p * 1e-6);

    let energy_full_design: Option<f32> = read_sysfs(format!("{}/{}", path, "energy_full_design"))
        .and_then(|s| s.parse::<f32>().ok())
        .map(|p| p * 1e-6);

    let technology: Option<String>=  read_sysfs(format!("{}/{}", path, "technology"));


    Some(BatteryInfo {name , status,capacity, cycle_count, power_now, energy_now, energy_full, energy_full_design, technology})
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

fn calc_time(info: &BatteryInfo) -> Option<(u32,u32)>{
    let power = info.power_now?;
    let energy_now = info.energy_now?;

    let energy = if info.status == "Charging"{
        info.energy_full? - energy_now
    } else {
        energy_now
    };

    Some(calc_remaining(energy, power))
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
        let battery_info = get_battery_info(&path).unwrap();

        let charging_symbol = match battery_info.status.as_str() {
            "Charging" => "↑".yellow(),
            "Discharging" | "Not charging" => "↓".cyan(),
            "Full" => "→".green(),
            _ => "?".white(),
        };

        let time = calc_time(&battery_info).map(|(h,m)| format!("{:2}h{:2}m", h, m)).unwrap_or("--:--".to_string());

        let bar = battery_info.capacity.map(|n| progress_bar(n, 10)).unwrap_or("None".to_string().white());

        println!(
            "{} {} {} {} {} {}",
            battery_info.name, bar, battery_info.capacity.map(|n| format!("{}%",n)).unwrap_or("Unknown".to_string()), battery_info.power_now.map(|n| format!("{:.1}W",n)).unwrap_or("Unknown".to_string()), charging_symbol, time
        );
    }
}
