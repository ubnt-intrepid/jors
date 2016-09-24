extern crate jors;
extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;
use std::io::{stdout, Write};

const USAGE: &'static str = "
Yet another command-line JSON generator
Usage:
  jors [--format=<format>] [-m] [-p]
  jors [--format=<format>] [-m] [-p] <params>...
  jors (-h | --help)

Options:
  -h --help          Show this message.
  -p --pretty        Pretty output.
  --format=<format>  Input format [default: keyval].
  -m --msgpack       Use Msgpack instead of JSON (experimental).
";

#[derive(Debug, RustcDecodable)]
struct Args {
  flag_pretty: bool,
  flag_msgpack: bool,
  arg_format: Option<String>,
  arg_params: Vec<String>,
}

fn main() {
  let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

  let mode = match args.arg_format.unwrap_or("keyval".to_owned()).as_str() {
    "array" => jors::InputMode::Array,
    "yaml" => jors::InputMode::Yaml,
    "toml" => jors::InputMode::Toml,
    "keyval" => jors::InputMode::KeyVal,
    _ => panic!("wrong input format"),
  };

  let inputs = if args.arg_params.len() == 0 {
    use std::io::Read;
    let mut inputs = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut inputs).unwrap();
    inputs
  } else {
    args.arg_params.join("\n")
  };

  let parsed = jors::make_output(inputs, mode, args.flag_msgpack, args.flag_pretty).unwrap_or_else(|e| {
    writeln!(&mut std::io::stderr(), "{:?}", e).unwrap();
    std::process::exit(1);
  });
  stdout().write_all(&parsed[..]).unwrap();
}
