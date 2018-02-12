extern crate colored;
extern crate iso8601;
extern crate serde_json;

use colored::*;
use serde_json::{Error, Map, Value};
use std::collections::BTreeSet;

pub struct Formatter {
    pub no_colors: bool,
    pub no_level: bool,
    pub nested_json: bool,
}

impl Formatter {
    pub fn new() -> Formatter {
        Formatter {
            no_colors: false,
            no_level: false,
            nested_json: false,
        }
    }
    pub fn reformat_str(&self, input: &str) -> Result<String, Error> {
        let val: Value = serde_json::from_str(input)?;
        return self.format_value(val, 0);
    }

    fn format_value(&self, val: Value, depth: u32) -> Result<String, Error> {
        if self.nested_json && depth > 0 {
            return Ok(val.to_string());
        }
        let out = match val {
            Value::Number(l) => format!("{}", l),
            Value::Bool(l) => format!("{}", l),
            Value::Null => String::from("null"),
            Value::String(l) => format!("{}", l),
            Value::Array(arr) => {
                let mut buf = String::new();
                buf.push('[');
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        buf.push(' ');
                    }
                    buf.push_str(&self.format_value(item.clone(), depth + 1)?);
                }
                buf.push(']');
                buf
            }
            Value::Object(obj) => self.format_obj(obj, depth + 1)?,
        };

        Ok(out)
    }

    fn format_level(&self, level: String) -> Option<String> {
        let max_len = 5;
        let mut colorized_level = match level.to_lowercase().as_str() {
            "trace" => "TRACE".normal(),
            "debug" => "DEBUG".green(),
            "info" => " INFO".blue(),
            "warn" => " WARN".yellow(),
            "error" => "ERROR".red(),
            "fatal" => "FATAL".red(),
            _ => {
                let mut lvl_upper = level.to_uppercase();
                if level.len() > max_len {
                    lvl_upper = lvl_upper[..max_len].to_string();
                } else if level.len() < max_len {
                    lvl_upper = format!("{:>width$}", lvl_upper, width = max_len)
                }
                lvl_upper.normal()
            }
        };
        if self.no_colors {
            colorized_level = colorized_level.normal();
        }

        if colorized_level == "     ".normal() {
            return None;
        }
        Some(format!("{}: ", colorized_level))
    }

    fn format_timestamp(&self, timestamp: &str) -> String {
        if self.no_colors {
            return format!("{}", timestamp);
        }
        return format!("{}", timestamp.blue().bold());
    }

    fn format_obj(&self, obj: Map<String, Value>, depth: u32) -> Result<String, Error> {
        let mut buf = String::new();
        let mut keys = BTreeSet::new();

        for k in obj.keys() {
            keys.insert(k);
        }

        let mut has_timestamp = false;
        let mut has_log_level = false;
        let mut has_message = false;

        // Render timestamp first if present
        for prop in vec!["time", "timestamp"] {
            let key = String::from(prop);
            if keys.contains(&key) {
                let val = obj.get(&key);
                match val {
                    Some(v) => match v.clone() {
                        Value::String(date_string) => {
                            let datetime = iso8601::datetime(date_string.as_str());
                            match datetime {
                                Ok(_d) => {
                                    buf.push_str(&format!(
                                        "[{}] ",
                                        self.format_timestamp(&date_string)
                                    ));
                                    keys.remove(&key);
                                    has_timestamp = true;
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

        if !self.no_level {
            // Then the log level
            let level_key = String::from("level");
            if keys.contains(&level_key) {
                let val = obj.get(&level_key);
                match val {
                    Some(v) => match v.clone() {
                        Value::String(lvl_str) => {
                            let formatted_lvl_str = self.format_level(lvl_str);
                            match formatted_lvl_str {
                                Some(s) => {
                                    buf.push_str(&s);
                                    keys.remove(&level_key);
                                    has_log_level = true;
                                }
                                None => {}
                            }
                        }
                        _ => {}
                    },
                    None => {}
                }
            }
        }

        // Then the log message
        for prop in vec!["message", "msg"] {
            let key = String::from(prop);
            if keys.contains(&key) {
                let val = obj.get(&key);
                match val {
                    Some(v) => match v.clone() {
                        Value::String(s) => {
                            buf.push_str(&format!("{} ", s));
                            keys.remove(&key);
                            has_message = true;
                        }
                        _ => {}
                    },
                    None => {}
                }
            }
        }

        // Then render the rest of the params
        let mut param_count = 0;
        for k in keys {
            let val = obj.get(k);
            match val {
                Some(v) => {
                    param_count += 1;
                    let formatted = self.format_value(v.clone(), depth)?;
                    // buf.push_str(&format!(
                    //     "{k}={v} ",
                    //     k = k.dimmed().underline(),
                    //     v = formatted.white(),
                    // ));
                    buf.push_str(&format!("{k}={v} ", k = k, v = formatted,));
                }
                None => {}
            }
        }

        if has_timestamp || has_log_level || has_message || param_count > 0 {
            let strlen = buf.len();
            buf.truncate(strlen - 1);
        }
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn reformat_obj_one_param() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{\"a\": 17}").unwrap();
        assert_eq!(a, "a=17");
    }

    #[test]
    fn reformat_obj_multiple_params() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{\"a\": 17, \"c\": 15, \"d\": \"210\"}")
            .unwrap();
        assert_eq!(a, "a=17 c=15 d=210");
    }

    #[test]
    fn reformat_obj_with_time() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{\"time\": \"2018-01-29T00:50:43.176Z\", \"a\": 17}")
            .unwrap();
        assert_eq!(a, "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] a=17");
    }

    #[test]
    fn reformat_obj_with_time_no_colors() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        let a = fmt.reformat_str("{\"time\": \"2018-01-29T00:50:43.176Z\", \"a\": 17}")
            .unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] a=17");
    }

    #[test]
    fn reformat_obj_with_timestamp() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{\"timestamp\": \"2018-01-29T00:50:43.176Z\", \"a\": 17}")
            .unwrap();
        assert_eq!(a, "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] a=17");
    }

    #[test]
    fn reformat_obj_with_time_no_params() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{\"time\": \"2018-01-29T00:50:43.176Z\"}")
            .unwrap();
        assert_eq!(a, "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m]");
    }

    #[test]
    fn reformat_obj_with_time_and_level_trace() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"trace\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] TRACE: a=17"
        );
    }

    #[test]
    fn reformat_obj_with_time_and_level_unknown() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"unknown\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] UNKNO: a=17"
        );
    }

    #[test]
    fn reformat_obj_with_time_and_level_blank() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] a=17 level="
        );
    }

    #[test]
    fn reformat_obj_with_time_and_level_short() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"sha\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m]   SHA: a=17"
        );
    }

    #[test]
    fn reformat_obj_with_time_and_level_debug() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"debug\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[32mDEBUG\u{1b}[0m: a=17"
        );
    }

    #[test]
    fn reformat_obj_with_time_and_level_debug_no_colors() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"debug\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] DEBUG: a=17");
    }

    #[test]
    fn reformat_obj_with_time_and_level_info() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"info\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[34m INFO\u{1b}[0m: a=17"
        );
    }

    #[test]
    fn reformat_obj_with_time_and_level_warn() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"warn\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[33m WARN\u{1b}[0m: a=17"
        );
    }

    #[test]
    fn reformat_obj_with_time_and_level_error() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"error\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mERROR\u{1b}[0m: a=17"
        );
    }

    #[test]
    fn reformat_obj_with_time_and_level_fatal() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mFATAL\u{1b}[0m: a=17"
        );
    }

    #[test]
    fn reformat_obj_with_time_message_and_level() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"message\": \"it's burning\"}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mFATAL\u{1b}[0m: it's burning"
        );
    }

    #[test]
    fn reformat_obj_with_time_message_attr_and_level() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"message\": \"something is on fire!\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mFATAL\u{1b}[0m: something is on fire! a=17"
        );
    }

    #[test]
    fn reformat_obj_with_time_message_attr_and_no_level() {
        let mut fmt = super::Formatter::new();
        fmt.no_level = true;
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"message\": \"something is on fire!\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] something is on fire! a=17 level=fatal"
        );
    }

    #[test]
    fn reformat_obj_with_time_message_attr_and_no_level_nested_json() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        fmt.no_level = true;
        fmt.nested_json = true;
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] a=17 level=\"fatal\"");
    }

    #[test]
    fn reformat_obj_with_time_message_attr_and_no_level_nested_json2() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        fmt.no_level = true;
        fmt.nested_json = true;
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"a\": 17, \"nested\": {\"prop1\": 5}}",
        ).unwrap();
        assert_eq!(
            a,
            "[2018-01-29T00:50:43.176Z] a=17 level=\"fatal\" nested={\"prop1\":5}"
        );
    }

    #[test]
    fn reformat_obj_with_time_message_attrs_and_level() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"message\": \"something is on fire!\", \"a\": 17, \"b\": 18}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mFATAL\u{1b}[0m: something is on fire! a=17 b=18"
        );
    }

    #[test]
    fn reformat_null() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("null").unwrap();
        assert_eq!(a, "null");
    }

    #[test]
    fn reformat_number() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("5").unwrap();
        assert_eq!(a, "5");
    }

    #[test]
    fn reformat_string() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("\"imma string\"").unwrap();
        assert_eq!(a, "imma string");
    }
}
