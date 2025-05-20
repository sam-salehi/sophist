use crate::storage;
use std::env::consts::OS;
use super::macd;
use super::linuxd;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

pub fn init() {
    // creates a simple SQL database with two columns vector and pathd
    if is_daemon_alive() {
        println!("Deamon is currently alive, to re initalize run 'sophist shutdown'");
        return;
    }
    
    println!("Daemon dead");
    if let Err(e) = storage::make_sql_table() { // ! modify with udpated code.
        println!("Unable to create database {e:?}");
    } 

    setup_api_keys();
    setup_daemon();
}


pub fn shutdown() {
    if !is_daemon_alive() {
        println!("Daemon is already inactive. Halting shutdown.");
        return;
    }

    match OS {
        "macos" => macd::shutdown(),
        "linux" => linuxd::shutdown(),
        _ => panic!("Invalid operating system"),
    }
}


fn setup_daemon() {
    let daemon_path = std::path::Path::new("src/scripts/daemon.py");
    assert!(daemon_path.exists(), "daemon.py not found");

    match OS {
        "macos" => macd::setup(daemon_path),
        "linux" => linuxd::setup(daemon_path),
        os => println!("Unsupported operating system: {}", os)
    }
}

fn is_daemon_alive() -> bool {
    match OS {
        "macos" => {
            // Get user ID
            let uid = Command::new("id")
                .arg("-u")
                .output()
                .expect("Failed to get user ID")
                .stdout;
            let uid = String::from_utf8(uid).unwrap().trim().to_string();

            // Check if daemon is running
            let output = Command::new("launchctl")
                .args(["print", &format!("gui/{}/com.sophist.filewatcher", uid)])
                .output()
                .expect("Failed to check daemon status");

            output.status.success()
        },
        "linux" => {
            // TODO: Check systemd service status
            false
        },
        _ => false
    }
}


fn setup_api_keys() {
    let mut gemini_key = String::new();
    let mut jina_key = String::new();
    
    println!("Please enter your API keys:");
    print!("GEMINI_API_KEY=");
    io::stdout().flush().unwrap();  // Flush to show prompt before read
    io::stdin().read_line(&mut gemini_key).unwrap();
    
    print!("JINA_API_KEY=");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut jina_key).unwrap();

    let env_content = format!(
        "GEMINI_API_KEY={}\nJINA_API_KEY={}",
        gemini_key.trim(),
        jina_key.trim()
    );

    fs::write(".env", env_content)
        .expect("Failed to write API keys to .env file");
}