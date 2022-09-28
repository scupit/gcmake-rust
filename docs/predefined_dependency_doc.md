# Predefined Dependency Compatibility Layer

This document gives a basic explanation of the configuration layer which makes non-gcmake C/C++ projects and
libraries compatible with the [gcmake-rust](https://github.com/scupit/gcmake-rust) tool.

Ideally, take a look through the
[gcmake-dependency-configs](https://github.com/scupit/gcmake-dependency-configs) repository
to quickly get an idea of how configuration works. That repo contains the full set of working
external library configurations.

## What is this "Compatibility Layer"?

The "predefined dependency compatibility layer" is the system through which configurations are
written for non-gcmake projects in order to make them compatible with the gcmake tool.

All these configurations exist in a single repository, organized by directory. Each directory
contains the configuration needed to make a single library/project useable with gcmake-rust out of
the box.

This system is beneficial because it allows us to make many dependencies/projects immediately
compatible with the gcmake-rust tool without requiring any modification to the dependencies or manual
configuration for gcmake-rust users. As a result, system libraries, other libraries installed on the
system, and libraries cloned locally into the repo can all be imported into gcmake projects using the
same universal interface.

The downside is that, oddly enough, it's easier to
[import non-gcmake projects and code](cmake_data.md#predefined-dependencies)
than it is to [import projects that use the gcmake tool](cmake_data.md/#gcmake-dependencies).
Having a gcmake package registry/index would resolve this.

## Configuration

Rather than give long, inline examples here, see the
[gcmake-dependency-configs](https://github.com/scupit/gcmake-dependency-configs) repo for the full set
of current working configurations. See the
[gcmake-test-project](https://github.com/scupit/gcmake-test-project) for working examples of using
these dependencies.

Each configuration directory can be configured using these files:

- `dep_config.yaml` (**REQUIRED**): The set of metadata which describes the dependency type and metadata
    needed by gcmake in order to properly import the project.
- `pre_load.cmake`: CMake script to be included just *before* the dependency is imported.
- `post_load.cmake`: CMake script to be included just *after* the dependency is imported. A common use case
    for this is finding and copying DLLs on Windows for libraries which are already installed on the
    system.
- `custom_populate.cmake`: This file provides the custom FetchContent population code for
    "subdirectory dependencies" which do not contain a CMakeLists.txt. "Custom population" involves
    creating usable library targets, populating include directories, and ensuring the headers for each
    target are installed correctly.
- `README.md`: Any helpful information on using the dependency, including import examples,
    installation help, caveats, etc.

> **NOTE:** `dep_config.yaml` is the only required file.

## dep_config.yaml

*dep_config.yaml* is the collection of metadata which makes non-gcmake projects compatible with the
gcmake tool.

A *dep_config.yaml* is split into sections, where each root object key begins the section for the
specified dependency type to be configured. These dependency type sections are supported:

- [`as_subdirectory`](#as_subdirectory)
- [`cmake_module`](#cmake_module)
- [`cmake_components_module`](#cmake_components_module)

However, **only one dependency type/section is required to be specified and configured**.

``` yaml
# Example dep_config.yaml
as_subdirectory:
  # ... its configuration
cmake_module:
  # ... its configuration
cmake_components_module:
  # ... its configuration
```

### as_subdirectory

This is the configuration section for dependencies which support being consumed by a CMake project
using CMake's `add_subdirectory` (and therefore [`FetchContent`](https://cmake.org/cmake/help/latest/module/FetchContent.html)).
(nlohmann_json, GLFW, etc.)

These dependencies will be cloned into your project's *dep/* directory at *CMake configure time*,
and will be built directly as a part of your project. These translate to a `FetchContent_Declare` call
in the CMakeLists.txt.

**Full example:** [gcmake-dependency-configs/nlohmann_json/dep_config.yaml](https://github.com/scupit/gcmake-dependency-configs/blob/develop/nlohmann_json/dep_config.yaml)

### cmake_module

This is the configuration for dependencies for which either
[CMake provides a pre-written 'Find Module'](https://cmake.org/cmake/help/latest/manual/cmake-modules.7.html#find-modules)
or the dependency provides its own
[CMake package config file](https://cmake.org/cmake/help/latest/manual/cmake-packages.7.html#package-configuration-file)

These dependencies are either system files/libraries (OpenGL, Threads, etc.), or other libraries which
you have installed on your system (WxWidgets, GLEW, etc.). These translate to a
`find_package(... <MODULE|CONFIG> REQUIRED )` call in the CMakeLists.txt.

**Full example:** [gcmake-dependency-configs/OpenGL/dep_config.yaml](https://github.com/scupit/gcmake-dependency-configs/blob/develop/OpenGL/dep_config.yaml)

### cmake_components_module

This is very similar to [normal cmake module configs](#cmake_module), except
the library being imported is composed of several components which may be optionally imported. See
the sample usage example in
[CMake's FindwxWidgets module page](https://cmake.org/cmake/help/latest/module/FindwxWidgets.html)
to get an idea of how this works internally.

These translate to a `find_package(... <MODULE|CONFIG> REQUIRED COMPONENTS ... )` call in CMakeLists.txt.

**Full example:** [gcmake-dependency-configs/wxWidgets/dep_config.yaml](https://github.com/scupit/gcmake-dependency-configs/blob/develop/wxWidgets/dep_config.yaml)
