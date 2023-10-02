# gcmake-rust

`gcmake-rust` is an opinionated C/C++ project configuration tool which generates FetchContent-ready CMake configurations for an entire project tree.

Among other things, this project's features include:

- Full C/C++ project, subproject, and test project generation
- In-tree header and source file generation
- Automatic installation configuration via cpack
- Configurable [pre-build scripts](/docs/pre_build_scripts.md)
- Automatic dependency link ordering and cycle detection
- [Cargo-like "project features"](/docs/cmake_data_config/properties/features.md)
- [Unified constraint expressions](/docs/cmake_data_config/data_formats.md#constraint-specifier): A readable, less painful alternative to [CMake generator expressions](https://cmake.org/cmake/help/latest/manual/cmake-generator-expressions.7.html)
- [Conditional (optional) dependencies](/docs/cmake_data_config/linking.md#conditional-dependencies)
- Out of the box [support for Herb Sutter's cppfront](/docs/cppfront_integration.md)
- Out of the box [support for Emscripten](/docs/emscripten.md)
- Out of the box [support for CUDA](/docs/using_cuda.md)

![Example GIF](assets/gcmake-example.gif)

## Documentation

Documentation is located in [docs/Docs_Home.md](/docs/Docs_Home.md).

## General Information

This project uses [CPM.cmake](https://github.com/cpm-cmake/CPM.cmake)
[v0.38.1](https://github.com/cpm-cmake/CPM.cmake/releases/tag/v0.38.1) for dependency management.

- For **documentation**, see [docs/Docs_Home.md](/docs/Docs_Home.md).
- For a list of dependencies currently compatible with this project, see the [external dependency configuration repository](/gcmake-dependency-configs) and [its README](/gcmake-dependency-configs/README.md)
- For working project examples, see the [gcmake-test-project repository](/gcmake-test-project) and [its README](/gcmake-test-project/README.md). The projects in that repository are tests cases for this tool, so they should all work.

## Project Overview

The project overview is part of the documentation, and is found in [docs/overview.md](docs/overview.md).

## Build Requirements

- A [Rust toolchain](https://www.rust-lang.org/tools/install)
- [Git](https://git-scm.com/) (to clone the project and submodules)

## Usage Requirements

- [Git](https://git-scm.com/) **1.6.5 or higher** must be installed on the system
- [CMake](https://cmake.org/download/) **3.25 or higher**

## Installation

For common use cases, see the [project overview](docs/overview.md) docs page.

1. Clone the repository: `git clone --recurse-submodules git@github.com:scupit/gcmake-rust.git`
2. `cd` into the cloned repository.
3. Switch to the desired branch or release tag: `git checkout v1.6.5`.
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
    [dependency compatibility layer](docs/predefined_dependency_doc.md) repository which allows non-gcmake
    projects to be imported and consumed 'out of the box' by gcmake-rust.
