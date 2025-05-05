use uuid::Uuid;
use serde::{Serialize, Deserialize};
use rusqlite;

#[derive(Serialize, Deserialize)]
struct EmbeddingData {
    values: Vec<f32>
}

fn get_connection() -> rusqlite::Connection{
    let conn = rusqlite::Connection::open("vectors.db").expect("Unable to get access to vectors.db");
    conn
}

pub fn make_sql_table() -> Result<(),rusqlite::Error> {
    let conn = get_connection();
    conn.execute_batch("
        BEGIN;
        CREATE TABLE IF NOT EXISTS FILES (
            id   TEXT PRIMARY KEY,
            emb  BLOB,
            path TEXT NOT NULL
        );
        COMMIT;
    ")?;
    Ok(())
}


pub fn insert_embedding(address: &str, embedding: Vec<f32>) -> Result<(), rusqlite::Error> {
    let conn = get_connection();
    let blob = serde_json::to_vec(&EmbeddingData { values: embedding }) 
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Blob,
            Box::new(e)
        ))?;

    if path_exists(address) {
        // Update existing row
        conn.execute(
            "UPDATE FILES SET emb = ?1 WHERE path = ?2",
            (&blob, address)
        )?;
        println!("Updated embedding for {}", address);
    } else {
        // Insert new row
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO FILES (id, emb, path) VALUES (?1, ?2, ?3)",
            (&id, &blob, address)
        )?;
        println!("Inserted new embedding for {}", address);
    }
    
    Ok(())
}

pub fn get_embedding(address: &str) -> Result<Vec<f32>, rusqlite::Error> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT emb FROM FILES WHERE path = ?1")?;
    let blob: Vec<u8> = stmt.query_row(&[address], |row| row.get(0))?;
    
    let embedding_data: EmbeddingData = serde_json::from_slice(&blob)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Blob,
            Box::new(e)
        ))?;
        
    Ok(embedding_data.values)
}


pub fn path_exists(abs_path: &str) -> bool {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM FILES WHERE path = ?1")
        .expect("Failed to prepare statement");
    
    let count: i32 = stmt.query_row(&[abs_path], |row| row.get(0))
        .unwrap_or(0);
        
    count > 0
}

pub fn remove_row(abs_path: &str) {
    assert!(path_exists(abs_path));
    
    let conn = get_connection();
    conn.execute(
        "DELETE FROM FILES WHERE path = ?1",
        &[abs_path]
    ).expect("Unable to execute SQL query.");
    
    println!("Successfully removed {} from database", abs_path);
}
