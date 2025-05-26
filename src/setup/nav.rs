use crate::storage;
use std::env::consts::OS;
use super::macd;
use super::linuxd;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::path::Path;

const DAEMON_PATH: &str = "src/scripts/daemon.py";

pub fn init() {
    // creates a simple SQL database with two columns vector and pathd
    if is_daemon_alive() {
        println!("Deamon is currently alive, to re initalize run 'sophist shutdown'");
        return;
    }
    
    println!("Daemon dead");
    if let Err(e) = storage::make_table() { // ! modify with udpated code.
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
    let script_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(DAEMON_PATH);
    assert!(script_path.exists(), "daemon.py not found");

    match OS {
        "macos" => macd::setup(&script_path),
        "linux" => linuxd::setup(&script_path),
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


fn api_keys_are_setup() -> bool {
    let env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if !env_path.exists() {return false;}
        
    let env_content = fs::read_to_string(&env_path)
        .expect("Failed to read .env file");
    return env_content.contains("GEMINI_API_KEY=") &&  env_content.contains("JINA_API_KEY=");
}

fn setup_api_keys() {
    let mut gemini_key = String::new();
    let mut jina_key = String::new();

    if api_keys_are_setup() {
        let mut status = String::new();
        while status.trim() != "y" {
            println!("API keys are already setup. Would you like to replace them(y/n) :");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut status).unwrap();
            if status.trim() == "n" {return;}
        }

    } 
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



pub(crate) fn setup(daemon_path: &std::path::Path) {
    // Check .env file exists and contains required keys
    let env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    assert!(env_path.exists(), ".env file not found at project root");
    
    let env_content = fs::read_to_string(&env_path)
        .expect("Failed to read .env file");
    
    assert!(env_content.contains("GEMINI_API_KEY="), "GEMINI_API_KEY not found in .env");
    assert!(env_content.contains("JINA_API_KEY="), "JINA_API_KEY not found in .env");

    // ... rest of setup code ...
}