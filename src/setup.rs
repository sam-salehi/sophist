use crate::storage;
use std::env::consts::OS;
use std::fs;
use std::io::Write;
use std::process::Command;

pub fn init() {
    // creates a simple SQL database with two columns vector and pathd
    if let Err(e) = storage::make_sql_table() {

        println!("Unable to create database {e:?}");
    } 

    // launch daemon
    setup_daemon();
}

fn setup_daemon() {
    let daemon_path = std::path::Path::new("src/scripts/daemon.py");
    assert!(daemon_path.exists(), "daemon.py not found");

    match OS {
        "macos" => setup_macos_daemon(daemon_path),
        "linux" => setup_linux_daemon(daemon_path),
        // "windows" => setup_windows_daemon(daemon_path),
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

    // Set correct ownership and make executable
    // let user = std::env::var("USER").expect("Could not get username");
    // Command::new("sudo")
    //     .args(["chown", &format!("{}:staff", user), abs_daemon_path.to_str().unwrap()])
    //     .status()
    //     .expect("Failed to set daemon.py ownership");

    // Command::new("chmod")
    //     .args(["+x", abs_daemon_path.to_str().unwrap()])
    //     .status()
    //     .expect("Failed to make daemon.py executable");

    // Create directory with standard permissions if it doesn't exist
    fs::create_dir_all(&launch_agents_dir)
        .expect("Failed to create LaunchAgents directory");

    // Write plist file directly
    println!("Creating plist file...");

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
    <string>{}/Library/Logs/sophist.log</string>
    <key>StandardErrorPath</key>
    <string>{}/Library/Logs/sophist.error.log</string>
</dict>
</plist>"#, 
        abs_daemon_path.display(),
        abs_daemon_path.display(), //.parent().unwrap().display(), // for PYTHONPATH
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
    println!("Loading Daemon");
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

// fn setup_windows_daemon(daemon_path: &std::path::Path) {
//     println!("Setting up daemon for Windows...");
//     // TODO: Create Windows service
// }
// question: how the fuck do daeons work.


