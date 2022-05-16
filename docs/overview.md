# Overview

This "project overview" page is essentially the same as the [README](/README.md).
It's here to explain the basic functionality of this project and describe its common uses.

## About gcmake-rust

`gcmake-rust` aims to be an intuitive, opinionated C/C++ project configuration tool which covers
most general and common use cases.

`gcmake-rust` currently provides the ability to:

1. Create new full C/C++ projects and subprojects.
2. Generate header, source, and template-impl files in-tree.
3. Generate a full working CMake configuration for an entire project tree, including dependencies
and subprojects.

## Example Project

See the [gcmake-test-project](https://github.com/scupit/gcmake-test-project) for a full example
of a working gcmake project.

## Suggestions

See [the README](/README.md) for build information and help [getting started](/README.md#getting-started).

See [the documentation](Docs_Home.md) for in-depth information about this tool.

> It's worth aliasing `gcmake-rust` to just `gcmake`, for convenience.

## Common Uses

> **NOTE:** This section assumes the `gcmake-rust` executable is aliased to `gcmake`.

`gcmake --help` shows toplevel help info.

`gcmake <command> --help` for command-specific info.

`gcmake [path-to-project]` configures the project in the given path and writes CMake configurations for the entire
project tree (excluding subdirectories).  If no path is provided, the current working directory is used.

`gcmake new <project-name>` steps you through the project initializer prompts and creates a new C/C++ project.

`gcmake new --subproject <project-name>` checks if the current working directory is a GCMake-rust project.
If it is, then runs the same project configuration process as above and creates the project in
*subprojects/\<project-name\>*.
