# gcmake-rust

`gcmake-rust` is a C/C++ project management and configuration tool.

## Documentation

Documentation for this project is found in [docs/Docs_Home.md](docs/Docs_Home.md).

## About

`gcmake-rust` aims to be an intuitive, opinionated C/C++ project configuration tool which covers
most general and common use cases.

`gcmake-rust` currently provides the ability to:

1. Create new full C/C++ projects and subprojects.
2. Generate header, source, and template-impl files in-tree.
3. Generate a full working CMake configuration for an entire project tree, including dependencies
and subprojects.

## Requirements

- [Git](https://git-scm.com/) must be installed on the system

## Installation/Getting Started

1. Clone the repository: `git clone --recurse-submodules git@github.com:scupit/gcmake-rust.git`
2. `cd` into the cloned repository.
3. Switch to the master branch with `git checkout master`.
4. Create an optimized build using `cargo build --release`.
5. The resulting executable will be located at *target/release/gcmake-rust.exe*.
    Make it available on your [system PATH](https://en.wikipedia.org/wiki/PATH_(variable)).
6. Optionally, alias `gcmake-rust` to just `gcmake`.
7. Create a new project with `gcmake new 'your-project-name-here'`. After stepping through the
    initializer, you now have a fully functioning gcmake (and CMake) project!

## Common Uses

> This section assumes the `gcmake-rust` executable is aliased to `gcmake`.

`gcmake --help` shows toplevel help info.

`gcmake <command> --help` for command-specific info.

`gcmake [path-to-project]` configures the project in the given path and writes CMake configurations for the entire
project tree (excluding subdirectories).  If no path is provided, the current working directory is used.

`gcmake new <project-name>` steps you through the project initializer prompts and creates a new C/C++ project.

`gcmake new --subproject <project-name>` checks if the current working directory is a GCMake-rust project. If it is, then runs the same
project configuration process as above and creates the project in *subprojects/\<project-name\>*.
