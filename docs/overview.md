# Overview

This "project overview" page is a summary of the motivation behind this project, and its functionality.

- For information on getting started, [see the README](/README.md).
- For documentation, see [docs/Docs_Home.md](/docs/Docs_Home.md).
- For a list of dependencies currently compatible with this project, see the [external dependency configuration repository](/gcmake-dependency-configs) and [its README](/gcmake-dependency-configs/README.md)
- For working project examples, see the [gcmake-test-project repository](/gcmake-test-project) and [its README](/gcmake-test-project/README.md). The projects in that repository are tests cases for this tool, so they should all work.

<!-- TODO: Explain project functionality -->
<!-- TODO: Explain project motivation -->

## Some Example Commands

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
