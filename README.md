# cargo-select
### Cargo subcommand to easily run targets/examples
Fuzzy match against targets and examples in current rust project.
[![asciicast](https://asciinema.org/a/QSaxIcXfjjjjyM4JGJwBZtZVS.svg)](https://asciinema.org/a/QSaxIcXfjjjjyM4JGJwBZtZVS)

```
cargo-select 

USAGE:
    cargo select [ARGS]

ARGS:
    <CARGO_COMMAND>    
    <PATTERN>          
    <CARGO_ARGS>...    

OPTIONS:
    -h, --help    Print help information
```
`cargo select run` is special-cased to `cargo run` with `--package NAME` or `--example NAME`

## Installation
### From crates.io
`cargo install cargo-select`
### From source
```
git clone https://github.com/LoipesMas/cargo-select.git
cd cargo-select
cargo install --path .
```
