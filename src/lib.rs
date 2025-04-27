pub mod stalker {
    // used for watchinf files

    use crate::sql_accessor::insert_embedding;
    use std::path::Path;

    pub fn begin_watch(file_path: String) {
        // TODO
        // validate files existence.
        is_valid_file(&file_path);
        // get embedding for the file
        
        
        // watch files movement behaviour
        


        // insert_embedding
        let stat = insert_embedding(&file_path, 2);

        match stat {
            Ok(_) => println!("sucessfuly stalking {file_path}"),
            Err(e) => println!("issue with ability to watch file: {e} ")
        }

    }

    fn is_valid_file(file_path: &String) {

        if !Path::new(file_path).exists() {
            println!("Path to {file_path} not found");
            std::process::exit(1);
        }
        // TODO check to see if file is of validf format
    }
    
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

    // restart here. :)

}


mod sql_accessor {

    use uuid::Uuid;

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
                emb  INTEGER,
                path TEXT NOT NULL
            );
            COMMIT;
        ")?;
        return Ok(());
    }


    pub fn insert_embedding(address: &str, embedding: i32) -> Result<(), rusqlite::Error> {
        let conn = get_connection()?;
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO FILES (id, emb, path) VALUES (?1, ?2, ?3)",
            (id,embedding,address)
        )?;
        return Ok(());
    }


    pub fn get_embedding(address: &str) -> Result<i32, rusqlite::Error> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT emb FROM files WHERE path = ?1")?;
        let embedding: i32 = stmt.query_row(&[address], |row| row.get(0))?;
        Ok(embedding)
    }

}