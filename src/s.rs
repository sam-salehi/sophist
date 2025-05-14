// use crate::utils;
// use std::path::Path;
// use crate::{embeddor::{get_embedding,generate_query_embedding}, storage};
// use std::process::Command;


// pub async fn begin_watch() {
//     let file_path = std::env::args().nth(2).unwrap();

//     utils::validate_file(&file_path);
//     let abs_path_buf = Path::new(&file_path).canonicalize().unwrap();
//     let abs_path = abs_path_buf.to_string_lossy().to_string();

//     let stat = generate_and_push_embedding(&abs_path).await;

//     match stat {
//         Ok(_) => println!("sucessfuly inserted embeddings"),
//         Err(e) => println!("Encountered error modifying SQL table: {e} ")
//     }
//     // actually begin stalking
//     stalk(&abs_path);
// }


// pub async fn generate_and_push_embedding(abs_path: &str)-> Result<(),rusqlite::Error> {
//     let embedding_res = get_embedding(&abs_path).await;
    
//     let embedding = match embedding_res {
//         Some(emb) => emb,
//         None => {
//             println!("Encountered error generating embeddings:");
//             println!("Terminating process");
//             std::process::exit(1) // ! not idiomatic
//         }
//     };
//     storage::insert_embedding(&abs_path, embedding)
// }



// pub fn abandon_watch() {
//     let file_path = std::env::args().nth(2).unwrap();
//     utils::validate_file(&file_path);
//     let abs_path_buf = Path::new(&file_path).canonicalize().unwrap();
//     let abs_path = abs_path_buf.to_string_lossy().to_string();

//     if !storage::path_exists(&abs_path) {
//         println!("Given path was not found in database: {}",abs_path);
//         return;
//     }
//     storage::remove_row(&abs_path);        
//     // todo stop tracking file.
//     abandon(&abs_path);
// }


// pub async fn semantic_search() {
//     let count: u32 = ask_for_file_count();
//     let query: String = ask_for_query();

//     let q_emb = match generate_query_embedding(query).await {
//         Some(emb) => emb,
//         None => {
//             println!("Failed to generate embedding for query");
//             return;
//         }
//     };

//     let paths = storage::get_closest_paths(q_emb, count).unwrap();
//     println!("Here are some files matching your description:"); 
//     paths.iter().enumerate()
//         .for_each(|(i, path)| println!("{}. {}", i + 1, path));
// }

// fn ask_for_file_count() -> u32 {
//     loop {
//         let mut count_str = String::new();
//         println!("How many files would you like retrieved:");
//         if let Err(e) = std::io::stdin().read_line(&mut count_str) {
//             println!("Error reading input: {}", e);
//             continue;
//         }
//         match count_str.trim().parse::<u32>() {
//             Ok(count) if count > 0 => return count,
//             Ok(_) => println!("Please enter a number greater than 0"),
//             Err(_) => println!("Please enter a valid number"),
//         }
//     }
// }

// fn ask_for_query() -> String {
//     loop {
//         let mut query: String = String::new();
//         println!("Enter the search query:");
//         if let Err(e) = std::io::stdin().read_line(&mut query) {
//             println!("Unable to retrieve query");
//             continue;
//         }
//         return query
//     }
// }

// // fn stalk(abs_path: &str) {
// //     let (tx, rx) = channel();

// //     let mut watcher = RecommendedWatcher::new(tx, Config::default())
// //         .expect("Failed to create watcher");

// //     watcher.watch(Path::new(abs_path), RecursiveMode::Recursive)
// //         .expect("Failed to start watching path");

// //     println!("Started watching {}", abs_path);
    
// //     loop {
// //         match rx.recv() {
// //             Ok(event) => {
// //                 println!("Event: {:?}", event);
// //                 if let Some(paths) = event.unwrap().paths.get(0..2) {
// //                     if paths.len() == 2 {
// //                         println!("File moved from {:?} to {:?}", paths[0], paths[1]);
// //                     }
// //                 }
// //             }
// //             Err(e) => println!("Watch error: {:?}", e),
// //         }
// //     }
// // }

// // TODO move these to stalker sub-module.
// const DAEOMON_PATH: &str = "src/scripts/daemon.py";

// fn stalk(abs_path: &str) -> notify::Result<()> {
//     println!("Adding path to stalk watcher");
    
//     let status = Command::new("python3")
//         .arg(DAEOMON_PATH)
//         .arg("add")
//         .arg(abs_path)
//         .status()
//         .expect(&format!("Failed to execute Python script at {}",DAEOMON_PATH));

//     if !status.success() {
//         println!("Daemon's stalk init failed with exit code: {}", status);
//     }

//     Ok(())
// }


// fn abandon(abs_path: &str) -> notify::Result<()> {
//     println!("Abandoning file");
//     let status = Command::new("python3")
//         .arg(DAEOMON_PATH)
//         .arg("remove")
//         .arg(abs_path)
//         .status()
//         .expect(&format!("Failed to execute Python script at {}",DAEOMON_PATH));

//     if !status.success() {
//         print!("Daemon's abandon failed with exit code: {}",status);
//     }

//     Ok(())
// }