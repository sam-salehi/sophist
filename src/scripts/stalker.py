import sys
from watchdog.observers import Observer # type: ignore
from watchdog.events import FileSystemEventHandler # type: ignore
import sys
import time
import os
import subprocess

class FileMovementHandler(FileSystemEventHandler):
    def __init__(self, target_file):
        self.target_file = os.path.basename(target_file)
        print(f"Tracking file: {self.target_file}")
        
    def on_moved(self, event):
        # TODO call rust script to update db
        if event.is_directory:
            return
        
        # Check if this event involves our target file
        if (os.path.basename(event.src_path) == self.target_file or 
            os.path.basename(event.dest_path) == self.target_file):
            print(f"File moved/renamed from: {event.src_path} to: {event.dest_path}")

            
    def on_modified(self, event):
        # TODO call rust script to generate new embedding.
        # keep running
        if event.is_directory:
            return
        if os.path.basename(event.src_path) == self.target_file:
            print(f"File modified: {event.src_path}")



    def on_deleted(self, event):
        if event.is_directory:
            return
        if os.path.basename(event.src_path) == self.target_file:
            print(f"File deleted: {event.src_path}")
            
            try:
                binary_path = self.get_binary_path("handle_delete")
                result = subprocess.run([binary_path, event.src_path], 
                                     capture_output=True, 
                                     text=True)
                
                if result.returncode != 0:
                    print(f"Error running handle_delete: {result.stderr}")
                else:
                    print("Successfully handled deletion")
                    sys.exit(0)
                    
            except FileNotFoundError as e:
                print(f"Binary not found: {e}")
                print("Did you run 'cargo build' first?")
            except Exception as e:
                print(f"Failed to execute handle_delete: {e}")

    def get_binary_path(self, name):
        """Get path to binary, checking both debug and release builds"""
        base_dir = os.path.join(os.path.dirname(__file__), "..", "..")
        
        # Try debug build first
        debug_path = os.path.join(base_dir, "target", "debug", name)
        if os.path.exists(debug_path):
            return debug_path
        
        # Try release build
        release_path = os.path.join(base_dir, "target", "release", name)
        if os.path.exists(release_path):
            return release_path
        
        raise FileNotFoundError(f"Could not find binary '{name}' in debug or release directories")

def watch_file(path):
    event_handler = FileMovementHandler(path)
    observer = Observer()
    

    # Watch from root directory to catch all possible moves
    root_path = "C:\\" if sys.platform == "win32" else "/" # / for macOS and linux
    
    observer.schedule(event_handler, root_path, recursive=True)
    
    observer.start()
    print(f"Started watching: {path}")
    print(f"Watching from root: {root_path}")
    
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    observer.join()


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python stalker.py <file_path>")
        sys.exit(1)
        
    file_path = os.path.abspath(sys.argv[1])
    print("watching ", file_path)
    watch_file(file_path)
