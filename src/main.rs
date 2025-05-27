use std::env;
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
        "shutdown" => setup::shutdown(),
        "help" => help(),
        _ => println!("Invalid command.Run sophist help for gudiance"),
    }
}


const COMMANDS : [&str;5] = [
    "init: Setup storage. Bootup daemon. Pass API keys. Install dependencies.",
    "shutdown: Kill daemon.",
    "watch <relative_path>: Begin stalking a file. Add embedding to storage.",
    "abandon: Stop stalking file for movement. Remove its embedding from database.",
    "search: Begin generating search query.",
];


fn help() {
    println!("run 'sophist <command>' for any of the commands below");
    for command in COMMANDS.iter() {
        println!("{command}");
    }
}



