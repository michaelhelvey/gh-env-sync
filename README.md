# gh-env-sync

A CLI tool for synchronizing a local TOML file containing variables and secrets with a Github
repository's environments. Many times a given codebase will have many required CI secrets and
variables for each environment, and it can get difficult and error prone to update each one
individually from the Github repository settings page, so this tool is desiged to allow you to
specify everything once via a TOML file and sync it to Github all at once.

## Getting Started

To get started, you will need a Rust toolchain for your machine.

Building:

```shell
$ cargo build --release
```

At this point, the tool will be installed into `./target/release/gh-env-sync`, and you can copy it
to wherever on your path you would like.

## Authors

- Michael Helvey

## License

MIT
