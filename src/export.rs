use crate::{db::Reading, format_timestamp};
use std::io::{self, Write};

pub fn write_csv(mut writer: impl Write, readings: &[Reading]) -> io::Result<()> {
    writeln!(
        writer,
        "timestamp,datetime,battery,status,capacity,power_now,energy_now"
    )?;

    for r in readings {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},",
            r.timestamp,
            format_timestamp(r.timestamp),
            r.battery,
            r.status,
            r.capacity.map(|v| v.to_string()).unwrap_or_default(),
            r.power_now.map(|v| format!("{:.2}", v)).unwrap_or_default(),
            r.energy_now
                .map(|v| format!("{:.2}", v))
                .unwrap_or_default(),
        )?;
    }

    Ok(())
}
