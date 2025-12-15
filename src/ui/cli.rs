use crate::battery::{BatteryInfo, BatteryStatus};
use colored::*;

pub fn print_normal(info: &BatteryInfo) {
    let color = get_status_color(info);

    let symbol = match info.status {
        BatteryStatus::Charging => "⚡️".color(color),
        BatteryStatus::Discharging => "↓".color(color),
        BatteryStatus::Full => "✓".color(color),
        _ => "?".color(color),
    };

    let capacity = info.capacity.unwrap_or(0);
    let bar = draw_progress_bar(capacity, 10, color);

    let power_str = info
        .power_now
        .map(|p| format!("{:.1}W", p))
        .unwrap_or_else(|| "--W".to_string())
        .white();

    let time_str = format_time(info.calc_remaining_time());

    println!(
        "{:<6} {} {:>4} {}  {}  {}",
        info.name.bold(),
        bar,
        format!("{}%", capacity).color(color).bold(),
        symbol,
        power_str,
        time_str,
    );
}

pub fn print_verbose(info: &BatteryInfo) {
    let color = get_status_color(info);
    let capacity = info.capacity.unwrap_or(0);

    // Header
    println!(
        "{} {} {}",
        info.name.bold().white(),
        draw_progress_bar(capacity, 20, color),
        info.status.to_string().color(color).bold()
    );

    let label = |s: &str| s.truecolor(127, 127, 127);
    let val = |s: String| s.white().bold();

    println!(
        "  {:12} {}",
        label("Power:"),
        val(info
            .power_now
            .map(|p| format!("{:.1} W", p))
            .unwrap_or("--".into()))
    );

    println!(
        "  {:<12} {}",
        label("Remaining:"),
        val(format_time(info.calc_remaining_time()))
    );

    println!(
        "  {:<12} {}",
        label("Capacity:"),
        val(format!("{} %", capacity))
    );

    if let (Some(now), Some(full)) = (info.energy_now, info.energy_full) {
        println!(
            "  {:<12} {} / {} Wh",
            label("Energy:"),
            val(format!("{:.1}", now)),
            val(format!("{:.1}", full)),
        );
    }

    if let Some(cycle) = info.cycle_count {
        println!("  {:<12} {}", label("Cycle count:"), val(cycle.to_string()))
    }

    if let Some(health) = info.calc_health() {
        let h_color = if health > 80.0 {
            Color::Green
        } else {
            Color::Red
        };
        println!(
            "  {:<12} {}",
            label("Health:"),
            format!("{:.1} %", health).color(h_color)
        );
    }

    println!(
        "  {:<12} {}",
        label("Technology:"),
        info.technology.as_deref().unwrap_or("Unknown")
    );
}

fn get_status_color(info: &BatteryInfo) -> Color {
    match info.status {
        BatteryStatus::Charging => Color::Cyan,
        BatteryStatus::Full => Color::Blue,
        BatteryStatus::Unknown => Color::White,
        _ => match info.capacity.unwrap_or(0) {
            0..=20 => Color::Red,
            21..=50 => Color::Yellow,
            _ => Color::Green,
        },
    }
}

fn draw_progress_bar(percent: u32, width: usize, color: Color) -> ColoredString {
    let filled = (percent as usize * width / 100).min(width);
    let empty = width - filled;
    let bar_str = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
    bar_str.color(color)
}

fn format_time(time: Option<(u32, u32)>) -> String {
    time.map(|(h, m)| format!("{}h{:02}m", h, m))
        .unwrap_or_else(|| "--:--".to_string())
}
