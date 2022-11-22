# gcmake-rust Documentation Home

This is the documentation home page for `gcmake-rust`.

See [gcmake-test-project](https://github.com/scupit/gcmake-test-project) for an in-depth existing
project example. To create a new working project, see the [new project command](overview.md#common-uses).
These are great ways to get a feel for how the tool, project structure, and [cmake_data.yaml](cmake_data.md)
configuration work together.

## Table of Contents

1. [Project Overview](overview.md)
2. [Project Structure](project_structure.md)
3. [cmake_data.yaml Configuration](cmake_data_config/cmake_data.md)
4. [Predefined Dependency Compatibility Layer](predefined_dependency_doc.md)
5. [The Configuration Directory](the_configuration_directory.md)
6. [Cross Compilation](cross_compilation.md)
7. [Compiler pitfalls](pitfall_list.md)
8. [Project TODOs/Roadmap](TODO.md)

## Points of Interest

- [Compiling using Zig](compile_using_zig.md)
- [Emscripten Usage and Caveats](./emscripten.md)
- [Using CCache](./using_ccache.md)
- [Cargo-like "features"](./cmake_data_config/properties/features.md)
- [Optional/conditional dependencies](./cmake_data_config/linking.md#conditional-dependencies)
- [CppFront (*.cpp2) Support](./cppfront_integration.md)

## Important Concepts

- ["Include prefix" accumulation](cmake_data_config/subproject_config.md#include-prefix-accumulation):
  How project hierarchy affects each subproject's file inclusion prefix.
- [Output item rules and constraints](cmake_data_config/properties/output.md#general-output-rules):
  Rules dictating output type and quantity per project instance.
- [Linking](cmake_data_config/linking.md): How linking works in GCMake
- [Consuming other GCMake projects](cmake_data_config/properties/properties_list.md#gcmake_dependencies)
- [Non-GCMake dependency consumption](cmake_data_config/properties/properties_list.md#predefined_dependencies)
- [Auto-generated export header](cmake_data_config/auto_generated_export_macro_header.md)
- [Making use of pre-build scripts](pre_build_scripts.md)

## Quick Links

- [Getting started](overview.md#suggestions)
- [Creating a new project](overview.md#common-uses)
- [gcmake-test-project: an example project](/gcmake-test-project/)
- [Configuring project compilation flags and defines](cmake_data_config/properties/build_configs.md)
- [Linking to an output](cmake_data_config/properties/output.md#link)
- [Configuring additional flags and defines per output item](cmake_data_config/properties/output.md#build_config)
- [Configuring a project's feature set](cmake_data_config/properties/features.md)
- [Adding a pre-build script](cmake_data_config/properties/properties_list.md#prebuild_config)
- [Managing dependencies](cmake_data_config/properties/properties_list.md#predefined_dependencies)
- [Default config files](the_configuration_directory.md#manual-configuration) such as .gitignore, .clang-format, and .clang-tidy
- [Compiling using Zig](compile_using_zig.md)
- [Emscripten Usage and Caveats](./emscripten.md)
- [Using a custom Emscripten HTML shell file](./emscripten.md#using-a-custom-html-shell-file)
- [Using CppFront with your project](./cppfront_integration.md#using-cppfront-in-a-gcmake-project)

## GCMake Repository Links

- [gcmake-rust](https://github.com/scupit/gcmake-rust): The gcmake C/C++ project configuration tool
- [gcmake-test-project](https://github.com/scupit/gcmake-test-project): The 'test case' project for
    gcmake-rust which also acts as its working example.
- [gcmake-dependency-configs](https://github.com/scupit/gcmake-dependency-configs): The
    [dependency compatibility layer](predefined_dependency_doc.md) repository which allows non-gcmake
    projects to be imported and consumed 'out of the box' by gcmake-rust.
