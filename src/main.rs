extern crate rustc_serialize;
extern crate docopt;

use std::io::BufRead;
use std::collections::HashMap;
use rustc_serialize::json::{self, Json};
use docopt::Docopt;

const USAGE: &'static str = "
Yet another command-line JSON generator
Usage:
  jors [-a] <params>...
  jors (-h | --help)

Options:
  -h --help   Show this message.
  -a --array  treats standard input as an array. 
";

#[derive(Debug, RustcDecodable)]
struct Args {
  flag_array: bool,
  arg_params: Vec<String>,
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

fn parse_rhs(s: &str) -> Json {
  Json::from_str(s).unwrap()
}

fn parse_input(lines: Vec<String>, is_array: bool) -> ParseResult {
  if is_array == false {
    let mut buf = HashMap::new();
    for line in lines {
      let parsed: Vec<_> = line.splitn(2, '=').map(|l| l.trim().to_owned()).collect();
      assert_eq!(parsed.len(), 2);
      let key = parsed[0].clone();
      let val = parse_rhs(&parsed[1]);
      buf.insert(key, val);
    }
    ParseResult::Object(buf)
  } else {
    let mut buf = Vec::new();
    for line in lines {
      let val = parse_rhs(&line);
      buf.push(val);
    }
    ParseResult::Array(buf)
  }
}

fn main() {
  let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
  let is_array = args.flag_array;

  let parsed = if args.arg_params.len() == 0 {
    let stdin = std::io::stdin();
    let lines = stdin.lock().lines().map(|line| line.unwrap().to_owned()).collect();
    parse_input(lines, is_array)
  } else {
    parse_input(args.arg_params, is_array)
  };

  println!("{}", json::encode(&parsed).unwrap());
}
