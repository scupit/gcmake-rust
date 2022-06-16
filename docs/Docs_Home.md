# gcmake-rust Documentation Home

This is the documentation home page for `gcmake-rust`.

See [gcmake-test-project](https://github.com/scupit/gcmake-test-project) for an in-depth existing
project example. To create a new working project, see the [new project command](overview.md#common-uses).
These are great ways to get a feel for how the tool, project structure, and [cmake_data.yaml](cmake_data.md)
configuration work together.

## Table of Contents

1. [Project Overview](overview.md)
2. [cmake_data.yaml Configuration](cmake_data.md)
3. [Additional Linking Explanation](linking_information.md)
4. [Predefined Dependency Compatibility Layer](predefined_dependency_doc.md)
5. [Project TODOs/Roadmap](TODO.md)

## Important Concepts

- ["Include prefix" accumulation](cmake_data.md#prefix-accumulation): How project hierarchy
    affects each subproject's file inclusion prefix.
- [Output item rules and constraints](cmake_data.md#output-rules-and-constraints): Rules dictating
    output type and quantity per project instance.
- [Link section format](cmake_data.md#linksection): Links are specified differently for compiled libraries
    than for other output types. 
- [Using other gcmake-rust projects as dependencies](cmake_data.md#gcmake-dependencies): Requires some
    extra steps at the moment.

## Quick Links

- [Getting started](/README.md#getting-started)
- [Creating a new project](overview.md#common-uses)
- [gcmake-test-project: an example project](https://github.com/scupit/gcmake-test-project)
- [Configuring project compilation flags and defines](cmake_data.md#build-configuration)
- [Linking to an output](cmake_data.md#output-link)
- [Configuring additional flags and defines per output item](cmake_data.md#output-buildconfig)
- [Adding a pre-build script](cmake_data.md#pre-build-script)
- [Managing dependencies](cmake_data.md#using-dependencies)
- [Subprojects (nested projects)](cmake_data.md#subprojects)

## GCMake Repository Links

- [gcmake-rust](https://github.com/scupit/gcmake-rust): The gcmake C/C++ project configuration tool
- [gcmake-test-project](https://github.com/scupit/gcmake-test-project): The 'test case' project for
    gcmake-rust which also acts as its working example.
- [gcmake-dependency-configs](https://github.com/scupit/gcmake-dependency-configs): The
    [dependency compatibility layer](predefined_dependency_doc.md) repository which allows non-gcmake
    projects to be imported and consumed 'out of the box' by gcmake-rust.
