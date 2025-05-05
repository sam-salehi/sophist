
use crate::storage;

pub fn init() {
    // creates a simple SQL database with two columns vector and pathd
    if let Err(e) = storage::make_sql_table() {

        println!("Unable to create database {e:?}");
    } 
}