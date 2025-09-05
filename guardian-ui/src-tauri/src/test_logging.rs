use std::fs;
use std::io::Write;
use chrono::Utc;

// Simple logging test
fn test_log_error(message: &str) {
    // Also print to console for debugging
    println!("GUARDIAN LOG: {}", message);
    
    // Try multiple locations for the log file
    let possible_log_dirs = vec![
        "log",  // Current directory
        "./log",  // Explicit current directory
        std::env::current_dir().map(|d| d.join("log")).unwrap_or_else(|_| "log".into()),
        std::env::temp_dir().join("guardian_logs"),
    ];
    
    for log_dir in possible_log_dirs {
        if let Ok(log_dir) = log_dir {
            if fs::create_dir_all(&log_dir).is_ok() {
                let log_file = log_dir.join("guardian_error.log");
                let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
                let log_entry = format!("[{}] {}\n", timestamp, message);
                
                if let Ok(mut file) = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file)
                {
                    let _ = file.write_all(log_entry.as_bytes());
                    let _ = file.flush();
                    println!("LOG WRITTEN TO: {:?}", log_file);
                    return; // Success, exit
                }
            }
        }
    }
    
    // If all else fails, try to write to a file in the temp directory
    if let Ok(temp_file) = std::env::temp_dir().join("guardian_error.log") {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let log_entry = format!("[{}] {}\n", timestamp, message);
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&temp_file)
        {
            let _ = file.write_all(log_entry.as_bytes());
            println!("LOG WRITTEN TO TEMP: {:?}", temp_file);
        }
    }
}

fn main() {
    println!("Testing Guardian logging...");
    test_log_error("Test log message 1");
    test_log_error("Test log message 2");
    test_log_error("Test log message 3");
    println!("Logging test complete. Check for log files.");
}
