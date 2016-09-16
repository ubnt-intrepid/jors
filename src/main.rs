extern crate rustc_serialize;
extern crate docopt;

use std::io::BufRead;
use std::collections::HashMap;
use rustc_serialize::json::{self, Json};
use docopt::Docopt;

const USAGE: &'static str = "
Yet another command-line JSON generator
Usage:
  jors [-a]
  jors (-h | --help)

Options:
  -h --help   Show this message.
  -a --array  treats standard input as an array. 
";

#[derive(Debug, RustcDecodable)]
struct Args {
  flag_array: bool,
}

enum ParseResult {
  Array(Vec<Json>),
  Object(HashMap<String, Json>),
}

impl rustc_serialize::Encodable for ParseResult {
  fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    match self {
      &ParseResult::Array(ref arr) => arr.encode(s),
      &ParseResult::Object(ref obj) => obj.encode(s),
    }
  }
}

fn parse_rhs(s:&str) -> Json {
  Json::from_str(s).unwrap()
}

fn parse_input<R: BufRead>(reader: R, is_array: bool) -> ParseResult {
  if is_array == false {
    let mut buf = HashMap::new();
    for line in reader.lines() {
      let parsed: Vec<_> = line.unwrap().splitn(2, '=').map(|l| l.trim().to_owned()).collect();
      assert_eq!(parsed.len(), 2);
      let key = parsed[0].clone();
      let val = parse_rhs(&parsed[1]);
      buf.insert(key, val);
    }
    ParseResult::Object(buf)
  } else {
    let mut buf = Vec::new();
    for line in reader.lines() {
      let val = parse_rhs(&line.unwrap());
      buf.push(val);
    }
    ParseResult::Array(buf)
  }
}

fn main() {
  let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
  let is_array = args.flag_array;

  let stdin = std::io::stdin();
  let parsed = parse_input(stdin.lock(), is_array);
  println!("{}", json::encode(&parsed).unwrap());
}
