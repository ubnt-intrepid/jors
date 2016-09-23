extern crate rustc_serialize;
extern crate toml;
extern crate yaml_rust as yaml;

use std::collections::BTreeMap;
use rustc_serialize::json::{self, Json};
use yaml::{Yaml, YamlLoader};

#[derive(Debug)]
pub enum JorsError {
  Json(json::ParserError),
  Io(std::io::Error),
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


pub fn parse_toml(input: String) -> Result<Json, JorsError> {
  toml::Parser::new(&input).parse().ok_or(JorsError::Toml).map(ToJson::to_json)
}

pub fn parse_yaml(input: String) -> Result<Json, JorsError> {
  YamlLoader::load_from_str(&input)
    .map_err(Into::into)
    .and_then(|y| y.into_iter().nth(0).ok_or("The length of document is wrong.".into()))
    .map(ToJson::to_json)
}

pub fn parse_array(lines: String) -> Result<Json, JorsError> {
  let lines: Vec<_> = lines.split("\n").collect();
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

pub fn parse_keyval(lines: String) -> Result<Json, JorsError> {
  let lines: Vec<_> = lines.split("\n").collect();
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
}

pub fn encode(parsed: Json, is_pretty: bool) -> String {
  if is_pretty {
    format!("{}", json::as_pretty_json(&parsed).indent(2))
  } else {
    format!("{}", json::as_json(&parsed))
  }
}



fn parse_rhs(s: &str) -> Result<Json, json::BuilderError> {
  if s.trim().len() == 0 {
    return Ok(Json::Null);
  }
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

#[cfg(test)]
mod test {
  use super::*;

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
}
