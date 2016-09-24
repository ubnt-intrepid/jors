extern crate jors;
extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;

const USAGE: &'static str = "
Yet another command-line JSON generator
Usage:
  jors [-a -t -y] [-m] [-p]
  jors [-a -t -y] [-m] [-p] <params>...
  jors (-h | --help)

Options:
  -h --help     Show this message.
  -p --pretty   Pretty output. 
  -a --array    Treat standard input / arguments as an array of JSON string.
  -t --toml     Treat standard input as TOML (experimental).
  -y --yaml     Treat standard input as YAML (experimental).
  -m --msgpack  Use Msgpack instead of JSON (experimental).
";

#[derive(Debug, RustcDecodable)]
struct Args {
  flag_pretty: bool,
  flag_msgpack: bool,
  flag_array: bool,
  flag_toml: bool,
  flag_yaml: bool,
  arg_params: Vec<String>,
}

fn main() {
  let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

  let mode;
  if args.flag_array {
    mode = jors::InputMode::Array;
  } else if args.flag_yaml {
    mode = jors::InputMode::Yaml;
  } else if args.flag_toml {
    mode = jors::InputMode::Toml;
  } else {
    mode = jors::InputMode::KeyVal;
  }

  let is_msgpack = args.flag_msgpack;

  let inputs = if args.arg_params.len() == 0 {
    use std::io::Read;
    let mut inputs = String::new();
    let stdin = std::io::stdin();
    stdin.lock().read_to_string(&mut inputs).unwrap();
    inputs
  } else {
    args.arg_params.join("\n")
  };

  if is_msgpack {
    use std::io::Write;

    let parsed = jors::make_msgpack(inputs, mode).unwrap_or_else(|e| {
      writeln!(&mut std::io::stderr(), "{:?}", e).unwrap();
      std::process::exit(1);
    });

    std::io::stdout().write_all(&parsed[..]).unwrap();
    return;
  }

  let json = jors::make_json(inputs, mode, args.flag_pretty).unwrap_or_else(|e| {
    use std::io::Write;
    writeln!(&mut std::io::stderr(), "{:?}", e).unwrap();
    std::process::exit(1);
  });

  println!("{}", json);
}
