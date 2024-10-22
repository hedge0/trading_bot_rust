use chrono::{DateTime, Utc};
use std::{fs::OpenOptions, io::Write, path::Path, process::exit};

/// Logs a message to a specified text file.
///
/// # Arguments
///
/// * `path` - A reference to the file path where the log message will be written.
/// * `message` - The log message to be written to the file.
///
/// # Returns
///
/// * `Ok(())` if the operation is successful.
/// * `Err(std::io::Error)` if there is an error during file operations.
///
/// # Example
///
/// ```
/// log_to_file("log.txt", "This is a log message").unwrap();
/// ```
fn log_to_file<P: AsRef<Path>>(path: P, message: &str) -> std::io::Result<()> {
    let mut file: std::fs::File = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)?;

    writeln!(file, "{}", message)?; // Writes the message and a newline character.

    Ok(())
}

/// Logs a message to the console and a text file.
///
/// The message is printed to the console. If not in test mode, the message is also logged to a
/// text file with a timestamp in UTC.
///
/// # Arguments
///
/// * `status` - The log message to be printed and written to the log file.
///
/// # Example
///
/// ```
/// log_message("Application started.".to_string());
/// ```
pub(crate) fn log_message(status: String) {
    println!("{}", status);
    if !cfg!(test) {
        let now: DateTime<Utc> = Utc::now();
        let formatted_now: String = now.format("%Y-%m-%d %H:%M:%S%.9f UTC").to_string();
        let _ = log_to_file("log.txt", &format!("{}   {}", formatted_now, status));
    }
}

/// Logs an error message and exits the program.
///
/// The error message is printed to the console and logged to a text file with a timestamp in UTC.
/// After logging the error, the program will exit with a status code of 1.
///
/// # Arguments
///
/// * `error` - The error message to be printed and logged.
///
/// # Example
///
/// ```
/// log_error("An unexpected error occurred.".to_string());
/// ```
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
