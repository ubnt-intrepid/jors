extern crate jors;
extern crate docopt;
extern crate rustc_serialize;

use std::io::{self, BufRead, Write};
use docopt::Docopt;

const USAGE: &'static str = "
Yet another command-line JSON generator
Usage:
  jors [-a -p -t -y]
  jors [-a -p -t -y] <params>...
  jors (-h | --help)

Options:
  -h --help     Show this message.
  -a --array    Treat inputs as an array.
  -p --pretty   Pretty output. 
  -t --toml     Treat standard input as TOML (experimental).
  -y --yaml     Treat standard input as YAML (experimental).
";

#[derive(Debug, RustcDecodable)]
struct Args {
  flag_array: bool,
  flag_pretty: bool,
  flag_toml: bool,
  flag_yaml: bool,
  arg_params: Vec<String>,
}

fn main() {
  let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
  let is_array = args.flag_array;
  let is_pretty = args.flag_pretty;
  let is_toml = args.flag_toml;
  let is_yaml = args.flag_yaml;

  let stdin = std::io::stdin();

  let parsed = if is_toml {
    jors::read_toml(stdin.lock())
  } else if is_yaml {
    jors::read_yaml(stdin.lock())
  } else {
    let lines;
    if args.arg_params.len() == 0 {
      lines = stdin.lock().lines().map(|line| line.unwrap().to_owned()).collect();
    } else {
      lines = args.arg_params;
    }

    jors::parse_lines(lines, is_array)
  };

  let parsed = parsed.unwrap_or_else(|e| {
    writeln!(&mut io::stderr(), "{:?}", e).unwrap();
    std::process::exit(1);
  });

  println!("{}", jors::encode(parsed, is_pretty));
}
