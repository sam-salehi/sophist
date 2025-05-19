// Called by deamon when file at abs_path is modified
// generates new embedding

use sophist::stalker::generate_and_push_embedding;

#[tokio::main]
async fn main() {
    let abs_path = std::env::args().nth(1).expect("Path to modified file must be passed");

    let error_msg = format!("Passed path {} does not exist", &abs_path);
    assert!(std::path::Path::new(&abs_path).exists(), "{}", error_msg);

    let stat = generate_and_push_embedding(&abs_path).await;
    match stat {
        Ok(_) => println!("sucessfuly inserted embeddings"),
        Err(e) => eprintln!("Encountered issue generating and pushing modified embeddings: {} ",e)
    }
}