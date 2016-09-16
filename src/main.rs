extern crate rustc_serialize;

use std::io::BufRead;
use rustc_serialize::json::{self,Json};

fn main() {
  let stdin = std::io::stdin();
  let parsed:Vec<_> = stdin.lock().lines().map(|line| Json::from_str(&line.unwrap()).unwrap()).collect();

  println!("{}",json::encode(&parsed).unwrap());
}
