

use sophist::storage::remove_row;


// get abs_path as argument
// removes file with given abs_path from sql db

fn main() {
    println!("Pinged to delete");
    let abs_path = std::env::args().nth(1).expect("Expected path to deleted file");
    remove_row(&abs_path);
}