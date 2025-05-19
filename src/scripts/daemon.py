#!/usr/bin/env python3
# figure out setup in linux environment: https://levelup.gitconnected.com/from-python-to-daemon-how-to-turn-your-python-app-into-a-linux-service-controlled-by-systemd-d87b59adfe7a
import sys
import time
import os
from watchdog.observers import Observer # type: ignore
from watchdog.events import FileSystemEventHandler # type: ignore

from stalker import Stalker, CONFIG_FILE

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
                print("Config file changed, reloading tracked files...")
                handler.tracked_files = handler.load_tracked_files()
                print(f"Now tracking {len(handler.tracked_files)} files")
    
    config_observer = Observer()
    config_observer.schedule(ConfigHandler(), os.path.dirname(CONFIG_FILE))
    
    observer.start()
    config_observer.start()
    print("File watcher started")
    
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
        config_observer.stop()
    observer.join()
    config_observer.join()

def main():
    # commands are used to modify tacked_files.json which is monitored by daemon
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python daeomon.py watch          # Start watching files")
        print("  python daeomon.py add <file>     # Add file to watch list")
        print("  python daeomon.py remove <file>  # Remove file from watch list")
        print("  python daeomon.py list           # List tracked files")
        sys.exit(1)

    command = sys.argv[1]
    handler = Stalker()

    if command == "watch":
        run_watcher()
    elif command == "add" and len(sys.argv) == 3:
        handler.add_file(sys.argv[2])
    elif command == "remove" and len(sys.argv) == 3:
        handler.remove_file(sys.argv[2])
    elif command == "list":
        for name, path in handler.tracked_files.items():
            print(f"{name}: {path}")
    else:
        print("Invalid command")
        sys.exit(1)

if __name__ == "__main__":
    main()
