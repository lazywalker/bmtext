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
    data: HashMap<u32, String>,
}

impl<'a> Whois {
    pub fn init() -> Self {
        let conf = Ini::load_from_file("config.ini").unwrap();
        let sec = conf.section(Some("whois")).unwrap();
        let mut data = HashMap::new();
        let mut count: u32 = 0;
        if let Ok(lines) = read_lines(sec.get("datafile").unwrap()) {
            // Consumes the iterator, returns an (Optional) String
            for line in lines {
                if let Ok(l) = line {
                    let key = l[0..7].to_string().parse::<u32>().unwrap();
                    data.insert(key, l);
                    count += 1;
                }
            }
        }

        info!("{} records loaded.", count);
        Whois { data }
    }

    pub fn query(&self, dmrid: u32) -> String {
        match self.data.get(&dmrid) {
            Some(value) => value.to_string(),
            None => String::new(),
        }
    }

    pub fn query_text(&self, dmrid: u32) -> String {
        let local = self.query(dmrid);
        if local != String::new() {
            let text: Vec<&str> = local.split("\t").collect();

            format!("ID:{}\nCall:{}\n{}\n{}", text[0], text[1], text[2], text[3])
        } else {
            format!("ID:{} not found.", dmrid)
        }
    }
}

#[test]
fn test_whois() {
    let whois = Whois::init();
    assert_eq!(
        whois.data.get(&4607133).unwrap(),
        "4607133	BD7MQB	Michael Changzhi Cai	CN"
    );

    assert_eq!(whois.query(4607177), "4607177	BD7MQB	Michael Changzhi Cai	CN");
}

#[test]
fn test_whois_query_text() {
    let whois = Whois::init();
    assert_eq!(
        whois.query_text(4607177),
        "ID:4607177\nCall:BD7MQB\nMichael Changzhi Cai\nCN"
    );
}

#[test]
fn test_whois_query_text_invalid() {
    let whois = Whois::init();

    assert_eq!(whois.query(4622), "");
}
