use serde_json::json;
use crate::types::Embedding;
use std::io::Write;
use std::fs::File;

struct Entry  {
    name: String,
    path:String,
    vec: Embedding
}

// ! needs refactoring inits


const DATA_PATH: &str = "scripts/tracked_files.json";

pub fn make_table() -> std::io::Result<()> { 
    let data = json!({
        "data": []
    });
    let json_string = serde_json::to_string_pretty(&data).unwrap();
    let mut file = File::create(DATA_PATH)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}

pub fn insert_embedding(address: &str, embedding: Embedding) -> std::io::Result<()> {
    let file = File::open(DATA_PATH)?;
    let mut data: serde_json::Value = serde_json::from_reader(file)?;
    
    let entry = Entry {
        name: address.split('/').last().unwrap_or(address).to_string(),
        path: address.to_string(),
        vec: embedding,
    };

    let entries = data["data"].as_array_mut().unwrap();
    
    if let Some(pos) = entries.iter().position(|x| x["path"] == address) {
        entries[pos] = json!({
            "name": entry.name,
            "path": entry.path,
            "vec": entry.vec
        });
    } else {
        entries.push(json!({
            "name": entry.name,
            "path": entry.path,
            "vec": entry.vec
        }));
    }

    let mut file = File::create(DATA_PATH)?;
    file.write_all(serde_json::to_string_pretty(&data)?.as_bytes())?;
    Ok(())
}


pub fn get_embedding(address: &str) -> Result<Embedding, std::io::Error> {
    assert!(path_exists(address),"Path passed to get_embedding must exist in {}.",DATA_PATH);
    
    let file = File::open(DATA_PATH)?;
    let data: serde_json::Value = serde_json::from_reader(file)?;
    
    if let Some(entries) = data["data"].as_array() {
        if let Some(entry) = entries.iter().find(|x| x["path"] == address) {
            if let Some(vec) = entry["vec"].as_array() {
                return Ok(vec.iter()
                    .map(|v| v.as_f64().unwrap() as f32)
                    .collect());
            }
        }
    }
    
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Embedding not found"
    ))
}

pub fn path_exists(address: &str) -> bool {
    if let Ok(file) = File::open(DATA_PATH) {
        if let Ok(data) = serde_json::from_reader(file) {
            let data: serde_json::Value = data;
            if let Some(entries) = data["data"].as_array() {
                return entries.iter().any(|x| x["path"] == address);
            }
        }
    }
    false
}

pub fn remove_row(address: &str) -> std::io::Result<()> {
    assert!(path_exists(address), "Path to remove must exist in {}", DATA_PATH);
    
    let file = File::open(DATA_PATH)?;
    let mut data: serde_json::Value = serde_json::from_reader(file)?;
    
    if let Some(entries) = data["data"].as_array_mut() {
        if let Some(pos) = entries.iter().position(|x| x["path"] == address) {
            entries.remove(pos);
            let mut file = File::create(DATA_PATH)?;
            file.write_all(serde_json::to_string_pretty(&data)?.as_bytes())?;
            return Ok(());
        }
    }
    
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Entry not found"
    ))
}


struct PathSimilarity {
    path: String,
    similarity: f32,
}

pub fn get_closest_paths(query_embedding: Embedding, k: u32) -> Result<Vec<String>, std::io::Error> {
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

pub fn get_all_rows() -> std::io::Result<Vec<(String, Embedding)>> {
    let file = File::open(DATA_PATH)?;
    let data: serde_json::Value = serde_json::from_reader(file)?;
    
    if let Some(entries) = data["data"].as_array() {
        let mut results = Vec::new();
        
        for entry in entries {
            if let (Some(path), Some(vec)) = (
                entry["path"].as_str(),
                entry["vec"].as_array()
            ) {
                let embedding: Embedding = vec.iter()
                    .map(|v| v.as_f64().unwrap() as f32)
                    .collect();
                results.push((path.to_string(), embedding));
            }
        }
        
        Ok(results)
    } else {
        Ok(Vec::new())
    }
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
