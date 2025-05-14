// Called by deamon when file at abs_path is modified
// generates new embedding
// TODO make sure modified doesn't get called recklessly by deamon.

use sophist::stalker::generate_and_push_embedding;

// TODO check behaviour
#[tokio::main]
async fn main() {
    let abs_path = std::env::args().nth(1).expect("Path to modified file must be passed");

    let error_msg = format!("Passed path {} does not exist", &abs_path);
    assert!(std::path::Path::new(&abs_path).exists(), "{}", error_msg);

    generate_and_push_embedding(&abs_path).await;

