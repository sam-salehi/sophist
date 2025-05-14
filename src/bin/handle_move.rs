// ran when Daemon notices movement of file from one location to antoher
// args passed are src_path and dest_path/

use sophist::storage::update_path;

// TODO check behavior
fn main() {
    println!("Pinged for movement");
    let src_path = std::env::args().nth(1).expect("Source path required");
    let dst_path = std::env::args().nth(2).expect("Destination path required");
    match update_path(&src_path, &dst_path) {
        Ok(_) => println!("Updated path sucessfuly"),
        Err(e) => println!("Encountered issue upadting path from {} to {}: {}",src_path,dst_path,e),
    }
}

