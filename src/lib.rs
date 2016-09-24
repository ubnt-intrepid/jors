extern crate rustc_serialize;
extern crate toml;
extern crate yaml_rust as yaml;
extern crate rmp_serialize as msgpack;

use std::io::Read;
use std::collections::BTreeMap;
use rustc_serialize::base64::{self, ToBase64};
use rustc_serialize::json::{self, Json};
use yaml::{Yaml, YamlLoader};

#[derive(Debug)]
pub enum JorsError {
  Json(json::ParserError),
  Io(std::io::Error),
  FromUtf8(std::string::FromUtf8Error),
  Msgpack(msgpack::encode::Error),
  OutOfRange,
  Toml,
  YamlScan(yaml::ScanError),
  Other(String),
}

impl From<std::io::Error> for JorsError {
  fn from(err: std::io::Error) -> JorsError {
    JorsError::Io(err)
  }
}

impl From<std::string::FromUtf8Error> for JorsError {
  fn from(err: std::string::FromUtf8Error) -> JorsError {
    JorsError::FromUtf8(err)
  }
}

impl From<msgpack::encode::Error> for JorsError {
  fn from(err: msgpack::encode::Error) -> JorsError {
    JorsError::Msgpack(err)
  }
}

impl From<json::ParserError> for JorsError {
  fn from(err: json::ParserError) -> JorsError {
    JorsError::Json(err)
  }
}

impl From<yaml::ScanError> for JorsError {
  fn from(err: yaml::ScanError) -> JorsError {
    JorsError::YamlScan(err)
  }
}

impl<'a> From<&'a str> for JorsError {
  fn from(err: &str) -> JorsError {
    JorsError::Other(err.to_owned())
  }
}

pub enum InputMode {
  KeyVal,
  Array,
  Yaml,
  Toml,
}

pub fn make_output(input: String, mode: InputMode, is_json: bool, is_pretty: bool) -> Result<Vec<u8>, JorsError> {
  if is_json {
    make_json(input, mode, is_pretty).map(|json| Vec::from(json.as_bytes()))
  } else {
    make_msgpack(input, mode)
  }
}

fn make_json(input: String, mode: InputMode, is_pretty: bool) -> Result<String, JorsError> {
  let parsed = match mode {
    InputMode::Array => parse_array(input),
    InputMode::KeyVal => parse_keyval(input),
    InputMode::Yaml => parse_yaml(input),
    InputMode::Toml => parse_toml(input),
  };
  parsed.map(|p| self::encode(p, is_pretty))
}

fn make_msgpack(input: String, mode: InputMode) -> Result<Vec<u8>, JorsError> {
  use rustc_serialize::Encodable;

  let parsed = try!(match mode {
    InputMode::Array => parse_array(input),
    InputMode::KeyVal => parse_keyval(input),
    InputMode::Yaml => parse_yaml(input),
    InputMode::Toml => parse_toml(input),
  });

  let mut buf = Vec::new();
  parsed.encode(&mut msgpack::Encoder::new(&mut buf)).map_err(Into::into).and(Ok(buf))
}

fn parse_toml(input: String) -> Result<Json, JorsError> {
  toml::Parser::new(&input).parse().ok_or(JorsError::Toml).map(ToJson::to_json)
}

fn parse_yaml(input: String) -> Result<Json, JorsError> {
  YamlLoader::load_from_str(&input)
    .map_err(Into::into)
    .and_then(|y| y.into_iter().nth(0).ok_or("The length of document is wrong.".into()))
    .map(ToJson::to_json)
}

fn parse_array(inputs: String) -> Result<Json, JorsError> {
  let mut buf = Vec::new();

  for line in inputs.split("\n") {
    if line.trim().is_empty() {
      continue;
    }
    let val = try!(parse_rhs(&line));
    buf.push(val);
  }

  Ok(Json::Array(buf))
}

fn parse_keyval(inputs: String) -> Result<Json, JorsError> {
  let mut buf = BTreeMap::new();

  for line in inputs.split("\n") {
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
}

fn encode(parsed: Json, is_pretty: bool) -> String {
  if is_pretty {
    format!("{}", json::as_pretty_json(&parsed).indent(2))
  } else {
    format!("{}", json::as_json(&parsed))
  }
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

impl ToJson for yaml::Yaml {
  fn to_json(self) -> Json {
    match self {
      Yaml::Boolean(b) => Json::Boolean(b),
      Yaml::Integer(i) => Json::I64(i),
      Yaml::Real(s) => Json::F64(s.parse::<f64>().unwrap()),
      Yaml::String(s) => Json::String(s),
      Yaml::Null => Json::Null,
      Yaml::Array(arr) => Json::Array(arr.into_iter().map(ToJson::to_json).collect()),
      Yaml::Hash(hash) => {
        Json::Object(hash.into_iter().map(|(k, v)| (k.as_str().unwrap().to_owned(), v.to_json())).collect())
      }
      _ => panic!("bad YAML value"),
    }
  }
}

fn parse_rhs(s: &str) -> Result<Json, JorsError> {
  if s.trim().len() == 0 {
    return Ok(Json::Null);
  }
  Json::from_str(s).or({
    match s.trim().chars().nth(0) {
      Some('@') => read_file(&s.trim()[1..]).and_then(|f| String::from_utf8(f).map_err(Into::into).map(Json::String)),
      Some('%') => read_file(&s.trim()[1..]).map(|s| Json::String(s.to_base64(base64::STANDARD))),
      Some('#') => {
        read_file(&s.trim()[1..])
          .and_then(|f| String::from_utf8(f).map_err(Into::into))
          .and_then(|s| Json::from_str(&s).map_err(Into::into))
      }
      Some(_) => Json::from_str(&format!("\"{}\"", s)).map_err(Into::into),
      None => Err(JorsError::OutOfRange),
    }
  })
}

fn read_file(path: &str) -> Result<Vec<u8>, JorsError> {
  let mut buf = Vec::new();
  let mut file = try!(std::fs::File::open(path));
  try!(file.read_to_end(&mut buf));
  Ok(buf)
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


#[test]
fn test_array() {
  let input = "10\n20\n\"aa\"\n{\"a\":10}\n";
  parse_array(input.to_owned()).unwrap();
}

#[test]
fn test_keyval() {
  let input = "a = 10\nb = 2\nc = \"hoge\"\n";
  parse_keyval(input.to_owned()).unwrap();
}

#[test]
fn test_empty() {
  let input = "\n";
  parse_array(input.to_owned()).unwrap();
}
