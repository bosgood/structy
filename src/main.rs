extern crate clap;
extern crate colored;
extern crate serde_json;

mod lib;

use clap::{App, Arg};
use std::io;
use std::process;

fn main() {
    let matches = App::new("structy")
        .about("JSON structured logging parser")
        .version("v0.1.0")
        .arg(
            Arg::with_name("no_colors")
                .long("no-colors")
                .short("n")
                .required(false)
                .help("Disable colorization"),
        )
        .arg(
            Arg::with_name("no_level")
                .long("no-level")
                .short("l")
                .required(false)
                .help("Disable log level highlighting"),
        )
        .arg(
            Arg::with_name("nested_json")
                .long("nested-json")
                .short("j")
                .required(false)
                .help("Output all sub-properties as JSON"),
        )
        // .arg(
        //     Arg::with_name("timestamp_property")
        //         .long("timestamp-prop")
        //         .required(false)
        //         .help("Property to use as a timestamp"),
        // )
        // .arg(
        //     Arg::with_name("highlight_properties")
        //         .long("highlight-props")
        //         .short("h")
        //         .required(false)
        //         .multiple(true)
        //         .help("Properties to highlight"),
        // )
        .get_matches();

    let no_colors = matches.is_present("no_colors");
    let no_level = matches.is_present("no_level");
    let nested_json = matches.is_present("nested_json");
    let fmt = lib::Formatter {
        no_colors: no_colors,
        no_level: no_level,
        nested_json: nested_json,
    };
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
