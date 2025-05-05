
use crate::utils;
use std::path::Path;
use crate::{embeddor::get_embedding, storage::{self, insert_embedding}};

pub async fn begin_watch(file_path: String) {
    utils::validate_file(&file_path);
    let abs_path_buf = Path::new(&file_path).canonicalize().unwrap();
    let abs_path = abs_path_buf.to_string_lossy().to_string();

    let embedding_res = get_embedding(&abs_path).await;
    
    let embedding = match embedding_res {
        Some(emb) => emb,
        None => {
            println!("Encountered error generating embeddings:");
            println!("Terminating process");
            std::process::exit(1)
        }
    };
    let stat = insert_embedding(&abs_path, embedding);

    match stat {
        Ok(_) => println!("sucessfuly stalking {file_path}"),
        Err(e) => println!("Encountered error modifying SQL table: {e} ")
    }
}


pub fn abandon_watch(file_path: String) {
    utils::validate_file(&file_path);
    let abs_path_buf = Path::new(&file_path).canonicalize().unwrap();
    let abs_path = abs_path_buf.to_string_lossy().to_string();

    if !storage::path_exists(&abs_path) {
        println!("Given path was not found in database: {}",abs_path);
        return;
    }
    storage::remove_row(&abs_path);        
    // todo stop tracking file.
}
