#!/usr/bin/env python3
# figure out setup in linux environment: https://levelup.gitconnected.com/from-python-to-daemon-how-to-turn-your-python-app-into-a-linux-service-controlled-by-systemd-d87b59adfe7a
import sys
import time
import os
from watchdog.observers import Observer # type: ignore
from watchdog.events import FileSystemEventHandler # type: ignore

from stalker import Stalker, CONFIG_FILE

# Get absolute path for logs
# TODO modify to handle non-unix based systes. eg linux
home_dir = os.path.expanduser("~")
log_path = os.path.join(home_dir, "Library", "Logs", "sophist.log")
error_log_path = os.path.join(home_dir, "Library", "Logs", "sophist.error.log")
# Redirect stdout and stderr to log files
sys.stdout = open(log_path, "a")
sys.stderr = open(error_log_path, "a")


def run_watcher():
    handler = Stalker()
    observer = Observer()
    
    # Watch root for file movements
    root_path = "C:\\" if sys.platform == "win32" else "/"
    observer.schedule(handler, root_path, recursive=True)
    
    # Watch config file for changes
    class ConfigHandler(FileSystemEventHandler):
        def on_modified(self, event):
            if event.src_path == CONFIG_FILE:
                handler.handle_tracked_files_change()
                print(f"Config file modified. Tracking {len(handler.tracked_files)} files.")
    config_observer = Observer()
    config_observer.schedule(ConfigHandler(), os.path.dirname(CONFIG_FILE))
    print("Starting observor")
    observer.start()
    config_observer.start()
    
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
        config_observer.stop()
    observer.join()
    config_observer.join()

def main():

    print("=== Daemon starting ===")
    print(f"Working directory: {os.getcwd()}")
    print(f"Python path: {sys.path}")

    # commands are used to modify tacked_files.json which is monitored by daemon
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python daeomon.py watch          # Start watching files")
        print("  python daeomon.py add <file>     # Add file to watch list")
        print("  python daeomon.py remove <file>  # Remove file from watch list")
        print("  python daeomon.py list           # List tracked files")
        sys.exit(1)

    command = sys.argv[1]
  

    if command == "watch":
        run_watcher()
    elif command == "list":
        handler = Stalker()
        for name, path in handler.tracked_files.items():
            print(f"{name}: {path}")
    else:
        print("Invalid command",file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
