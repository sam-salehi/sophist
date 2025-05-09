import sys
import time
import os
import json
import subprocess
from watchdog.observers import Observer # type: ignore
from watchdog.events import FileSystemEventHandler # type: ignore

CONFIG_FILE = os.path.join(os.path.dirname(__file__), "tracked_files.json")

class FileMovementHandler(FileSystemEventHandler):
    def __init__(self):
        self.tracked_files = self.load_tracked_files()
        print(f"Tracking {len(self.tracked_files)} files")
        
    def load_tracked_files(self):
        if os.path.exists(CONFIG_FILE):
            with open(CONFIG_FILE, 'r') as f:
                return json.load(f)
        return {}
        
    def save_tracked_files(self):
        with open(CONFIG_FILE, 'w') as f:
            json.dump(self.tracked_files, f, indent=2)
    
    def add_file(self, path):
        abs_path = os.path.abspath(path)
        if not os.path.exists(abs_path):
            print(f"File not found: {abs_path}")
            return False
        
        self.tracked_files[os.path.basename(abs_path)] = abs_path
        self.save_tracked_files()
        print(f"Now tracking: {abs_path}")
        return True
        
    def remove_file(self, path):
        basename = os.path.basename(path)
        if basename in self.tracked_files:
            del self.tracked_files[basename]
            self.save_tracked_files()
            print(f"Stopped tracking: {path}")
            return True
        return False

    def on_moved(self, event):
        if event.is_directory:
            return
            # TODO check to see we don't have decendants. 
        src_name = os.path.basename(event.src_path)
        dst_name = os.path.basename(event.dest_path)        
        if src_name in self.tracked_files and self.tracked_files[src_name] == event.src_path:
            print(f"Tracked file moved: {event.src_path} -> {event.dest_path}")
            self.tracked_files[src_name] = event.dest_path
            self.save_tracked_files()

            try:
                binary_path = self.get_binary_path("handle_move")
                result = subprocess.run([binary_path,event.src_path,event.dest_path],
                                        capture_output=True,
                                        text=True,
                                        cwd="../..")
                print("Script output:", result.stdout)
                if result.stderr:
                    print("Script errors:", result.stderr)
                print("Script return code:", result.returncode)
            except Exception as e:
                print(f"Failed to handle move: {e}")

            
    def on_modified(self, event):
        if event.is_directory:
            return
        basename = os.path.basename(event.src_path)
        if basename in self.tracked_files and self.tracked_files[basename] == event.src_path:
            print(f"File modified: {event.src_path}")
            try:
                binary_path = self.get_binary_path("handle_modify")
                result = subprocess.run([binary_path, event.src_path],
                                     capture_output=True,
                                     text=True,
                                     cwd="../..")  # Specify your desired working directory here
                print("Script output:", result.stdout)
                if result.stderr:
                    print("Script errors:", result.stderr)
                print("Script return code:", result.returncode)
            except Exception as e:
                print(f"Failed to handle modification at {event.src_path}:\n {e}")

            

    def on_deleted(self, event):
        if event.is_directory:
            return
            
        basename = os.path.basename(event.src_path)
        if basename in self.tracked_files:
            print(f"Tracked file deleted: {event.src_path}")
            try:
                binary_path = self.get_binary_path("handle_delete")
                result = subprocess.run([binary_path, event.src_path], 
                                     capture_output=True, 
                                     text=True,
                                     cwd="../..")
                print("Script output:", result.stdout)
                if result.stderr:
                    print("Script errors:", result.stderr)
                print("Script return code:", result.returncode)
                if result.returncode == 0:
                    del self.tracked_files[basename]
                    self.save_tracked_files()
            except Exception as e:
                print(f"Failed to handle deletion: {e}")

    def get_binary_path(self, name):
        base_dir = os.path.join(os.path.dirname(__file__), "..", "..")
        debug_path = os.path.join(base_dir, "target", "debug", name)
        if os.path.exists(debug_path):
            return debug_path
        raise FileNotFoundError(f"Binary not found: {name}")

def run_watcher():
    handler = FileMovementHandler()
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
        print("  python stalker.py watch          # Start watching files")
        print("  python stalker.py add <file>     # Add file to watch list")
        print("  python stalker.py remove <file>  # Remove file from watch list")
        print("  python stalker.py list           # List tracked files")
        sys.exit(1)

    command = sys.argv[1]
    handler = FileMovementHandler()

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
