# RustGit - A Toy Git CLI Written in Rust

![License](https://img.shields.io/badge/license-MIT-blue)
[![codecov](https://codecov.io/gh/nero19960329/RustGit/graph/badge.svg?token=D2BBB05QHQ)](https://codecov.io/gh/nero19960329/RustGit)
[![CI](https://github.com/nero19960329/RustGit/actions/workflows/ci.yml/badge.svg)](https://github.com/nero19960329/RustGit/actions/workflows/ci.yml)


RustGit is a toy project developed to learn Rust by creating a simplified version of the Git CLI. This project aims to explore Rust's capabilities in handling command-line applications and manipulating data structures, similar to how Git operates. Although currently, it only includes basic functionalities, future enhancements will include more commands and performance optimizations.

## Installation

RustGit requires [Rust](https://www.rust-lang.org/tools/install) and Cargo for building and running. To get started, clone the repository and build the project using Cargo:

```bash
git clone https://github.com/nero19960329/RustGit.git
cd RustGit
cargo build --release
```

## Quick Start

To initialize a new RustGit repository, run:

```bash
cargo run -- init
```

You can explore other commands by running:

```bash
cargo run -- <command>
```

## Features

- `init`: Initialize a new repo.
- `write-tree`: Write the contents of the index to the object database as a tree.
- `cat-file`: Provide content or type and size information for repository objects.
- `hash-object`: Compute object ID and optionally creates a blob from a file.
- `check-ignore`: Check if a file is ignored by `.rgitignore`.
- `read-tree`: Read a tree object into the current index.

## References

This project was inspired by and built upon the knowledge and examples from the following resources:

- [ugit: DIY Git in Python](https://www.leshenko.net/p/ugit/)
- [Build Your Own Git by CodeCrafters](https://app.codecrafters.io/courses/git/introduction)

## License

RustGit is released under the MIT License. See the LICENSE file for more details.
