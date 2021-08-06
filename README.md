[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
### Description
TASD-Edit is a CLI-based editing tool for [TASD](https://github.com/ViGrey/tas-replay-file-format) formatted dump files. Using a series of menus, the user can create, edit, or remove any supported packets in a file, as well as create a file from scratch.

Format version supported: **0001**

Windows and Linux are supported.

### Usage


### Building
If you wish to build from source, Rust is integrated with the `cargo` build system. To install Rust and `cargo`, just follow [these instructions](https://doc.rust-lang.org/cargo/getting-started/installation.html). Once installed, while in the project directory, run `cargo build --release` to build, or use `cargo run --release` to run directly. The built binary will be available in `./out/release/tasd-edit/`
