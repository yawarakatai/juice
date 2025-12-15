use core::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::{fs, io};

const MICRO: f32 = 1e-6;
const PICO: f32 = 1e-12;

const POWER_SUPPLY_PATH: &str = "/sys/class/power_supply";

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

impl fmt::Display for BatteryStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            BatteryStatus::Charging => "Charging",
            BatteryStatus::Discharging => "Discharging",
            BatteryStatus::Full => "Full",
            BatteryStatus::NotCharging => "Not charging",
            BatteryStatus::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for BatteryStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Charging" => Ok(BatteryStatus::Charging),
            "Discharging" => Ok(BatteryStatus::Discharging),
            "Full" => Ok(BatteryStatus::Full),
            "Not charging" => Ok(BatteryStatus::NotCharging),
            _ => Ok(Self::Unknown),
        }
    }
}

pub struct BatteryInfo {
    pub name: String,
    pub status: BatteryStatus,
    pub capacity: Option<u32>,
    pub cycle_count: Option<u32>,
    pub power_now: Option<f32>,
    pub energy_now: Option<f32>,
    pub energy_full: Option<f32>,
    pub energy_full_design: Option<f32>,
    pub technology: Option<String>,
}

impl BatteryInfo {
    pub fn calc_health(&self) -> Option<f32> {
        let current_full = self.energy_full?;
        let design_full = self.energy_full_design?;

        Some(current_full / design_full * 100.0)
    }

    pub fn calc_remaining_time(&self) -> Option<(u32, u32)> {
        let power = self.power_now?;
        if power <= 0.0 {
            return None;
        }

        let energy_now = self.energy_now?;

        let energy = if self.status == BatteryStatus::Charging {
            self.energy_full? - energy_now
        } else {
            energy_now
        };

        let hours = energy / power;
        let minutes = hours.fract() * 60.0;
        Some((hours as u32, minutes as u32))
    }
}

fn read_sysfs(path: impl AsRef<Path>) -> io::Result<String> {
    let file = fs::read_to_string(path.as_ref())?;
    Ok(file.trim().to_string())
}

pub fn find_batteries() -> Vec<PathBuf> {
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
        .map(|entry| entry.path())
        .collect()
}

// power_supply class has several types of expressions
// For example:
// power_now / current_now
// energy_now / charge_now
// energy_full / charge_full
// energy_full_design / charge_full_design

fn read_power(path: &Path) -> Option<f32> {
    if let Some(power) = read_sysfs(path.join("power_now"))
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
    {
        return Some(power * MICRO);
    }

    let current = read_sysfs(path.join("current"))
        .ok()
        .and_then(|s| s.parse::<f32>().ok())?;
    let voltage = read_sysfs(path.join("voltage"))
        .ok()
        .and_then(|s| s.parse::<f32>().ok())?;

    Some(current * voltage * PICO)
}

fn read_energy_or_charge(path: &Path, class_name: &str) -> Option<f32> {
    read_sysfs(path.join(format!("energy_{}", class_name)))
        .ok()
        .or_else(|| read_sysfs(path.join(format!("charge_{}", class_name))).ok())
        .and_then(|s| s.parse::<f32>().ok())
        .map(|p| p * MICRO)
}

pub fn get_battery_info(path: &Path) -> BatteryInfo {
    let name: String = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let status = read_sysfs(path.join("status"))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(BatteryStatus::Unknown);

    let capacity: Option<u32> = read_sysfs(path.join("capacity"))
        .ok()
        .and_then(|s| s.parse().ok());

    let cycle_count: Option<u32> = read_sysfs(path.join("cycle_count"))
        .ok()
        .and_then(|s| s.parse().ok());

    let power_now: Option<f32> = read_power(path);

    let energy_now: Option<f32> = read_energy_or_charge(path, "now");
    let energy_full: Option<f32> = read_energy_or_charge(path, "full");
    let energy_full_design: Option<f32> = read_energy_or_charge(path, "full_design");

    let technology: Option<String> = read_sysfs(path.join("technology")).ok();

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
