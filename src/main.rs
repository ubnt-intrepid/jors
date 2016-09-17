extern crate rustc_serialize;
extern crate docopt;

use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};
use rustc_serialize::json::{self, Json};
use docopt::Docopt;

const USAGE: &'static str = "
Yet another command-line JSON generator
Usage:
  jors [-a -p]
  jors [-a -p] <params>...
  jors (-h | --help)

Options:
  -h --help     Show this message.
  -a --array    Treats standard input as an array.
  -p --pretty   Pretty output. 
";

#[derive(Debug, RustcDecodable)]
struct Args {
  flag_array: bool,
  flag_pretty: bool,
  arg_params: Vec<String>,
}

#[derive(Debug)]
enum JorsError {
   Json(json::ParserError),
   OutOfRange,
 }
 
 impl From<json::ParserError> for JorsError {
   fn from(err: json::ParserError) -> JorsError {
     JorsError::Json(err)
   }
 }


fn parse_rhs(s: &str) -> Result<Json, json::BuilderError> {
  match Json::from_str(s) {
    Ok(val) => Ok(val),
    Err(_) => Json::from_str(&format!("\"{}\"", s)),
  }
}

fn parse_input(lines: Vec<String>, is_array: bool) -> Result<Json, JorsError> {
  if is_array == false {
    let mut buf = BTreeMap::new();
    for line in lines {
      if line.trim().is_empty() {
        continue;
      }
      let parsed: Vec<_> = line.splitn(2, '=').map(|l| l.trim().to_owned()).collect();
      let key = parsed[0].clone();
      if parsed.len() != 2 {
        return Err(JorsError::OutOfRange);
      }
      let val = try!(parse_rhs(&parsed[1]));
      buf.insert(key, val);
    }
    Ok(Json::Object(buf))
  } else {
    let mut buf = Vec::new();
    for line in lines {
      if line.trim().is_empty() {
        continue;
      }
      let val = try!(parse_rhs(&line));
      buf.push(val);
    }
    Ok(Json::Array(buf))
  }
}

#[test]
fn test1() {
  let input = r#"
10
20
"aa"
{"a":10}
"#
    .split('\n')
    .map(|m| m.to_owned())
    .collect();

  parse_input(input, true);
}

#[test]
fn test2() {
  let input = r#"
a = 10
b = 2
c = "hoge"
"#
    .split('\n')
    .map(|m| m.to_owned())
    .collect();

  parse_input(input, false);
}

#[test]
fn test3() {
  let input = r#"
"#
    .split('\n')
    .map(|m| m.to_owned())
    .collect();

  parse_input(input, true);
}

fn main() {
  let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
  let is_array = args.flag_array;
  let is_pretty = args.flag_pretty;

  let lines = if args.arg_params.len() == 0 {
    let stdin = std::io::stdin();
    let lines = stdin.lock().lines().map(|line| line.unwrap().to_owned()).collect();
    lines
  } else {
    args.arg_params
  };
  let parsed = parse_input(lines, is_array).unwrap_or_else(|e| {
    writeln!(&mut io::stderr(), "{:?}", e).unwrap();
    std::process::exit(1);
  });

  if is_pretty {
    println!("{}", json::as_pretty_json(&parsed).indent(2));
  } else {
    println!("{}", json::as_json(&parsed));
  }
}
