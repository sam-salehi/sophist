

use crate::storage;
use std::path::Path;

pub async fn find_embedding(file_path: String) {
    let abs_path_buf = Path::new(&file_path).canonicalize().unwrap();
    let abs_path = abs_path_buf.to_string_lossy().to_string();

    println!("Adress being searched for: {}", abs_path);
    let emb = storage::get_embedding(&abs_path);
    match emb {
        Ok(v) => println!("Got embeeding {:?}",v),
        Err(e) => println!("Issue getting embedding: {}",e),
    }
}



