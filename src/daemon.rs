use crate::battery::{find_batteries, get_battery_info};
use crate::db::Database;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn run(db_path: PathBuf, interval_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
    let battery_paths = find_batteries();
    if battery_paths.is_empty() {
        return Err("No battery found".into());
    }

    let db = Database::open(&db_path)?;
    db.init_scheme()?;

    // Handle Ctrl+C
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    println!("Daemon started (interval: {}s)", interval_secs);

    while running.load(Ordering::SeqCst) {
        let timestamp = unix_timestamp();

        for path in &battery_paths {
            let info = get_battery_info(path);
            db.insert_reading(
                &info.name,
                timestamp,
                &info.status.to_string(),
                info.capacity,
                info.power_now,
                info.energy_now,
            )?;
        }

        thread::sleep(Duration::from_secs(interval_secs));
    }

    println!("Shutting down juice daemon...");
    Ok(())
}
