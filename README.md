# gcmake-rust

`gcmake-rust` is a C/C++ project management and configuration tool.

## Documentation Links

**TODO:** Add separate docs on *cmake_data.yaml* and each helper module in *cmake/*.

## About

`gcmake-rust` aims to be an opinionated C/C++ project configuration tool which covers
most general and common use cases.

`gcmake-rust` currently provides the ability to:

1. Create new full C/C++ projects and subprojects.
2. Generate header, source, and template-impl files in-tree.
3. Generate a full working CMake configuration for an entire project tree, including dependencies
and subprojects.

## Common Uses

> This section assumes the `gcmake-rust` execuatable is aliased to `gcmake`.

`gcmake --help` shows toplevel help info.

`gcmake <command> --help` for command-specific info.

`gcmake [path-to-project]` configures the project in the given path and writes CMake configurations for the entire
project tree (excluding subdirectories).  If no path is provided, the current working directory is used.

`gcmake new <project-name>` steps you through the project initializer prompts and creates a new C/C++ project.

`gcmake new --subproject <project-name>` checks if the current working directory is a GCMake-rust project. If it is, then runs the same
project configuration process as above and creates the project in *subprojects/\<project-name\>*.
