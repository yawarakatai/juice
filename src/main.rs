use std::fs;
use std::path::Path;

fn read_sysfs(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn main() {
    let capacity = read_sysfs("/sys/class/power_supply/BAT0/capacity");
    let status = read_sysfs("/sys/class/power_supply/BAT0/status");
    let power_now = read_sysfs("/sys/class/power_supply/BAT0/power_now");

    println!("Capacity: {:?}", capacity);
    println!("Status: {:?}", status);
    println!("Power: {:?}", power_now);
}
