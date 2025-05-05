pub mod stalker {

    use std::path::Path;
    use crate::{embeddor::get_embedding, sql_accessor::{self, insert_embedding}};

    pub async fn begin_watch(file_path: String) {
        is_valid_file(&file_path);
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
        let stat = insert_embedding(&abs_path, embedding);

        match stat {
            Ok(_) => println!("sucessfuly stalking {file_path}"),
            Err(e) => println!("Encountered error modifying SQL table: {e} ")
        }
    }
 

    pub fn abandon_watch(file_path: String) {
        is_valid_file(&file_path);
        let abs_path_buf = Path::new(&file_path).canonicalize().unwrap();
        let abs_path = abs_path_buf.to_string_lossy().to_string();

        if !sql_accessor::path_exists(&abs_path) {
            println!("Given path was not found in database: {}",abs_path);
            return;
        }
        sql_accessor::remove_row(&abs_path);        
        // todo stop tracking file.

    }

    fn is_valid_file(file_path: &String) {
        if !Path::new(file_path).exists() {
            println!("Path to {file_path} not found");
            std::process::exit(1);
        }
        // TODO check to see if file is of valid format
    }
    
}


pub mod embeddor {

    use core::panic;
    use std::process::Command;

    use rusqlite::types::Value;




    const GENERAL_TEXT_TYPES: [&str; 6] = [
        ".txt",
        ".md",
        ".csv",
        ".json",
        ".tsv",
        ".xml"
    ];
    const UNIQUE_TEXT_TYPES: [&str; 2] = [
        ".pdf",
        ".doc"
    ];


    const GENERAL_IMAGE_TYPE: [&str; 3] = [
    ".jpg",
    ".jpeg",
    ".png",
];

    // TODO work on return type
    pub async fn get_embedding(abs_path: &str) -> Option<Vec<f32>> {
        assert!(is_valid_file(abs_path));


        let file_type = if let Some(pos) = abs_path.find('.') {
            &abs_path[pos..]
        } else {
            ""
        };

        let embedding: Option<Vec<f32>> = match file_type {
            ext if GENERAL_TEXT_TYPES.contains(&ext)=> get_general_text_embedding(abs_path).await,
            ".pdf" => get_pdf_embedding(abs_path).await,
            ".doc"=>get_doc_embedding(abs_path).await,
            ext if GENERAL_IMAGE_TYPE.contains(&ext) => get_general_image_embedding(abs_path).await,
            _ => None
        };

        println!("Recieved embedding: {:?}", embedding);
        embedding
    }


    
    async fn get_pdf_embedding(abs_path: &str) -> Option<Vec<f32>> {
        assert!(abs_path.contains(".pdf"));
        let file_path = if let Some(pos) = abs_path.find(".") {
            &abs_path[..pos]
        } else { "" };

        let text_path = format!("{}{}",file_path,".txt");

        Command::new("pdftotext")
            .arg(abs_path)
            .arg(&text_path)
            .status()
            .expect("Failed to convert PDF to txt using pdftotext");
    
            get_general_text_embedding(&text_path).await
    }

    async fn get_general_text_embedding(abs_path: &str) ->  Option<Vec<f32>> {
        assert!(is_valid_file(&abs_path), "not valid type for general text extraction at {}", abs_path);
        let content = std::fs::read_to_string(abs_path).ok()?;
        generate_embedding(&content).await
    }

    async fn get_general_image_embedding(abs_path: &str) -> Option<Vec<f32>> {
        // handle errors here. Return Option
        match generate_jina_image_embedding(abs_path).await {
            Ok(emb) => Some(emb),
            Err(e) => {
                println!("Issue generating embeeding: \n {}",e);
                None
            }
        }

    }


    fn is_valid_file(path: &str) -> bool {
        let file_type = if let Some(pos) = path.find('.') {
            &path[pos..]
        } else {
            return false
        };

        return GENERAL_IMAGE_TYPE.contains(&file_type) 
            || GENERAL_TEXT_TYPES.contains(&file_type) 
            || UNIQUE_TEXT_TYPES.contains(&file_type)
    }


    async fn get_doc_embedding(abs_path: &str) -> Option<Vec<f32>> {   
        // TODO
        assert!(abs_path.contains(".doc"));
        panic!("Not yet implemented");
        None
    } 

    async fn generate_embedding(content: &str) -> Option<Vec<f32>> {
        let summary = generate_gemini_summary(content).await.ok()?;
        generate_jina_text_embedding(&summary).await
    }


    // could begin by asking Gemini to crate a summary of the file, 
    // and crate an embedding from the result.
    // could cut large modules in this way.

    const SUMMARY_CONTEXT: &str = "
        Please provide a summary of 3 paragraphs or 
        less for the following content, focusing on 
        whats being represented as well as the context of 
        relevance with file format. If you're not sure,
        just don't mention uncertainty:";

    async fn generate_gemini_summary(content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // fit result into embedding requirements.
        // * don't see reason to watch for token limits at this stage.

        let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set.");
        
        let client = reqwest::Client::new();
        let response = client
            .post("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent")
            .query(&[("key", api_key)])
            .json(&serde_json::json!({
                "contents": [{
                    "parts": [{
                        "text": format!("{}\n\n{}", SUMMARY_CONTEXT, content)
                    }]
                }]
            }))
            .send()
            .await
            .ok().unwrap()
            .json::<serde_json::Value>()
            .await
            .ok().unwrap();

        // Extract the summary text from response
        println!("Recived response: {}", response);

        let summary = response["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or("Failed to get summary text")?
            .to_string();
        Ok(summary)
    }

    async fn generate_jina_text_embedding(content: &str) -> Option<Vec<f32>> {
        let api_key = std::env::var("JINA_API_KEY").expect("JINA_API_KEY");
        let auth_header = format!("Bearer {}", api_key);

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.jina.ai/v1/embeddings")
            .header("Content-Type", "application/json")
            .header("Authorization", auth_header)
            .json(&serde_json::json!({
                "model": "jina-clip-v2",
                "encoding_type": "float",
                "input": content
            }))
            .send()
            .await
            .ok()?
            .json::<serde_json::Value>()
            .await
            .ok()?;
        

        let embedding = response["data"][0]["embedding"]
            .as_array()?
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect::<Vec<f32>>();

        Some(embedding)
    }

    async fn generate_jina_image_embedding(abs_path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // Check if file exists and is a supported image type

        let path = std::path::Path::new(abs_path);
        if !path.exists() {
            return Err("Image file not found: {abs_path}".into())
        }
        
        // Read the image file
        let image_data = std::fs::read(abs_path)?;
        
        // Convert to base64
        let base64_image = base64::encode(&image_data);
        
        let api_key = std::env::var("JINA_API_KEY").expect("JINA_API_KEY");
        let auth_header = format!("Bearer {}", api_key);

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.jina.ai/v1/embeddings")
            .header("Content-Type", "application/json")
            .header("Authorization", auth_header)
            .json(&serde_json::json!({
                "model": "jina-clip-v2",
                "encoding_type": "float",
                "input": [
                    {"image": base64_image}
                ]
            }))
            .send()
            .await?;

        
        let response_json = response.json::<serde_json::Value>().await?;

        // Extract embedding if successful
        let embedding = response_json["data"][0]["embedding"]
            .as_array().ok_or("Could not convert embedding to array")?
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect::<Vec<f32>>();
        Ok(embedding)
    }

}



pub mod setup {

    pub fn init() {
        // creates a simple SQL database with two columns vector and pathd
        if let Err(e) = super::sql_accessor::make_sql_table() {

            println!("Unable to create database {e:?}");
        } 
    }
}


pub mod debugger {
    // for debugging in development.
    use crate::sql_accessor;
    use std::path::Path;

    pub async fn find_embedding(file_path: String) {
        let abs_path_buf = Path::new(&file_path).canonicalize().unwrap();
        let abs_path = abs_path_buf.to_string_lossy().to_string();
 
        println!("Adress being searched for: {}", abs_path);
        let emb = sql_accessor::get_embedding(&abs_path);
        match emb {
            Ok(v) => println!("Got embeeding {:?}",v),
            Err(e) => println!("Issue getting embedding: {}",e),
        }
    }
}

mod sql_accessor {

    use uuid::Uuid;
    use serde::{Serialize, Deserialize};

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

}