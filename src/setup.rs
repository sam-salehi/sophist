use crate::storage;
use core::panic;
use std::env::consts::OS;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

// TODO refactor in mac specific and linux specific files.

pub fn init() {
    // creates a simple SQL database with two columns vector and pathd
    if is_daemon_alive() {
        println!("Deamon is currently alive, to re initalize run 'sophist shutdown'");
        return;
    }
    
    println!("Daemon dead");
    if let Err(e) = storage::make_sql_table() {
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
        "macos" => shutdown_macos_daemon(),
        "linux" => shutdown_linux_daemon(),
        _ => panic!("Invalid operating system"),
    }
}

fn shutdown_macos_daemon() { // TODO make this mac specific.
    let home_dir = std::env::var("HOME").expect("Could not find home directory"); 
    let plist_path = format!("{}/Library/LaunchAgents/com.sophist.filewatcher.plist", home_dir);

    // Get user ID
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .expect("Failed to get user ID")
        .stdout;
    let uid = String::from_utf8(uid).unwrap().trim().to_string();

    // Unload the daemon
    let output = Command::new("launchctl")
        .args(["bootout", &format!("gui/{}", uid), &plist_path])
        .output()
        .expect("Failed to execute launchctl bootout");

    if !output.status.success() {
        eprintln!("Shutdown error: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to shutdown daemon");
    }

    println!("Daemon successfully shutdown");
}

fn shutdown_linux_daemon() {
    // TODO
    panic!("Not yet implemented");
}


// checks depending on os. wether daemon is running or not.
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

fn setup_daemon() {
    let daemon_path = std::path::Path::new("src/scripts/daemon.py");
    assert!(daemon_path.exists(), "daemon.py not found");

    match OS {
        "macos" => setup_macos_daemon(daemon_path),
        "linux" => setup_linux_daemon(daemon_path),
        os => println!("Unsupported operating system: {}", os)
    }
}

// installation of packages with /usr/bin/python3 -m pip install watchdog

fn setup_macos_daemon(daemon_path: &std::path::Path) {
    println!("Setting up daemon for macOS...");
    
    let home_dir = std::env::var("HOME").expect("Could not find home directory");
    let launch_agents_dir = format!("{}/Library/LaunchAgents", home_dir);
    let plist_path = format!("{}/com.sophist.filewatcher.plist", launch_agents_dir);
    
    // Get absolute path to daemon.py and verify it exists
    let abs_daemon_path = daemon_path.canonicalize()
        .expect("Could not get absolute path to daemon.py");
    assert!(abs_daemon_path.exists(), "daemon.py not found at: {}", abs_daemon_path.display());
    println!("Using daemon at: {}", abs_daemon_path.display());

    // Create directory with standard permissions if it doesn't exist
    fs::create_dir_all(&launch_agents_dir)
        .expect("Failed to create LaunchAgents directory");

    // make PLIST content
    let plist_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.sophist.filewatcher</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/bin/python3</string>
        <string>{}</string>
        <string>watch</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PYTHONPATH</key>
        <string>{}</string>
    </dict>
    <key>WorkingDirectory</key>
    <string>{}</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key> 
    <true/>
    <key>com.apple.security.files.all</key>
    <true/>
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}/library/logs/sophist.log</string>
    <key>StandardErrorPath</key>
    <string>{}/library/logs/sophist.error.log</string>
</dict>
</plist>"#, 
        abs_daemon_path.display(),
        abs_daemon_path.display(),  // for PYTHONPATH
        abs_daemon_path.parent().unwrap().parent().unwrap().display(), // project root for WorkingDirectory
        home_dir,
        home_dir
    );

    let mut file = fs::File::create(&plist_path)
        .expect("Failed to create plist file");
    file.write_all(plist_content.as_bytes())
        .expect("Failed to write plist content");

    // Unload if exists
    Command::new("launchctl")
        .args(["unload", &plist_path])
        .status()
        .ok();

    // Load the daemon
    let output = Command::new("launchctl")
        .args(["load", "-w", &plist_path])
        .output()
        .expect("Failed to execute launchctl");

    if !output.status.success() {
        eprintln!("Load error: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to load daemon");
    }

    println!("Daemon setup complete. Check logs at ~/Library/Logs/sophist.log");
}

fn setup_linux_daemon(daemon_path: &std::path::Path) {
    println!("Setting up daemon for Linux...");
    // TODO: Create and enable systemd service
    panic!("Not implemented yet");
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