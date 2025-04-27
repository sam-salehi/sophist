use std::{env,process};
use sophist::{stalker, setup};


fn main() {
    
    // 
    let command = env::args().nth(1).expect("null");

    
    match command.as_str() {
        "init" => setup::init(), 
        "watch" => stalker::begin_watch(env::args().nth(2).expect("invalid_file")),
        _ => handle_invalid_command(command),
    }

}




fn handle_invalid_command(command: String) {
    println!("Recieved invalid command {command}");
    println!("Below are plausible commands {}", list_commands());
    process::exit(0);
}

fn list_commands() -> String {
    let s = String::from("\nwatch");
    s
}


// want commands to be 
// init
// watch
// search
// rm_watch