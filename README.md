# cargo-select
### Cargo subcommand to easily run targets/examples/tests
Fuzzy match against targets, examples or tests in current rust project.
[![asciicast](https://asciinema.org/a/QSaxIcXfjjjjyM4JGJwBZtZVS.svg)](https://asciinema.org/a/QSaxIcXfjjjjyM4JGJwBZtZVS)

```
cargo-select 0.2.0
LoipesMas
Fuzzy-match targets/examples

USAGE:
    cargo select [OPTIONS] [ARGS]

ARGS:
    <CARGO_COMMAND>    Cargo command to run with selected target (e.g. "run").
    <PATTERN>          Pattern to fuzzy-match targets with. Omit for interactive mode.
    <CARGO_ARGS>...    Additional arguments to pass to cargo.

OPTIONS:
    -h, --help       Print help information
        --no-skip    Run all tests that match selected test (i.e. dont skip names that are
                     supersets)(tests only)
    -V, --version    Print version information
```

`cargo select run` is special-cased to `cargo run` with `--package NAME` or `--example NAME`.

`cargo select test` is special-cased to match against test names (deduced from source files) and run them with `cargo test`.  
Alternatives:  
- You *could* just do `cargo test NAME`, but it doesn't let you find tests by name and doesnt work well with workspaces
- You *could* do a command similar to `cargo test -- --list | fzf | sed "s/: .*//" | xargs cargo test -- --exact`, but it requires compiling the tests which can take a lot of time and disk space on some projects (especially ones with a few workspaces). And it still has some limitations.

## Installation
### From crates.io
`cargo install cargo-select`
### From source
```
git clone https://github.com/LoipesMas/cargo-select.git
cd cargo-select
cargo install --path .
```
