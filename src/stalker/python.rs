use std::process::Command;

const DAEMON_PATH: &str = "src/scripts/daemon.py";


pub(crate) fn stalker_init() {
    //TODO
}

pub(crate) fn stalk(abs_path: &str) -> notify::Result<()> {
    println!("Adding file to watch");
    
    
    let status: std::process::ExitStatus = Command::new("python3")
        .arg(DAEMON_PATH)
        .arg("add")
        .arg(abs_path)
        .status()
        .expect(&format!("Failed to execute Python script at {}", DAEMON_PATH));

    if !status.success() {
        println!("Daemon's add failed with exit code: {}", status);
    }

    Ok(())
}

pub(crate) fn abandon(abs_path: &str) -> notify::Result<()> {
    println!("Abandoning file");
    
    let status = Command::new("python3")
        .arg(DAEMON_PATH)
        .arg("remove")
        .arg(abs_path)
        .status()
        .expect(&format!("Failed to execute Python script at {}", DAEMON_PATH));

    if !status.success() {
        print!("Daemon's abandon failed with exit code: {}", status);
    }
    Ok(())
}