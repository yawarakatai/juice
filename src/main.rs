mod battery;
mod daemon;
mod db;
mod export;
mod ui;
mod utils;

use clap::{Parser, Subcommand};
use db::{default_db_path, Database};
use std::error::Error;
use std::path::PathBuf;
use utils::{format_duration, format_size, format_timestamp};

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

    /// Export data to CSV
    Export {
        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,

        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let battery_paths = battery::find_batteries();

            if battery_paths.is_empty() {
                println!("No battery found");
                return Ok(());
            }

            for path in battery_paths {
                let battery_info = battery::get_battery_info(&path);
                if cli.verbose {
                    ui::cli::print_verbose(&battery_info);
                } else {
                    ui::cli::print_normal(&battery_info);
                }
            }
        }
        Some(Commands::Daemon { interval }) => {
            let db_path = default_db_path();
            println!("Starting daemon with {}s interval...", interval);
            daemon::run(db_path, interval)?;
        }
        Some(Commands::Status) => {
            handle_status_command()?;
        }
        Some(Commands::Export { output, from, to }) => {
            handle_export_command(output, from, to)?;
        }
    }

    Ok(())
}

fn handle_status_command() -> Result<(), Box<dyn Error>> {
    let db_path = default_db_path();
    let file_size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

    println!("Database: {}", db_path.display());
    println!("Size:     {}", format_size(file_size));

    match Database::open(&db_path) {
        Ok(db) => {
            db.init_scheme()?;
            let count = db.count_readings().unwrap_or(0);
            println!("Records:  {}", count);

            if let (Some(first_timestamp), Some(last_timestamp)) =
                (db.first_timestamp(), db.last_timestamp())
            {
                println!();
                println!("First:    {}", format_timestamp(first_timestamp));
                println!("Last:     {}", format_timestamp(last_timestamp));
                println!(
                    "Period:   {}",
                    format_duration(first_timestamp, last_timestamp)
                );
            }
        }
        Err(e) => println!("Database error: {}", e),
    }
    Ok(())
}

fn handle_export_command(
    output: Option<PathBuf>,
    from: Option<String>,
    to: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let db_path = default_db_path();
    let db = Database::open(&db_path)?;

    let from_timestamp = from.as_ref().and_then(|s| utils::parse_date(s));
    let to_timestamp = to.as_ref().and_then(|s| utils::parse_date(s));

    let readings = db.get_readings(from_timestamp, to_timestamp)?;

    match output {
        Some(path) => {
            let file = std::fs::File::create(path)?;
            export::write_csv(file, &readings)?;
        }
        None => {
            export::write_csv(std::io::stdout(), &readings)?;
        }
    }
    Ok(())
}
