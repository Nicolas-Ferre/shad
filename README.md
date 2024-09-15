# shad

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Nicolas-Ferre/shad#license)
[![CI](https://github.com/Nicolas-Ferre/shad/actions/workflows/ci.yml/badge.svg)](https://github.com/Nicolas-Ferre/shad/actions/workflows/ci.yml)
[![Coverage with grcov](https://img.shields.io/codecov/c/gh/Nicolas-Ferre/shad)](https://app.codecov.io/gh/Nicolas-Ferre/shad)

Shad is a programming language to run applications almost entirely on the GPU.

It is particularly well suited for graphics applications like games.

## ⚠️ Warning ⚠️

Before considering to use this language, please keep in mind that:

- It is developed by a single person in his spare time.
- The language is very experimental, so it shouldn't be used for production applications.

## Main language features

- 🔥 Maximize execution on GPU side
- 💪 Strongly typed
- 🔀 Data race free
- 🔄 Hot reloadable

## Supported platforms

- Windows
- Linux
- macOS (limited support because the maintainer doesn't have access to a physical device)
- Android
- Web

Shad may also work on some other platforms, but they have not been tested.

## Getting started

Shad scripts can be run with the following command:

```shell
cargo run --release --bin shad -- run <script path>
```

Examples of Shad scripts are located in the `examples` folder.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE)
  or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or
conditions.
