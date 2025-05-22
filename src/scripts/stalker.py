import os
import json
import subprocess
from watchdog.events import FileSystemEventHandler # type: ignore


import sys
home_dir = os.path.expanduser("~")
log_path = os.path.join(home_dir, "Library", "Logs", "sophist.log")
error_log_path = os.path.join(home_dir, "Library", "Logs", "sophist.error.log")

# sys.stdout = open(log_path, "a")
# sys.stderr = open(error_log_path, "a")

CONFIG_FILE = os.path.join(os.path.dirname(__file__), "tracked_files.json")


class Stalker(FileSystemEventHandler):
    def __init__(self):
        self.dataAcessor = DataLoader()
        self.tracked_files = self.dataAcessor.get_stalker_data()
        print(self.tracked_files)
        print("Loaded tracked files:")
        print(f"Tracking {len(self.tracked_files)} files")

    def handle_tracked_files_change(self):
        self.dataAcessor.refresh()
        self.tracked_files = self.dataAcessor.get_stalker_data()

    def on_moved(self, event):
        src_path = event.src_path
        dest_path = event.dest_path

        if event.is_directory:
            descendants = self.get_common_descendants(src_path, dest_path)
        else:
            descendants = [(src_path, dest_path)]
    

        for src_path, dest_path in descendants:
            src_name = os.path.basename(src_path)
            dest_name = os.path.basename(dest_path) 
        
            
            if src_name in self.tracked_files and self.tracked_files[src_name] == src_path:
                print(f"Tracked file moved: {src_path} -> {dest_path}")

                if src_name == dest_name: # chekc for basename change
                    self.tracked_files[src_name] = dest_path
                else:
                    self.tracked_files[dest_name] = dest_path
                    del self.tracked_files[src_name]

                self.dataAcessor.modify_file_path(dest_name,src_path,dest_path)
    
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
        

    def on_deleted(self, event):
        if event.is_directory:
            return
            
        basename = os.path.basename(event.src_path)
        if basename in self.tracked_files:
            print(f"Removing tracked file: {event.src_path}")
            del self.tracked_files[basename]
            self.dataAcessor.delete_file(event.src_path)


class DataLoader():
    def __init__(self):
        self.data = None
        self.refresh()

    def refresh(self):
        self.data = self.load_tracked_files()

    def dump_data(self):
        with open(CONFIG_FILE, 'w') as f:
            json.dump(self.data, f, indent=2)

    def load_tracked_files(self):
        if os.path.exists(CONFIG_FILE):
            with open(CONFIG_FILE, 'r') as f:
                data = json.load(f)
                return data
        else:
            raise Exception(f"Given path: {CONFIG_FILE} does not exist")
        

    def get_stalker_data(self):
        return  {entry["name"]: entry["path"] for entry in self.data["data"]}

    
    def delete_file(self, path):
        self.data["data"] = [entry for entry in self.data["data"] if entry["path"] != path]
        self.dump_data()

    def modify_file_path(self, new_name, old_path, new_path):
        print("Modifying file")
        for entry in self.data["data"]:
            if entry["path"] == old_path:
                entry["name"] = new_name
                entry["path"] = new_path
                break
        self.dump_data()




