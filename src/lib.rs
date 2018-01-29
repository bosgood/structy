extern crate colored;
extern crate iso8601;
extern crate serde_json;

use std::collections::BTreeSet;
use colored::*;
use serde_json::{Error, Map, Value};

pub fn reformat_str(input: &str) -> Result<String, Error> {
    let val: Value = serde_json::from_str(input)?;
    return format_value(val);
}

fn format_value(val: Value) -> Result<String, Error> {
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
                buf.push_str(&format_value(item.clone())?);
            }
            buf.push(']');
            buf
            // colored::ColoredString::from(buf.as_str())
        }
        Value::Object(obj) => format_obj(obj)?,
    };

    Ok(out)
}

fn format_obj(obj: Map<String, Value>) -> Result<String, Error> {
    let mut buf = String::new();
    let mut keys = BTreeSet::new();

    for k in obj.keys() {
        keys.insert(k);
    }

    // Render timestamp first if present
    for prop in vec!["time", "timestamp", "date"] {
        let key = String::from(prop);
        if keys.contains(&key) {
            let val = obj.get(&key);
            match val {
                Some(v) => match v.clone() {
                    Value::String(date_string) => {
                        let datetime = iso8601::datetime(date_string.as_str());
                        match datetime {
                            Ok(_d) => {
                                buf.push_str(&format!("[{}] ", date_string));
                                keys.remove(&key);
                            }
                            Err(_) => {}
                        }
                    }
                    _ => {}
                },
                None => {}
            }
        }
    }

    // Then the log level
    if keys.contains(&String::from("level")) {}

    // Then add the actual log message
    // vec!["message", "msg"]

    // Then render the rest of the params
    let mut param_count = 0;
    for k in keys {
        let val = obj.get(k);
        match val {
            Some(v) => {
                param_count += 1;
                let formatted = format_value(v.clone())?;
                buf.push_str(&format!("{k}={v} ", k = k, v = formatted));
            }
            None => {}
        }
    }

    if param_count > 0 {
        let strlen = buf.len();
        buf.truncate(strlen - 1);
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reformat_obj_one_param() {
        let a = super::reformat_str("{\"a\": 17}").unwrap();
        assert_eq!(a, "a=17");
    }

    #[test]
    fn reformat_obj_multiple_params() {
        let a = super::reformat_str("{\"a\": 17, \"c\": 15, \"d\": \"210\"}").unwrap();
        assert_eq!(a, "a=17 c=15 d=210");
    }

    #[test]
    fn reformat_obj_with_time() {
        let a = super::reformat_str("{\"time\": \"2018-01-29T00:50:43.176Z\", \"a\": 17}").unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] a=17");
    }

    #[test]
    fn reformat_obj_with_time_no_params() {
        let a = super::reformat_str("{\"time\": \"2018-01-29T00:50:43.176Z\"}").unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z]");
    }

    #[test]
    fn reformat_obj_with_time_and_level_trace() {
        let a = super::reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"trace\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] TRACE: a=17");
    }

    #[test]
    fn reformat_obj_with_time_and_level_debug() {
        let a = super::reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"debug\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] DEBUG: a=17");
    }

    #[test]
    fn reformat_obj_with_time_and_level_info() {
        let a = super::reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"info\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z]  INFO: a=17");
    }

    #[test]
    fn reformat_obj_with_time_and_level_warn() {
        let a = super::reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"warn\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z]  WARN: a=17");
    }

    #[test]
    fn reformat_obj_with_time_and_level_error() {
        let a = super::reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"error\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] ERROR: a=17");
    }

    #[test]
    fn reformat_obj_with_time_and_level_fatal() {
        let a = super::reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] FATAL: a=17");
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
