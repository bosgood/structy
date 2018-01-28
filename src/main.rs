extern crate clap;
extern crate colored;
extern crate serde_json;

use std::process;
use std::io;
use colored::*;
use serde_json::{Error, Value};
use clap::App;

fn reformat_str(blah: &str) -> Result<&'static str, serde_json::Error> {
    let val: serde_json::Value = serde_json::from_str(blah)?;
    return reformat_value(val);
}

fn reformat_value(val: serde_json::Value) -> Result<&'static str, serde_json::Error> {
    let out = match val {
        serde_json::Value::Number(l) => format!("{}", l).green(),
        serde_json::Value::Bool(l) => format!("{}", l).green(),
        serde_json::Value::Null => "null".green(),
        serde_json::Value::String(l) => format!("{}", l).green(),
        serde_json::Value::Array(arr) => {
            let mut buf = String::new();
            buf.push('[');
            for item in arr {
                buf.push_str(reformat_value(item)?);
            }
            buf.push(']');
            colored::ColoredString::from(buf.as_str())
        }
        // serde_json::Value::Object(obj) => format!("{}", obj).blue(),
        _ => "unknown".yellow(),
    };

    Ok(&format!("{}", out).as_str())
}

fn main() {
    App::new("structy")
        .about("JSON structured logging parser")
        .version("v0.1.0")
        .get_matches();

    let mut line = String::new();
    loop {
        match io::stdin().read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    process::exit(0);
                }
                match reformat_str(&line) {
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
