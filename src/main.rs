mod battery;
mod daemon;
mod db;

use battery::{
    calc_health, find_batteries, get_battery_info, progress_bar, BatteryInfo, BatteryStatus,
};
use clap::{Parser, Subcommand};
use colored::*;
use db::{default_db_path, Database};
use std::error::Error;

#[derive(Parser)]
#[command(version, about)]
/// Battery status and history for Linux
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show detailed information
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start daemon for recording battery info frequently
    Daemon {
        /// Interval in seconds
        #[arg(short, long, default_value = "30")]
        interval: u64,
    },
    // Show status about daemon and stored data
    Status,
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

        let energy = if self.status == BatteryStatus::Charging {
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

fn print_normal(info: &BatteryInfo) {
    let charging_symbol = match info.status {
        BatteryStatus::Charging => "↑".yellow(),
        BatteryStatus::Discharging | BatteryStatus::NotCharging => "↓".cyan(),
        BatteryStatus::Full => "→".green(),
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

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let battery_paths = find_batteries();

            if battery_paths.is_empty() {
                println!("No battery found");
                return Ok(());
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
            let db_path = default_db_path();
            println!("Starting daemon with {}s interval...", interval);
            daemon::run(db_path, interval)?;
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

    Ok(())
}
