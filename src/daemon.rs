use crate::battery::{find_batteries, get_battery_info};
use crate::db::Database;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::signal;
use tokio::time::Duration;

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub async fn run(db_path: PathBuf, interval_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
    let battery_paths = find_batteries();

    if battery_paths.is_empty() {
        // eprintln!("No battery found");
        return Err("No battery found".into());
    }

    let db = Database::open(&db_path)?;
    db.init_scheme()?;

    let mut timer = tokio::time::interval(Duration::from_secs(interval_secs));

    loop {
        tokio::select! {
            _ = timer.tick() => {
                let timestamp = unix_timestamp();

                for path in &battery_paths{
                    let info = get_battery_info(path);
                    db.insert_reading(&info.name, timestamp,&info.status.to_string(), info.capacity, info.power_now, info.energy_now)?;

                }
            }
            _ = signal::ctrl_c() => {
                println!("Shutting down juice daemon...");
                break;
            }
        }
    }

    Ok(())
}
