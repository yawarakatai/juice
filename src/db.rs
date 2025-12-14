use directories::ProjectDirs;
use rusqlite::{Connection, Result};
use std::path::PathBuf;

use crate::battery::BatteryStatus;

pub struct Reading {
    pub battery: String,
    pub timestamp: i64,
    pub status: BatteryStatus,
    pub capacity: Option<u32>,
    pub power_now: Option<f32>,
    pub energy_now: Option<f32>,
}

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
                    energy_now REAL
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
            "
            INSERT INTO readings
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

    pub fn first_timestamp(&self) -> Option<i64> {
        self.conn
            .query_row(
                "SELECT timestamp FROM readings ORDER BY timestamp ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn last_timestamp(&self) -> Option<i64> {
        self.conn
            .query_row(
                "SELECT timestamp FROM readings ORDER BY timestamp DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn get_readings(&self, from: Option<i64>, to: Option<i64>) -> Result<Vec<Reading>> {
        let start = from.unwrap_or(i64::MIN);
        let end = to.unwrap_or(i64::MAX);

        let mut stmt = self.conn.prepare(
            "
            SELECT timestamp, battery, status, capacity, power_now, energy_now
            FROM readings
            WHERE timestamp >= ?1 AND timestamp <= ?2
            ORDER BY timestamp ASC
            ",
        )?;

        let rows = stmt.query_map([start, end], |row| {
            let status_str: String = row.get(2)?;
            Ok(Reading {
                timestamp: row.get(0)?,
                battery: row.get(1)?,
                status: status_str.parse().unwrap_or(BatteryStatus::Unknown),
                capacity: row.get(3)?,
                power_now: row.get(4)?,
                energy_now: row.get(5)?,
            })
        })?;

        let readings = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(readings)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_init_and_insert() {
        let db = Database::open(&PathBuf::from(":memory:")).unwrap();
        db.init_scheme().unwrap();

        db.insert_reading(
            "BAT0",
            1234567890,
            "Discharging",
            Some(85),
            Some(12.5),
            Some(45.0),
        )
        .unwrap();

        assert_eq!(db.count_readings().unwrap(), 1);
    }
}
