use std::{env,process};
use sophist::{debugger, setup, stalker::{begin_watch,abandon_watch,semantic_search}};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let command = env::args().nth(1).expect("null");

    let second_arg = env::args().nth(2);


    match command.as_str() {
        "init" => setup::init(), 
        "watch" => begin_watch().await,
        "abandon" => abandon_watch(),
        "search" => semantic_search().await, 
        "debug" => debugger::find_embedding(second_arg.unwrap()).await,
        _ => handle_invalid_command(command),
    }
}


const COMMANDS : [&str;5] = [
    "init: Get started with a SQL database. Pass Gemini Key. Install dependencies.",
    "watch: Begin stalking a file.",
    "debug: for debugging purposes.",
    "abandon: Stop stalking file for movement. Remove its embedding from database.",
    "search: Semantic search for file to get location."
];

fn handle_invalid_command(command: String) {
    println!("Recieved invalid command {command}");
    println!("Below are plausible commands {}", COMMANDS.join("\n"));
    process::exit(0);
}



