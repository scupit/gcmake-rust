# gcmake-rust

`gcmake-rust` is a C/C++ project configuration tool which acts as an abstraction layer over CMake.

## Documentation

Documentation is found in [docs/Docs_Home.md](docs/Docs_Home.md).

## About

Among other things, this tool is able to:

- Generate full CMake configurations for an entire project tree.
- Generate new C/C++ projects, subprojects, and test projects.
- Generate header, source, and template-impl files in-tree.

## Project Overview

The project overview is part of the documentation, and is found in [docs/overview.md](docs/overview.md).

## Build Requirements

- A [Rust toolchain](https://www.rust-lang.org/tools/install)

## Usage Requirements

- [Git](https://git-scm.com/) **1.6.5 or higher** must be installed on the system
- [CMake](https://cmake.org/download/) **3.24** or higher

## Installation

For common use cases, see the [project overview](docs/overview.md) docs page.

1. Clone the repository: `git clone --recurse-submodules git@github.com:scupit/gcmake-rust.git`
2. `cd` into the cloned repository.
3. Switch to the desired branch or release tag: `git checkout v1.5.5`.
4. Run `cargo install --path .` to create an optimized build and install the resulting gcmake-rust executable
  to `$HOME/.cargo/bin` (or `%USERPROFILE%\.cargo\bin` on Windows).
5. Optionally, alias `gcmake-rust` to just `gcmake`.
6. Run the executable: `gcmake-rust dep-config update` to install the
[external dependency compatibility configuration repository](docs/predefined_dependency_doc.md)

The tool is now fully installed and ready to go.

To get started, try creating a new project with `gcmake-rust new root-project 'your-project-name'`.
After stepping through the initializer, you will have a fully functioning CMake-compatible project.

## Getting Started

After [building and installing GCMake](#installation), step through the project initializer with
`gcmake-rust new root-project 'your-project-name'`.
Once it finishes, you'll have a fully working, fully CMake compatible project.

After making any change to *cmake_data.yaml* in your project, run `gcmake-rust` to regenerate
the *CMakeLists.txt* and *Config.cmake.in* files and re-run all validation checks.

## GCMake Repository Links

- [gcmake-rust](https://github.com/scupit/gcmake-rust): The gcmake C/C++ project configuration tool
- [gcmake-test-project](https://github.com/scupit/gcmake-test-project): The 'test case' project for
    gcmake-rust which also acts as its working example.
- [gcmake-dependency-configs](https://github.com/scupit/gcmake-dependency-configs): The
    [dependency compatibility layer](predefined_dependency_doc.md) repository which allows non-gcmake
    projects to be imported and consumed 'out of the box' by gcmake-rust.
