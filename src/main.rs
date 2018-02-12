extern crate clap;
extern crate colored;
extern crate serde_json;

mod lib;

use std::process;
use std::io;
use clap::App;

fn main() {
    App::new("structy")
        .about("JSON structured logging parser")
        .version("v0.1.0")
        .get_matches();

    let fmt = lib::Formatter {};
    let mut line = String::new();

    loop {
        match io::stdin().read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    process::exit(0);
                }
                match fmt.reformat_str(&line) {
                    Ok(l) => println!("{}", l),
                    Err(error) => println!("parsing error: {}", error),
                }

                line.clear();
            }
            Err(error) => {
                println!("stdin error: {}", error);
                process::exit(1)
            }
        }
    }
}
