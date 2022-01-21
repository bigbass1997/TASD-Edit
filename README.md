[![License: BSD 2-Clause](https://img.shields.io/badge/License-BSD%202--Clause-blue)](LICENSE)
### Description
TASD-Edit is a CLI-based editing tool for [TASD](https://github.com/ViGrey/TASD-File-Format) formatted dump files. Using a series of menus, the user can create or remove existing packets from a file, import/export legacy formats, or create a new file from scratch.

Highest format version supported: **0x0001**

Windows and Linux are supported.

### Usage


### Building
If you wish to build from source, for your own system, Rust is integrated with the `cargo` build system. To install Rust and `cargo`, just follow [these instructions](https://doc.rust-lang.org/cargo/getting-started/installation.html). Once installed, while in the project directory, run `cargo build --release` to build, or use `cargo run --release` to run directly. The built binary will be available in `./target/release/`

To cross-compile builds for other operating systems, you can use [rust-embedded/cross](https://github.com/rust-embedded/cross).