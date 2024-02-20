# `cargo-merge-driver`

Automatically merge `Cargo.lock` files.

## Installation

To enable `cargo-merge-driver` globally, clone and run:

```sh
$ cargo install --path .
$ cargo-merge-driver install --global
```

Alternatively, it can be enabled per-repository:

```sh
$ cargo install --path .
$ cd /path/to/git/repository/
$ cargo-merge-driver install
```

In the future this may be published to crates.io, in which case the first `cargo install` commands
can be replaced with simply:
```sh
$ cargo install cargo-merge-driver
```
