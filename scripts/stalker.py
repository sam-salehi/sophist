import os
import json
import subprocess
from watchdog.events import FileSystemEventHandler # type: ignore


CONFIG_FILE = os.path.join(os.path.dirname(__file__), "tracked_files.json")

class Stalker(FileSystemEventHandler):
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
        if basename in self.tracked_files and self.tracked_files[basename]==path:
            del self.tracked_files[basename]
            self.save_tracked_files()
            print(f"Stopped tracking: {path}")
            return True
        return False

    def on_moved(self, event):

        src_path = event.src_path
        dest_path = event.dest_path

        if event.is_directory:
            descendants = self.get_common_descendants(src_path,dest_path)
        else:
            descendants = [(src_path,dest_path)]
    

        for src_path,dest_path in descendants:
            src_name = os.path.basename(event.src_path)
            dest_name = os.path.basename(event.dest_path) 
            if src_name in self.tracked_files and self.tracked_files[src_name] == src_path:
                print(f"Tracked file moved: {src_path} -> {dest_path}")

                if src_name == dest_name: # chekc for basename change
                    self.tracked_files[src_name] = dest_path
                else:
                    self.tracked_files[dest_name] = dest_path
                    del self.tracked_files[src_name]


                self.save_tracked_files()            
                try:
                    self.execute_rs("handle_move",[src_path,dest_path])
                except Exception as e:
                    print(f"Failed to handle move: {e}")
    
    def get_common_descendants(self, src, dst):
        descendants = []
        for name, path in self.tracked_files.items():
            if path.startswith(src):
                # This file is under the source directory
                # Calculate its relative path and create new destination
                rel_path = path[len(src):].lstrip('/')  # Remove leading slash
                new_dest = os.path.join(dst, rel_path)
                descendants.append((path, new_dest))
        return descendants

            
    def on_modified(self, event):
        if event.is_directory:
            return
        basename = os.path.basename(event.src_path)
        if basename in self.tracked_files and self.tracked_files[basename] == event.src_path:
            print(f"File modified: {event.src_path}")
            try:
                self.execute_rs("handle_modified",[event.src_path])
            except Exception as e:
                print(f"Failed to handle modification at {event.src_path}:\n {e}")

    def on_deleted(self, event):
        if event.is_directory:
            return
            
        basename = os.path.basename(event.src_path)
        if basename in self.tracked_files:
            print(f"Removing tracked file: {event.src_path}")
            try:
                rc = self.execute_rs("handle_delete", [event.src_path])
                if rc == 0:
                    del self.tracked_files[basename]
                    self.save_tracked_files()
            except Exception as e:
                print(f"Failed to handle deletion: {e}")

    def execute_rs(self, exec, args):    
        # Get current environment and add our variables
        env = os.environ.copy()
        
        # Load .env file
        env_path = os.path.join(os.path.dirname(__file__), "..", "..", ".env")
        if os.path.exists(env_path):
            with open(env_path) as f:
                for line in f:
                    if '=' in line and not line.startswith('#'):
                        key, value = line.strip().split('=', 1)
                        env[key] = value

        # locate and run executable
        base_dir = os.path.join(os.path.dirname(__file__), "..", "..")
        binary_path = os.path.join(base_dir, "target", "debug", exec)
        if not os.path.exists(binary_path):
            raise FileNotFoundError(f"Binary not found: {exec}")
            
        result = subprocess.run([binary_path] + args,
                              capture_output=True,
                              text=True,
                              cwd="../..",
                              env=env)  # Pass environment variables
        
        print(result.stdout)
        if result.stderr:
            print(result.stderr)
        return result.returncode
    

        
    


    

    
    

