
use std::process::exit;

// make copies of these in embeddor.rs
pub const GENERAL_TEXT_TYPES: [&str; 6] = [
    ".txt",
    ".md",
    ".csv",
    ".json",
    ".tsv",
    ".xml"
];
pub const UNIQUE_TEXT_TYPES: [&str; 2] = [
    ".pdf",
    ".doc"
];


pub const GENERAL_IMAGE_TYPE: [&str; 3] = [
".jpg",
".jpeg",
".png",
];


pub fn validate_file(path: &str) {
    // used to validate file format 

    if !std::path::Path::new(path).exists() {
        terminate(format!("File not found at {}.",path));
    }

    let file_type = if let Some(pos) = path.find('.') {
        &path[pos..]
    } else {
        terminate(format!("File {} does not include type extension.",path));
        ""
    };

    let valid_type =  GENERAL_IMAGE_TYPE.contains(&file_type) 
    || GENERAL_TEXT_TYPES.contains(&file_type) 
    || UNIQUE_TEXT_TYPES.contains(&file_type);

    if !valid_type {
        terminate(format!("Invalid file type for {}.", path));
    }
}

fn terminate(message: String) {
    println!("{}",message);
    println!("Terminating");
    exit(0);
}

