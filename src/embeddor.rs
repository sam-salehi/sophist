use core::panic;
use std::process::Command;
use crate::utils::{self, GENERAL_TEXT_TYPES,GENERAL_IMAGE_TYPE, UNIQUE_TEXT_TYPES};
use crate::types::Embedding;


// todo refactor into constituent parts


pub async fn get_embedding(abs_path: &str) -> Option<Embedding> {
    utils::validate_file(abs_path);

    let file_type = if let Some(pos) = abs_path.find('.') {
        &abs_path[pos..]
    } else {
        ""
    };

    let embedding: Option<Embedding> = match file_type {
        ext if GENERAL_TEXT_TYPES.contains(&ext)=> get_general_text_embedding(abs_path).await,
        ".pdf" => get_pdf_embedding(abs_path).await,
        ".doc"=>get_doc_embedding(abs_path).await,
        ext if GENERAL_IMAGE_TYPE.contains(&ext) => get_general_image_embedding(abs_path).await,
        _ => None
    };

    println!("Recieved embedding: {:?}", embedding);
    embedding
}


pub async fn generate_query_embedding(query: String)-> Option<Embedding> {
    generate_jina_text_embedding(&query).await
}


async fn get_pdf_embedding(abs_path: &str) -> Option<Embedding> {
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

async fn get_general_text_embedding(abs_path: &str) ->  Option<Embedding> {
    utils::validate_file(abs_path);
    let content = std::fs::read_to_string(abs_path).ok()?;
    generate_embedding(&content).await
}

async fn get_general_image_embedding(abs_path: &str) -> Option<Embedding> {
    // handle errors here. Return Option
    match generate_jina_image_embedding(abs_path).await {
        Ok(emb) => Some(emb),
        Err(e) => {
            println!("Issue generating embeeding: \n {}",e);
            None
        }
    }

}

async fn get_doc_embedding(abs_path: &str) -> Option<Embedding> {   
    // TODO
    assert!(abs_path.contains(".doc"));
    panic!("Not yet implemented");
} 

async fn generate_embedding(content: &str) -> Option<Embedding> {
    let summary = generate_gemini_summary(content).await.ok()?;
    generate_jina_text_embedding(&summary).await
}

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

async fn generate_jina_text_embedding(content: &str) -> Option<Embedding> {
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
        .collect::<Embedding>();

    Some(embedding)
}

async fn generate_jina_image_embedding(abs_path: &str) -> Result<Embedding, Box<dyn std::error::Error>> {
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
        .collect::<Embedding>();
    Ok(embedding)
}



