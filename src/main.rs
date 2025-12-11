use std::fs;
use std::path::Path;

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
            if let Ok(t) = fs::read_to_string(&type_path) {
                if t.trim() == "Battery" {
                    batteries.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    batteries
}

fn progress_bar(percent: u32, width: u32) -> String {
    let filled = (percent * width / 100) as usize;
    let empty = (width as usize) - filled;

    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn main() {
    let battery_paths = find_batteries();

    if battery_paths.is_empty() {
        println!("No battery found");
        return;
    }

    for path in battery_paths {
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
            "Charging" => "↑",
            "Discharging" => "↓",
            _ => "?",
        };
        let power_now: f32 = read_sysfs(format!("{}/{}", path, "power_now"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0f32);
        let power_watt = power_now / 1e6;
        let energy_now: f32 = read_sysfs(format!("{}/{}", path, "energy_now"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        // Avoid zero division
        let (h, m) = if power_now > 0.0 {
            let remaining_hours = energy_now / power_now;
            (
                remaining_hours as u32,
                (remaining_hours.fract() * 60.0) as u32,
            )
        } else {
            (0, 0)
        };

        let bar = progress_bar(capacity, 10);

        println!(
            "{} {} {:3}% {} {:2.1}W {:}h{:2}m",
            battery_name, bar, capacity, charging_symbol, power_watt, h, m
        );
    }
}
