# Overview

This "project overview" page is essentially the same as the [README](/README.md).
It's here to explain the basic functionality of this project and describe its common uses.

## Documentation

Documentation is found in [Docs_Home.md](./Docs_Home.md).

## Build Requirements

- A [Rust toolchain](https://www.rust-lang.org/tools/install)

## Usage Requirements

<!-- https://cmake.org/cmake/help/latest/module/ExternalProject.html#id6 (FetchContent has a lot of
      the same constraints as ExternalProject) -->
- [Git](https://git-scm.com/) **1.6.5 or higher** must be installed on the system
- [CMake](https://cmake.org/download/) **3.25** or higher

## About gcmake-rust

This project uses [CPM.cmake](https://github.com/cpm-cmake/CPM.cmake)
[v0.38.1](https://github.com/cpm-cmake/CPM.cmake/releases/tag/v0.38.1) for dependency management.

Among other things, this tool is able to:

- Generate full CMake configurations for an entire project tree.
- Generate new C/C++ projects, subprojects, and test projects.
- Generate header, source, and template-impl files in-tree.

## Example Project

See the [gcmake-test-project](/gcmake-test-project/) for a full example
of a working gcmake project.

## Suggestions

See [the README](/README.md) for [build/installation information](/README.md#installation) and
help [getting started](/README.md#getting-started).

See [the documentation](Docs_Home.md) for in-depth information about this tool.

> It's worth aliasing `gcmake-rust` to just `gcmake`, for convenience.

## Common Uses

> **NOTE:** This section assumes the `gcmake-rust` executable is aliased to `gcmake`.

`gcmake [path-to-project]` configures the project in the given path and writes CMake configurations for the entire
project tree (excluding subdirectories). **If no path is provided, the current working directory is used.**

`gcmake --help` shows toplevel help info.

`gcmake <command> --help` for command-specific info.

`gcmake dep-config update [--to-branch <branch>]` to download/update the dependency configuration repository.

`gcmake new root-project <project-name>` steps you through the project initializer prompts and creates a new C/C++ project.

`gcmake new subproject <project-name>` checks if the current working directory is a GCMake-rust project.
If it is, then the subproject configuration process runs and creates the subproject in
*subprojects/\<project-name\>* if successful.

`gcmake new test <project-name>` checks if the current working directory is a GCMake-rust project, and that
the existing project defines a [test_framework](cmake_data_config/properties/properties_list.md#test_framework).
If both are true, then the test project configuration process runs and creates the test project in
*tests/\<project-name\>* if successful.
