# jors

Rust's implementation of command-line JSON generator, a.k.a [jo](https://github.com/jpmens/jo).

## Installation

```shell-session
$ git clone https://github.com/ys-nuem/jors.git
$ cd jors
$ cargo install
```

## Examples

```shell-session
$ jors name=jo n=17 parser=false
{"parser":false,"name":"jo","n":17}
```

```shell-session
$ cat << EOF | jors -p
name=jo
n=7
parser= false
EOF
{
  "n": 7,
  "name": "jo",
  "parser": false
}
```

Nested structure
```shell-session
$ jors a.b.c=10
{"a":{"b":{"c":10}}}

$ cat << EOF | jors
a.b.[] = 10
a.b.[] = 20
EOF
{"a":{"b":[10,20]}}
```

## License
This software is released under the MIT license.
See [LICENSE](LICENSE) for details.
