use std::process::Command;
use std::fs;
use std::io::Write;

pub(crate) fn setup(daemon_path: &std::path::Path) {
    println!("Setting up daemon for macOS...");
    
    let home_dir = std::env::var("HOME").expect("Could not find home directory");
    let launch_agents_dir = format!("{}/Library/LaunchAgents", home_dir);
    let plist_path: String = format!("{}/com.sophist.filewatcher.plist", launch_agents_dir);
    
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
    <key>StandardErrorPath</key>s
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


pub(crate) fn shutdown() { // TODO make this mac specific.
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


