extern crate clap;
extern crate colored;
extern crate serde_json;

use colored::*;
use serde_json::{Error, Value};

pub fn reformat_str(input: &str) -> Result<String, Error> {
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
        Value::Object(obj) => {
            let mut buf = String::new();
            for k in obj.keys() {
                let val = obj.get(k);
                match val {
                    Some(v) => {
                        let formatted = reformat_value(v.clone())?;
                        buf.push_str(&format!("{k}={v} ", k = k, v = formatted));
                    }
                    None => {}
                }
            }
            buf
        }
    };

    Ok(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reformat_obj() {
        let a = super::reformat_str("{\"a\": 17}").unwrap();
        assert_eq!(a, "a=17 ");
    }

    #[test]
    fn reformat_null() {
        let a = super::reformat_str("null").unwrap();
        assert_eq!(a, "null");
    }

    #[test]
    fn reformat_number() {
        let a = super::reformat_str("5").unwrap();
        assert_eq!(a, "5");
    }

    #[test]
    fn reformat_string() {
        let a = super::reformat_str("\"imma string\"").unwrap();
        assert_eq!(a, "imma string");
    }
}
