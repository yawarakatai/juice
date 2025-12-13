use directories::ProjectDirs;
use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    pub fn init_scheme(&self) -> Result<()> {
        self.conn.execute_batch(
            "
                CREATE TABLE IF NOT EXISTS readings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp INTEGER NOT NULL,
                    battery TEXT NOT NULL,
                    status TEXT,
                    capacity INTEGER,
                    power_now REAL,
                    energy_now REAL,
                );

                CREATE INDEX IF NOT EXISTS idx_readings_timestamp
                    ON readings(timestamp);
                CREATE INDEX IF NOT EXISTS idx_readings_battery_time
                    ON readings(battery, timestamp);
            ",
        )?;
        Ok(())
    }

    pub fn insert_reading(
        &self,
        battery: &str,
        timestamp: i64,
        status: &str,
        capacity: Option<u32>,
        power_now: Option<f32>,
        energy_now: Option<f32>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO readings
            (timestamp, battery, status, capacity, power_now, energy_now)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6 )",
            (timestamp, battery, status, capacity, power_now, energy_now),
        )?;
        Ok(())
    }

    pub fn count_readings(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM readings", [], |row| row.get(0))
    }
}

pub fn default_db_path() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("", "", "juice") {
        let data_dir = project_dirs.data_dir();
        std::fs::create_dir_all(data_dir).ok();
        data_dir.join("history.db")
    } else {
        PathBuf::from(".juice-history.db")
    }
}
