use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::str;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct CodeSearchStat {
    pub port: u32,
}

pub fn load(stat_path: &str) -> CodeSearchStat {
    let stat_file = File::open(stat_path).unwrap();
    let mut reader = BufReader::new(&stat_file);
    let mut input = String::new();
    reader.read_to_string(&mut input).unwrap();
    serde_json::from_str(&input).unwrap()
}
