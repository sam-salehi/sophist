use uuid::Uuid;
use serde::{Serialize, Deserialize};
use rusqlite;
use crate::types::Embedding;

#[derive(Serialize, Deserialize)]
struct EmbeddingData {
    values: Embedding
}

#[derive(Debug)]
struct PathSimilarity {
    path: String,
    similarity: f32,
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


pub fn insert_embedding(address: &str, embedding: Embedding) -> Result<(), rusqlite::Error> {
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

pub fn get_embedding(address: &str) -> Result<Embedding, rusqlite::Error> {
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
    assert!(path_exists(abs_path), "Path being asked to remove does not exist in database.");
    
    let conn = get_connection();
    conn.execute(
        "DELETE FROM FILES WHERE path = ?1",
        &[abs_path]
    ).expect("Unable to execute SQL query.");
    
    println!("Successfully removed {} from database", abs_path);
}


pub fn get_all_rows() -> Result<Vec<(String, Embedding)>, rusqlite::Error> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT path, emb FROM FILES")?;
    let rows = stmt.query_map([], |row| {
        let path: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        
        let embedding_data: EmbeddingData = serde_json::from_slice(&blob)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Blob,
                Box::new(e)
            ))?;
            
        Ok((path, embedding_data.values))
    })?;
    
    rows.collect()
}




pub fn get_closest_paths(query_embedding: Embedding, k: u32) -> Result<Vec<String>, rusqlite::Error> {
    let rows = get_all_rows()?;

    let mut similarities: Vec<PathSimilarity> = rows.into_iter()
        .map(|(path, emb)| PathSimilarity {
            path,
            similarity: cosine_sim(query_embedding.clone(), emb)
        })
        .collect();

    // Sort by similarity in descending order
    similarities.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    // Take top k paths
    let paths: Vec<String> = similarities.into_iter()
        .take(k as usize)
        .map(|ps| {
            println!("Path: {}, Similarity: {:.4}", ps.path, ps.similarity);
            ps.path
        })
        .collect();

    Ok(paths)
}


fn cosine_sim(a: Embedding, b: Embedding) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must be of equal length");
    
    let dot_product: f32 = a.iter()
        .zip(b.iter())
        .map(|(x, y)| x * y)
        .sum();

    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    dot_product / (magnitude_a * magnitude_b)
}

pub fn update_path(old_path: &str, new_path: &str) -> Result<(), rusqlite::Error> {
    let conn = get_connection();
    conn.execute(
        "UPDATE FILES SET path = ?1 WHERE path = ?2",
        (&new_path, &old_path)
    )?;
    println!("Updated path in database from {} to {}", old_path, new_path);
    Ok(())
}
