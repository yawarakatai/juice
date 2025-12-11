use std::fs;
use std::path::Path;

fn read_sysfs(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
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

fn main() {
    let batteries = find_batteries();
    println!("{:?}", batteries);

    let capacity = read_sysfs("/sys/class/power_supply/BAT0/capacity");
    let status = read_sysfs("/sys/class/power_supply/BAT0/status");
    let power_now = read_sysfs("/sys/class/power_supply/BAT0/power_now");

    println!("Capacity: {:?}", capacity);
    println!("Status: {:?}", status);
    println!("Power: {:?}", power_now);
}
