extern crate clap;
extern crate colored;
extern crate serde_json;

use std::process;
use std::io;
use colored::*;
use serde_json::{Error, Value};
use clap::App;

fn reformat_str(input: &str) -> Result<String, Error> {
    let val: Value = serde_json::from_str(input)?;
    return reformat_value(val);
}

fn reformat_value(val: Value) -> Result<String, Error> {
    let out = match val {
        Value::Number(l) => format!("{}", l), //.green(),
        Value::Bool(l) => format!("{}", l),   //.green(),
        Value::Null => String::from("null"),  //.green(),
        Value::String(l) => format!("{}", l), //.green(),
        Value::Array(arr) => {
            let mut buf = String::new();

            buf.push('[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    buf.push(' ');
                }
                buf.push_str(&reformat_value(item.clone())?);
            }
            buf.push(']');
            buf
            // colored::ColoredString::from(buf.as_str())
        }
        // Value::Object(obj) => format!("{}", obj).blue(),
        _ => String::from("unknown"), //.yellow(),
    };

    Ok(out)
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
