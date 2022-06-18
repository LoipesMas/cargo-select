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
