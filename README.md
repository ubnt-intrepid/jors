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
$ cat << EOF | jors
name=jo
n=7
parser= false
EOF
{"name":"jo","n":7,"parser":false}
```

## TODO
- [ ] prety printing
- [ ] support for nested data structure

## License
This software is released under the MIT license.
See [LICENSE](LICENSE) for details.
