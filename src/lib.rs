pub mod stalker {

    use std::path::Path;
    use crate::{embeddor::get_embedding, sql_accessor::{self, insert_embedding}};

    pub async fn begin_watch(file_path: String) {
        is_valid_file(&file_path);
        let abs_path_buf = Path::new(&file_path).canonicalize().unwrap();
        let abs_path = abs_path_buf.to_string_lossy().to_string();

        let embedding_res = get_embedding(&abs_path).await;
        
        let embedding = match embedding_res {
            Ok(Some(emb)) => emb,
            Ok(None) => {println!("unable to extract embeddings.");vec![]},
            Err(e) => {
                println!("Encountered error generating embeddings: {}",e);
                println!("Terminating process");
                std::process::exit(1)
             }, 
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

        // check if abs_path exists in database
        
        // if not exit
        if !sql_accessor::path_exists(&abs_path) {
            println!("Given path was not found in database: {}",abs_path);
            return;
        }
        // else remove it.
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

    const SUPPORTED_FILES: [&str; 6] = [
        ".txt",
        ".pdf",
        ".csv",
        ".json",
        ".tsv",
        ".xml"
    ];


    pub async fn get_embedding(abs_path: &str) -> Result<Option<Vec<f32>>, Box<dyn std::error::Error>> {
        let file_type = if let Some(pos) = abs_path.find('.') {
            &abs_path[pos..]
        } else {
            return Err("No file extension found".into());
        };


        let embedding: Option<Vec<f32>> = match file_type {
            ".txt" => get_general_embedding(abs_path).await,
            ".pdf" => get_pdf_embedding(abs_path).await,
            ".md" => get_general_embedding(abs_path).await,
            ".csv"=> get_general_embedding(abs_path).await,
            ".json"=>get_general_embedding(abs_path).await,
            ".tsv"=>get_general_embedding(abs_path).await,
            ".xml"=>get_general_embedding(abs_path).await,
            ".doc"=>get_doc_embedding(abs_path).await,
            _ => None
        };
        Ok(embedding)
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
    
            get_general_embedding(&text_path).await
    }

    async fn get_general_embedding(abs_path: &str) ->  Option<Vec<f32>> {
        assert!(valid_general_file(&abs_path), "not valid type for general text extraction at {}", abs_path);
        let content = std::fs::read_to_string(abs_path).ok()?;
        generate_embedding(&content).await
    }


    fn valid_general_file(path: &str) -> bool{
        // TODO
        // Check file to see if its of valid format.
        return true
    }


    async fn get_doc_embedding(abs_path: &str) -> Option<Vec<f32>> {   
        // TODO
        assert!(abs_path.contains(".doc"));
        panic!("Not yet implemented");
        None
    } 

    async fn generate_embedding(content: &str) -> Option<Vec<f32>> {
        let summary = generate_gemini_summary(content).await.ok()?;
        generate_gemini_embedding(&summary).await
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


    async fn generate_gemini_embedding(content: &str) -> Option<Vec<f32>> {
        let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set."); 

        let client = reqwest::Client::new();
        let response = client
            .post("https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-exp-03-07:embedContent")
            .query(&[("key", api_key)])
            .json(&serde_json::json!({
                "model": "models/gemini-embedding-exp-03-07",
                "content": {
                    "parts": [{
                        "text": content
                    }]
                }
            }))
            .send()
            .await
            .ok()?
            .json::<serde_json::Value>()
            .await
            .ok()?;


        let embedding = response["embedding"]["values"]
            .as_array()?
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect::<Vec<f32>>();
        Some(embedding)
    }

    // pub fn get_supported_files() -> [&'static str; 2] {
    //     // returns files which allow for an embeddor to be used
    //     return SUPPORTED_FILES;
    // }
}



pub mod setup {

    pub fn init() {
        // creates a simple SQL database with two columns vector and pathd
        let status = super::sql_accessor::make_sql_table();
        match status {
            Ok(_v) => println!("Was sucessful"),
            Err(e) => println!("Unable to create database {e:?}"),
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

    fn get_connection() -> Result<rusqlite::Connection,rusqlite::Error>{
        let conn = rusqlite::Connection::open("vectors.db")?;
        Ok(conn)
    }

    pub fn make_sql_table() -> Result<(),rusqlite::Error> {
        let conn = get_connection()?;
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
        let conn = get_connection()?;
        let id = Uuid::new_v4().to_string();
        let blob = serde_json::to_vec(&EmbeddingData { values: embedding }) // further error handling
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Blob,
                Box::new(e)
            ))?;
        
        conn.execute(
            "INSERT INTO FILES (id, emb, path) VALUES (?1, ?2, ?3)",
            (&id, &blob, address)
        )?;
        Ok(())
    }

    pub fn get_embedding(address: &str) -> Result<Vec<f32>, rusqlite::Error> {
        let conn = get_connection()?;
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

    // ! test below.
    pub fn path_exists(abs_path: &str) -> Result<bool,rusqlite::Error> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM FILES WHERE path = ?1")
            .expect("Failed to prepare statement");
        
        let count: i32 = stmt.query_row(&[abs_path], |row| row.get(0))
            .unwrap_or(0);
            
        Ok(count > 0)
    }

    pub fn remove_row(abs_path: &str) -> Result<(), rusqlite::Error> {
        assert!(path_exists(abs_path).unwrap());
        
        let conn = get_connection()?;
        conn.execute(
            "DELETE FROM FILES WHERE path = ?1",
            &[abs_path]
        )?;
        
        println!("Successfully removed {} from database", abs_path);
        Ok(())
    }

}