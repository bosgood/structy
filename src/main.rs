#[macro_use]
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
        .version("v0.3.0")
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
            Arg::with_name("parse_depth")
                .long("parse-depth")
                .short("d")
                .required(false)
                .takes_value(true)
                .help("Number of levels deep to parse JSON"),
        )
        .arg(
            Arg::with_name("timestamp_property")
                .long("timestamp-prop")
                .short("t")
                .required(false)
                .takes_value(true)
                .help("Property to use as a timestamp"),
        )
        .arg(
            Arg::with_name("highlight_properties")
                .long("highlight-props")
                .short("h")
                .required(false)
                .multiple(true)
                .takes_value(true)
                .help("Properties to highlight"),
        )
        .get_matches();

    let no_colors = matches.is_present("no_colors");
    let no_level = matches.is_present("no_level");
    let parse_depth_str = matches.value_of("parse_depth").unwrap_or("1");
    let parse_depth: u32 = parse_depth_str.parse().unwrap();
    let timestamp_prop = matches.value_of("timestamp_property").unwrap_or("");

    let highlight_properties_vals = values_t!(matches.values_of("highlight_properties"), String);
    let highlight_properties: Vec<String> = match highlight_properties_vals {
        Ok(props) => props.iter().map(|p| p.to_string()).collect(),
        Err(_) => vec![],
    };

    let fmt = lib::Formatter::new_with_params(
        no_colors,
        no_level,
        parse_depth,
        timestamp_prop.to_string(),
        highlight_properties,
    );

    let mut line = String::new();
    loop {
        match io::stdin().read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    process::exit(0);
                }
                match fmt.reformat_str(&line) {
                    Ok(l) => println!("{}", l),
                    Err(_) => print!("{}", line),
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
