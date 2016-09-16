extern crate rustc_serialize;

use std::io::BufRead;
use rustc_serialize::json::{self, Json};

fn main() {
  let is_array = {
    let arg = std::env::args().nth(0).unwrap_or("-a".to_owned());
    arg == "-a"
  };

  let stdin = std::io::stdin();
  if !is_array {
    for line in stdin.lock().lines() {
      let parsed: Vec<_> = line.unwrap().split('=').map(|l| l.trim().to_owned()).collect();
      assert_eq!(parsed.len(), 2);

      let key = parsed[0].clone();
      let val = Json::from_str(&parsed[1]).unwrap();

      println!("{},{}", key, json::encode(&val).unwrap());
    }
  } else {
    let parsed: Vec<_> = stdin.lock().lines().map(|line| Json::from_str(&line.unwrap()).unwrap()).collect();
    println!("{}", json::encode(&parsed).unwrap());
  }
}
