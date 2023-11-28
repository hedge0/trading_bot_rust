use chrono::{DateTime, Utc};
use std::{fs::OpenOptions, io::Write, path::Path, process::exit};

// Function that logs a message to text file.
fn log_to_file<P: AsRef<Path>>(path: P, message: &str) -> std::io::Result<()> {
    let mut file: std::fs::File = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)?;

    writeln!(file, "{}", message)?; // Writes the message and a newline character.

    Ok(())
}

// Function that logs a message.
pub(crate) fn log_message(status: String) {
    println!("{}", status);
    if !cfg!(test) {
        let now: DateTime<Utc> = Utc::now();
        let formatted_now: String = now.format("%Y-%m-%d %H:%M:%S%.9f UTC").to_string();
        let _ = log_to_file("log.txt", &format!("{}   {}", formatted_now, status));
    }
}

// Function that logs an error message and exits the program.
pub(crate) fn log_error(error: String) {
    eprintln!("{}", format!("Error: {}.", error));
    if !cfg!(test) {
        let now: DateTime<Utc> = Utc::now();
        let formatted_now: String = now.format("%Y-%m-%d %H:%M:%S%.9f UTC").to_string();
        let _ = log_to_file("log.txt", &format!("{}   Error: {}.", formatted_now, error));
    }
    log_message(format!("Exiting..."));
    exit(1);
}
