use ini::Ini;
use log::{debug, error, info};

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub struct Whois {
    index: HashMap<String, u32>,
    data: HashMap<u32, String>,
}

impl<'a> Whois {
    pub fn init() -> Self {
        let conf = Ini::load_from_file("config.ini").unwrap();
        let sec = conf.section(Some("whois")).unwrap();
        let mut data = HashMap::new();
        let mut index = HashMap::new();
        let mut count: u32 = 0;
        if let Ok(lines) = read_lines(sec.get("datafile").unwrap()) {
            for line in lines {
                if let Ok(l) = line {
                    let v: Vec<&str> = l.split("\t").collect();
                    let key_id = v[0].to_string();
                    let key_callsign = v[1].to_string();

                    index.insert(key_id, count);
                    index.insert(key_callsign, count);
                    data.insert(count, l);

                    count += 1;
                }
            }
        }

        info!("{} records loaded.", count);
        Whois { index, data }
    }

    pub fn query(&self, key: &str) -> String {
        match self.index.get(key) {
            Some(i) => match self.data.get(i) {
                Some(value) => value.to_string(),
                None => String::new(),
            },
            None => String::new(),
        }
    }

    pub fn query_text(&self, key: &str) -> String {
        let local = self.query(key);
        if local != String::new() {
            let text: Vec<&str> = local.split("\t").collect();

            format!("ID:{}\nCall:{}\n{}\n{}", text[0], text[1], text[2], text[3])
        } else {
            String::from("ID Unknown")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whois_query() {
        let whois = Whois::init();
        assert_eq!(
            whois.query("4607177"),
            "4607177	BD7MQB	Michael Changzhi Cai	CN"
        );
    }

    #[test]
    fn test_whois_query_text() {
        let whois = Whois::init();
        assert_eq!(
            whois.query_text("4607177"),
            whois.query_text("BD7MQB")
        );
    }

    #[test]
    fn test_whois_query_text_invalid() {
        let whois = Whois::init();

        assert_eq!(whois.query("4622"), "");
    }
}
