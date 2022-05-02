# Universal Archiver

Universal Archiver is a tool to easily extract well known archive files
based on their signature. The type of the file doesn't need to be specified.


## Why

Because it's annoying to learn all the tar and zip commands.

## Installation

You can install this crate using cargo.

```sh
cargo install universal-archiver
```

## Usage

```sh
USAGE:
    uarch <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    x, extract    Extracts a given file
    help       Print this message or the help of the given subcommand(s)
```

## Extract

```sh
Extracts a given file

USAGE:
    uarch extract <FILE> [OUTPUT]

ARGS:
    <FILE>      The file to extract
    <OUTPUT>    The output folder for the given file

OPTIONS:
    -h, --help    Print help information
```

## License

MIT