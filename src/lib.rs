extern crate colored;
extern crate iso8601;
extern crate serde_json;

use colored::*;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::hash_map;
mod logfmt;


#[derive(Clone)]
pub struct Formatter {
    pub no_colors: bool,
    pub no_level: bool,
    pub parse_depth: u32,
    pub timestamp_prop: String,
    pub highlight_properties: Vec<String>,
    highlight_properties_set: BTreeSet<String>,
}

impl Formatter {
    #[test]
    pub fn new() -> Formatter {
        Formatter {
            no_colors: false,
            no_level: false,
            parse_depth: 1,
            timestamp_prop: "".to_string(),
            highlight_properties: vec![],
            highlight_properties_set: BTreeSet::new(),
        }
    }

    pub fn new_with_params(
        no_colors: bool,
        no_level: bool,
        parse_depth: u32,
        timestamp_prop: String,
        highlight_properties: Vec<String>,
    ) -> Formatter {
        let prop_set: BTreeSet<_> = highlight_properties.iter().map(|p| p.to_string()).collect();
        Formatter {
            no_colors: no_colors,
            no_level: no_level,
            parse_depth: parse_depth,
            timestamp_prop: timestamp_prop,
            highlight_properties: highlight_properties,
            highlight_properties_set: prop_set,
        }
    }

    pub fn reformat_str(&self, input: &str) -> Option<String> {
        match serde_json::from_str(input) {
            Ok(val) => {
                let v: serde_json::Value = val;
                let fmt_clone = self.clone();
                let s: String = v.format(fmt_clone, 0);
                Some(s)
            },
            Err(err) => self.reformat_space_sep_str(input)
        }
    }

    fn reformat_space_sep_str(&self, input: &str) -> Option<String> {
        let mut h = serde_json::Map::new();
        for pair in input.split_whitespace() {
            println!("{}", pair);
            // h.insert(String::from(pair), serde_json::Value::String(String::from("")));
        }
        let fmt_clone = self.clone();
        Some(h.format(fmt_clone, 0))
    }

    fn format_level(&self, level: &str) -> Option<String> {
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
            return timestamp.to_string();
        }
        return timestamp.blue().bold().to_string();
    }

    fn colorize_obj_key(&self, key: &str) -> String {
        if self.no_colors {
            return key.to_string();
        }
        if self.highlight_properties_set.contains(&key.to_string()) {
            return key.yellow().underline().to_string();
        }
        return key.dimmed().underline().to_string();
    }

    fn colorize_obj_value(&self, val: &str) -> String {
        if self.no_colors {
            return val.to_string();
        }
        return val.white().to_string();
    }

    fn timestamp_props(&self) -> Vec<&str> {
        if self.timestamp_prop != "" {
            return vec![&self.timestamp_prop];
        }
        vec!["time", "timestamp"]
    }
}

trait Formattable {
    fn format(&self, fmt: Formatter, depth: u32) -> String;
}

impl Formattable for serde_json::Map<String, serde_json::Value> {
    fn format(&self, fmt: Formatter, depth: u32) -> String {
        let mut buf = String::new();
        let mut keys = BTreeSet::new();

        for k in self.keys() {
            keys.insert(k);
        }

        let mut has_timestamp = false;
        let mut has_log_level = false;
        let mut has_message = false;

        // Render timestamp first if present
        for prop in fmt.timestamp_props() {
            let key = String::from(prop);
            if keys.contains(&key) {
                let val = self.get(&key);
                match val {
                    Some(v) => match v.clone() {
                        serde_json::Value::String(date_string) => {
                            let datetime = iso8601::datetime(date_string.as_str());
                            match datetime {
                                Ok(_d) => {
                                    buf.push_str(&format!(
                                        "[{}] ",
                                        fmt.format_timestamp(&date_string)
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

        if !fmt.no_level {
            // Then the log level
            let level_key = String::from("level");
            if keys.contains(&level_key) {
                let val = self.get(&level_key);
                match val {
                    Some(v) => match v.clone() {
                        serde_json::Value::String(lvl_str) => {
                            let formatted_lvl_str = fmt.format_level(&lvl_str);
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
                let val = self.get(&key);
                match val {
                    Some(v) => match v.clone() {
                        serde_json::Value::String(s) => {
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
            let val = self.get(k);
            match val {
                Some(v) => {
                    param_count += 1;
                    let formatted = v.clone().format(fmt.clone(), depth);
                    buf.push_str(&format!(
                        "{k}={v} ",
                        k = fmt.colorize_obj_key(k),
                        v = fmt.colorize_obj_value(&formatted),
                    ));
                }
                None => {}
            }
        }

        if has_timestamp || has_log_level || has_message || param_count > 0 {
            let strlen = buf.len();
            buf.truncate(strlen - 1);
        }
        buf
    }
}

impl Formattable for serde_json::Value {
    fn format(&self, fmt: Formatter, depth: u32) -> String {
        if depth >= fmt.parse_depth {
            return self.to_string();
        }
        match *self {
            serde_json::Value::Number(ref l) => l.to_string(),
            serde_json::Value::Bool(ref l) => l.to_string(),
            serde_json::Value::Null => String::from("null"),
            serde_json::Value::String(ref l) => l.to_string(),
            serde_json::Value::Array(ref arr) => {
                let values = arr.iter()
                    .map(|item| item.format(fmt.clone(), depth + 1))
                    .collect::<Vec<String>>();
                format!("[{}]", values.join(", "))
            }
            serde_json::Value::Object(ref obj) => obj.format(fmt.clone(), depth + 1),
        }
    }
}

impl Formattable for String {
    fn format(&self, fmt: Formatter, depth: u32) -> String {
        self.clone()
    }
}

// trait Hashlike {
//     fn get_string(&self, key: &str) -> Option<&String>;
//     // fn keys<O>(&self) -> O
//     //     where O: Iterator<Item=String>;
//     fn keys(&self) -> Box<impl Iterator<Item=String>>;
// }

// impl Hashlike for serde_json::Map<String, serde_json::Value> {
//     fn get_string(&self, key: &str) -> Option<&String> {
//         match self.get(key) {
//             Some(v) => match v {
//                 &serde_json::Value::String(ref s) => return Some(s),
//                 _ => None,
//             },
//             _ => None,
//         }
//     }
//     fn keys(&self) -> Box<serde_json::map::Keys> {
//         return Box::new(self.keys());
//     }
// }

// impl Hashlike for HashMap<String, String> {
//     fn get_string(&self, key: &str) -> Option<&String> {
//         self.get(key)
//     }
//     fn keys(&self) -> Box<hash_map::Keys<String, String>> {
//         return Box::new(self.keys());
//     }
// }

// // Render timestamp first if present
// for prop in &fmt.timestamp_props() {
//     let key = String::from(prop);
//     if keys.contains(&key) {
//         let val = self.get_string(&key);
//         match val {
//             Some(date_str) => {
//                 let datetime = iso8601::datetime(date_str.as_str());
//                 match datetime {
//                     Ok(_d) => {
//                         buf.push_str(&format!("[{}] ", fmt.format_timestamp(&date_str)));
//                         keys.remove(&key);
//                         has_timestamp = true;
//                     }
//                     Err(_) => {}
//                 }
//             }
//             None => {}
//         }
//     }
// }

// impl Formattable for Hashlike {

#[cfg(test)]
mod tests {
    #[test]
    fn reformat_obj_one_param() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        let a = fmt.reformat_str("{\"a\": 17}").unwrap();
        assert_eq!(a, "a=17");

        let b = fmt.reformat_str("a=17").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_one_param_color() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{\"a\": 17}").unwrap();
        assert_eq!(a, "\u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m");

        let b = fmt.reformat_str("a=17").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_multiple_params() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        let a = fmt.reformat_str("{\"a\": 17, \"c\": 15, \"d\": \"210\"}")
            .unwrap();
        assert_eq!(a, "a=17 c=15 d=\"210\"");

        let b = fmt.reformat_str("a=17 c=15 d=\"210\"").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_multiple_params_color() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{\"a\": 17, \"c\": 15, \"d\": \"210\"}")
            .unwrap();
        assert_eq!(a, "\u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m \u{1b}[2;4mc\u{1b}[0m=\u{1b}[37m15\u{1b}[0m \u{1b}[2;4md\u{1b}[0m=\u{1b}[37m\"210\"\u{1b}[0m");

        let b = fmt.reformat_str("a=17 c=15 d=\"210\"").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_multiple_params_parse_depth_2() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        fmt.parse_depth = 2;
        let a = fmt.reformat_str("{\"a\": 17, \"c\": 15, \"d\": \"210\"}")
            .unwrap();
        assert_eq!(a, "a=17 c=15 d=210");
    }

    #[test]
    fn reformat_obj_with_time() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        let a = fmt.reformat_str("{\"time\": \"2018-01-29T00:50:43.176Z\", \"a\": 17}")
            .unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] a=17");

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_color() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{\"time\": \"2018-01-29T00:50:43.176Z\", \"a\": 17}")
            .unwrap();
        assert_eq!(a, "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m");

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_custom() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        fmt.timestamp_prop = "custom_timestamp".to_string();
        let a = fmt.reformat_str("{\"custom_timestamp\": \"2018-01-29T00:50:43.176Z\", \"a\": 17}")
            .unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] a=17");

        let b = fmt.reformat_str("custom_timestamp=\"2018-01-29T00:50:43.176Z\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_and_custom() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        fmt.timestamp_prop = "custom_timestamp".to_string();
        let a = fmt.reformat_str("{\"custom_timestamp\": \"2018-01-29T00:50:43.500Z\", \"time\": \"2018-01-29T00:50:43.176Z\", \"a\": 17}")
            .unwrap();
        assert_eq!(
            a,
            "[2018-01-29T00:50:43.500Z] a=17 time=\"2018-01-29T00:50:43.176Z\""
        );

        let b = fmt.reformat_str(
            "custom_timestamp=\"2018-01-29T00:50:43.500Z\" time=\"2018-01-29T00:50:43.176Z\" a=17",
        ).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_no_colors() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        let a = fmt.reformat_str("{\"time\": \"2018-01-29T00:50:43.176Z\", \"a\": 17}")
            .unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] a=17");

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_timestamp() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        let a = fmt.reformat_str("{\"timestamp\": \"2018-01-29T00:50:43.176Z\", \"a\": 17}")
            .unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] a=17");

        let b = fmt.reformat_str("timestamp=\"2018-01-29T00:50:43.176Z\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_no_params() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{\"time\": \"2018-01-29T00:50:43.176Z\"}")
            .unwrap();
        assert_eq!(a, "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m]");

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\"")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_and_level_trace() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"trace\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] TRACE: \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=trace a=17")
            .unwrap();
        assert_eq!(a, b);

        let c = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"trace\" a=17")
            .unwrap();
        assert_eq!(a, c);
    }

    #[test]
    fn reformat_obj_with_time_and_level_unknown() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"unknown\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] UNKNO: \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=unknown a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_and_level_blank() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] a=17 level=\"\"");

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_and_level_short() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"sha\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m]   SHA: \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"sha\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_and_level_debug() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"debug\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[32mDEBUG\u{1b}[0m: \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"debug\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_and_level_debug_no_colors() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"debug\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] DEBUG: a=17");

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"debug\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_and_level_info() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"info\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[34m INFO\u{1b}[0m: \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"info\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_and_level_warn() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"warn\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[33m WARN\u{1b}[0m: \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"warn\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_and_level_error() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"error\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mERROR\u{1b}[0m: \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"error\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_and_level_fatal() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mFATAL\u{1b}[0m: \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"fatal\" a=17")
            .unwrap();
        assert_eq!(a, b);
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

        let b = fmt.reformat_str(
            "time=\"2018-01-29T00:50:43.176Z\" level=\"fatal\" message=\"it's burning\"",
        ).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_message_attr_and_level() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"message\": \"something is on fire!\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mFATAL\u{1b}[0m: something is on fire! \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m"
        );

        let b = fmt.reformat_str(
            "time=\"2018-01-29T00:50:43.176Z\" level=\"fatal\" message=\"something is on fire!\" a=17",
        ).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_message_attr_and_no_level() {
        let mut fmt = super::Formatter::new();
        fmt.no_level = true;
        fmt.no_colors = true;
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"message\": \"something is on fire!\", \"a\": 17}",
        ).unwrap();
        assert_eq!(
            a,
            "[2018-01-29T00:50:43.176Z] something is on fire! a=17 level=\"fatal\""
        );

        let b = fmt.reformat_str(
            "time=\"2018-01-29T00:50:43.176Z\" level=\"fatal\" message=\"something is on fire!\"",
        ).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_message_attr_and_no_level_nested_json() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        fmt.no_level = true;
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"a\": 17}",
        ).unwrap();
        assert_eq!(a, "[2018-01-29T00:50:43.176Z] a=17 level=\"fatal\"");

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"fatal\" a=17")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_message_attr_and_no_level_nested_json2() {
        let mut fmt = super::Formatter::new();
        fmt.no_colors = true;
        fmt.no_level = true;
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
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mFATAL\u{1b}[0m: something is on fire! \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m \u{1b}[2;4mb\u{1b}[0m=\u{1b}[37m18\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"fatal\" message=\"something is on fire!\" a=17 b=18")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_message_attrs_and_level_highlight_property() {
        let mut fmt = super::Formatter::new();
        fmt.highlight_properties_set = super::BTreeSet::new();
        fmt.highlight_properties_set.insert("b".to_string());
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"message\": \"something is on fire!\", \"a\": 17, \"b\": 18}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mFATAL\u{1b}[0m: something is on fire! \u{1b}[2;4ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m \u{1b}[4;33mb\u{1b}[0m=\u{1b}[37m18\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"fatal\" message=\"something is on fire!\" a=17 b=18")
            .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reformat_obj_with_time_message_attrs_and_level_highlight_properties() {
        let mut fmt = super::Formatter::new();
        fmt.highlight_properties_set = super::BTreeSet::new();
        fmt.highlight_properties_set.insert("a".to_string());
        fmt.highlight_properties_set.insert("b".to_string());
        let a = fmt.reformat_str(
            "{\"time\": \"2018-01-29T00:50:43.176Z\", \"level\": \"fatal\", \"message\": \"something is on fire!\", \"a\": 17, \"b\": 18}",
        ).unwrap();
        assert_eq!(
            a,
            "[\u{1b}[1;34m2018-01-29T00:50:43.176Z\u{1b}[0m] \u{1b}[31mFATAL\u{1b}[0m: something is on fire! \u{1b}[4;33ma\u{1b}[0m=\u{1b}[37m17\u{1b}[0m \u{1b}[4;33mb\u{1b}[0m=\u{1b}[37m18\u{1b}[0m"
        );

        let b = fmt.reformat_str("time=\"2018-01-29T00:50:43.176Z\" level=\"fatal\" message=\"something is on fire!\" a=17 b=18")
            .unwrap();
        assert_eq!(a, b);
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

    #[test]
    fn reformat_unparsable_string() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{");
        assert!(a.is_none());
    }

    #[test]
    fn reformat_obj_with_malformed_json() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("{\"time\": \"2018-01-29T00:50:43.176Z\" \"a\": 17}");
        assert!(a.is_none())
    }

    #[test]
    fn reformat_array() {
        let fmt = super::Formatter::new();
        let a = fmt.reformat_str("[\"value1\", 1, 2, 3, \"value2\"]")
            .unwrap();
        assert_eq!(a, "[\"value1\", 1, 2, 3, \"value2\"]");
    }
}
