# jors

[![Build Status](https://travis-ci.org/ys-nuem/jors.svg?branch=master)](https://travis-ci.org/ys-nuem/jors)
[![Build status](https://ci.appveyor.com/api/projects/status/52admc367hopgtyr/branch/master?svg=true)](https://ci.appveyor.com/project/y-sasaki-nuem/jors/branch/master)

`jors` is a tiny command-line JSON generator, written in Rust.

This project is inspired by [jo](https://github.com/jpmens/jo).

## Features
* Dependency-free (distributed as a single binary)
* Run on many operating systems, architectures (Windows, MacOSX, Linux and so on)  
* Import external JSON from command line

## Installation

### Manual download
The prebuilt binaries can be downloaded from [here](https://github.com/ys-nuem/jors/releases).

### From source
The binary is created into `$HOME/.cargo/bin` or `%USERPROFILE%\.cargo\bin`.

```shell-session
$ git clone https://github.com/ys-nuem/jors.git
$ cd jors
$ cargo install
```

## Usage

### Synposis

```shell-session
$ jors --help
Yet another command-line JSON generator
Usage:
  jors [-a -t -y] [-p]
  jors [-a -t -y] [-p] <params>...
  jors (-h | --help)

Options:
  -h --help     Show this message.
  -p --pretty   Pretty output.
  -a --array    Treat standard input / arguments as an array of JSON string.
  -t --toml     Treat standard input as TOML (experimental).
  -y --yaml     Treat standard input as YAML (experimental).
```

### Basic usage
The behaviour of `jors` is similar to `jo`:

```shell-session
$ jors name=jo n=17 parser=false
{"parser":false,"name":"jo","n":17}
```

```shell-session
$ jors -p << EOF
name   = jors
n      = 7
parser = false
EOF
{
  "n": 7,
  "name": "jors",
  "parser": false
}
```

```shell-session
$ jors -p raw=@Cargo.toml
{
  "raw": "[package]\nname = \"jors\"\ndescription = \"Rust's implementation of command-line JSON generator, jo\"\nversion = \"0.1.0\"\nauthors = [\"Yusuke Sasaki <y_sasaki@nuem.nagoya-u.ac.jp>\"]\n\n[lib]\npath = \"src/lib.rs\"\n\n[[bin]]\nname = \"jors\"\npath = \"src/main.rs\"\n\n[dependencies]\nrustc-serialize = \"*\"\ndocopt = \"*\"\ntoml = \"*\"\nyaml-rust = \"*\""
}
$ jors -p encoded=%Cargo.toml
{
  "encoded": "W3BhY2thZ2VdCm5hbWUgPSAiam9ycyIKZGVzY3JpcHRpb24gPSAiUnVzdCdzIGltcGxlbWVudGF0aW9uIG9mIGNvbW1hbmQtbGluZSBKU09OIGdlbmVyYXRvciwgam8iCnZlcnNpb24gPSAiMC4xLjAiCmF1dGhvcnMgPSBbIll1c3VrZSBTYXNha2kgPHlfc2FzYWtpQG51ZW0ubmFnb3lhLXUuYWMuanA+Il0KCltsaWJdCnBhdGggPSAic3JjL2xpYi5ycyIKCltbYmluXV0KbmFtZSA9ICJqb3JzIgpwYXRoID0gInNyYy9tYWluLnJzIgoKW2RlcGVuZGVuY2llc10KcnVzdGMtc2VyaWFsaXplID0gIioiCmRvY29wdCA9ICIqIgp0b21sID0gIioiCnlhbWwtcnVzdCA9ICIqIg=="
}
```

### Support for nested structure
Currently, the support for nested structure is experimental.

```shell-session
$ cat rustfmt.toml | jors -t -p > rustfmt.json
$ cat rustfmt.json
{
  "max_width": 120,
  "tab_spaces": 2,
  "write_mode": "Overwrite"
}
$ jors -p json=#rustfmt.json
{
  "json": {
    "max_width": 120,
    "tab_spaces": 2,
    "write_mode": "Overwrite"
  }
}
```

```shell-session
$ jors a.b.c=10
{"a":{"b":{"c":10}}}

$ jors << EOF
a.b.[] = 10
a.b.[] = 20
EOF
{"a":{"b":[10,20]}}
```

## License
This software is released under the MIT license.
See [LICENSE](LICENSE) for details.
