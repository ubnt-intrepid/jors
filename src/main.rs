extern crate rustc_serialize;
extern crate docopt;
extern crate toml;

use std::collections::BTreeMap;
use std::io::{self, BufRead, Read, Write};
use rustc_serialize::json::{self, Json};
use docopt::Docopt;

const USAGE: &'static str = "
Yet another command-line JSON generator
Usage:
  jors [-a -p -t]
  jors [-a -p -t] <params>...
  jors (-h | --help)

Options:
  -h --help     Show this message.
  -a --array    Treat inputs as an array.
  -p --pretty   Pretty output. 
  -t --toml     Treat standard input as TOML (experimental).
";

#[derive(Debug, RustcDecodable)]
struct Args {
  flag_array: bool,
  flag_pretty: bool,
  flag_toml: bool,
  arg_params: Vec<String>,
}

#[derive(Debug)]
enum JorsError {
  Json(json::ParserError),
  OutOfRange,
  Toml,
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

fn insert_nested(buf: &mut BTreeMap<String, Json>, key: &str, val: Json) -> Result<(), JorsError> {
  let keys: Vec<_> = key.split('.').map(|s| s.trim().to_owned()).collect();
  if keys.len() == 0 {
    return Err(JorsError::OutOfRange);
  }
  insert_nested_impl(buf, keys.as_slice(), val)
}

fn insert_nested_impl(buf: &mut BTreeMap<String, Json>, keys: &[String], val: Json) -> Result<(), JorsError> {
  if keys.len() <= 1 {
    buf.insert(keys[0].clone(), val);
    Ok(())
  } else {
    if keys[1].trim() == "[]" {
      // array
      let value = buf.entry(keys[0].clone()).or_insert(Json::Array(Vec::new()));
      if let Some(ref mut arr) = value.as_array_mut() {
        // FIXME: deal with: a.b.[].d.e = sstr
        if keys.len() != 2 {
          return Err(JorsError::OutOfRange);
        }
        arr.push(val);
        Ok(())
      } else {
        Err(JorsError::OutOfRange)
      }
    } else {
      // object
      let value = buf.entry(keys[0].clone()).or_insert(Json::Object(BTreeMap::new()));
      if let Some(ref mut obj) = value.as_object_mut() {
        insert_nested_impl(obj, &keys[1..], val)
      } else {
        Err(JorsError::OutOfRange)
      }
    }
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
      if parsed.len() != 2 {
        return Err(JorsError::OutOfRange);
      }

      let rhs = try!(parse_rhs(&parsed[1]));
      try!(insert_nested(&mut buf, &parsed[0], rhs));
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

trait ToJson {
  fn to_json(self) -> Json;
}

impl ToJson for toml::Value {
  fn to_json(self) -> Json {
    use toml::Value;
    match self {
      Value::Boolean(b) => Json::Boolean(b),
      Value::Integer(i) => Json::I64(i),
      Value::Float(v) => Json::F64(v),
      Value::String(s) => Json::String(s),
      Value::Datetime(dt) => Json::String(dt),
      Value::Array(arr) => Json::Array(arr.into_iter().map(ToJson::to_json).collect()),
      Value::Table(tbl) => Json::Object(tbl.into_iter().map(|(k, v)| (k, v.to_json())).collect()),
    }
  }
}

impl ToJson for toml::Table {
  fn to_json(self) -> Json {
    toml::Value::Table(self).to_json()
  }
}

fn main() {
  let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
  let is_array = args.flag_array;
  let is_pretty = args.flag_pretty;
  let is_toml = args.flag_toml;

  let stdin = std::io::stdin();

  let parsed = if is_toml {
    let mut input = String::new();
    stdin.lock().read_to_string(&mut input).unwrap();
    toml::Parser::new(&input).parse().map(ToJson::to_json).ok_or(JorsError::Toml)
  } else {
    let lines;
    if args.arg_params.len() == 0 {
      lines = stdin.lock().lines().map(|line| line.unwrap().to_owned()).collect();
    } else {
      lines = args.arg_params;
    }

    parse_input(lines, is_array)
  };

  let parsed = parsed.unwrap_or_else(|e| {
    writeln!(&mut io::stderr(), "{:?}", e).unwrap();
    std::process::exit(1);
  });

  if is_pretty {
    println!("{}", json::as_pretty_json(&parsed).indent(2));
  } else {
    println!("{}", json::as_json(&parsed));
  }
}
