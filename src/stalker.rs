use crate::utils;
use std::path::Path;
use crate::{embeddor::{get_embedding,generate_query_embedding}, storage};

pub async fn begin_watch() {
    let file_path = std::env::args().nth(2).unwrap();

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
    let stat = storage::insert_embedding(&abs_path, embedding);

    match stat {
        Ok(_) => println!("sucessfuly stalking {file_path}"),
        Err(e) => println!("Encountered error modifying SQL table: {e} ")
    }
}


pub fn abandon_watch() {
    let file_path = std::env::args().nth(2).unwrap();
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


pub async fn semantic_search() {
    let count: u32 = ask_for_file_count();
    let query: String = ask_for_query();

    let q_emb = match generate_query_embedding(query, count).await {
        Some(emb) => emb,
        None => {
            println!("Failed to generate embedding for query");
            return;
        }
    };

    let paths = storage::get_closest_paths(q_emb, count).unwrap();
    println!("Here are some files matching your description:"); 
    paths.iter().enumerate()
        .for_each(|(i, path)| println!("{}. {}", i + 1, path));
}

fn ask_for_file_count() -> u32 {
    loop {
        let mut count_str = String::new();
        println!("How many files would you like retrieved:");
        if let Err(e) = std::io::stdin().read_line(&mut count_str) {
            println!("Error reading input: {}", e);
            continue;
        }
        match count_str.trim().parse::<u32>() {
            Ok(count) if count > 0 => return count,
            Ok(_) => println!("Please enter a number greater than 0"),
            Err(_) => println!("Please enter a valid number"),
        }
    }
}

fn ask_for_query() -> String {
    loop {
        let mut query: String = String::new();
        println!("Enter the search query:");
        if let Err(e) = std::io::stdin().read_line(&mut query) {
            println!("Unable to retrieve query");
            continue;
        }
        return query
    }
}

