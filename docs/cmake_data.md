# cmake_data.yaml

> This is the documentation page for the **cmake_data.yaml** file.

`cmake_data.yaml` is the configuration file for GCMake projects. It is used to describe
the configuration elements and project metadata which cannot be inferred from the
project structure itself.

## Data Type Reference

Reference for special values in cmake_data.yaml. Most of these "special values" are
just strings which only allow certain values..

### Build Type Specifier

Name of a build configuration (case sensitive). Allowed values are:

- `Debug`
- `Release`
- `MinSizeRel`
- `RelWithDebInfo`

### Language Specifier

*Case sensitive* name of a programming language used by the project.

- `C`
- `Cpp`

### Compiler Specifier

*Case sensitive* name of a compiler which can be used with `gcmake-rust` projects.

- `MSVC`
- `GCC`
- `Clang`

### Compiler Selection Specifier

*Case sensitive* name of a compiler listed in cmake_data.yaml [supported_compilers](#supportedcompilers), or `All`.
This value is used to declare which compiler options are being configured, and is used as a map key for
[single build config configuration](#build-config-options)

- `All`
- Any single [compiler specifier](#compiler-specifier)

## General Project Information

These options are for basic project information, such as the project name, description, and include prefix.

> **TODO:** document cmake_data.yaml options for both regular projects and subprojects.

### name

**REQUIRED** `String`

Name of the project. Cannot contain spaces.

``` yaml
---
name: the-project-name
```

### include_prefix

**REQUIRED** `String`

The project's 'include prefix' directory name. Cannot contain spaces.

``` yaml
---
include_prefix: MY_INCLUDE_PREFIX
```

The include prefix directly affects the file inclusion path for a project. This is necessary for
"namespacing" a project's files directly, so that it is always clear which project a file is
being included from. That being said, it's a good idea to **make the include prefix similar to**
**the project name, so that developers can easily associate the include path with your project.**

Assuming an include_prefix `MY_INCLUDE_PREFIX`, a toplevel project's structure will look like this:

``` txt
src/
  L-- MY_INCLUDE_PREFIX/
      L-- SomeFile.cpp
include/
  L-- MY_INCLUDE_PREFIX/
      L-- SomeFile.hpp
template-impl/
  L-- MY_INCLUDE_PREFIX/
      L-- SomeFile.tpp
```

SomeFile.hpp would be included as `MY_INCLUDE_PREFIX/SomeFile.hpp`, no matter which file or project is
including it.

#### Prefix Accumulation

**A subproject's include_prefix is appended to the include_prefix of its parent project**. This is recursively
true for subprojects of a subproject.

Subproject include_prefix is specified the same way as a toplevel project. Ex:

``` yaml
---
include_prefix: SUBPROJECT_INCLUDE_PREFIX
```

For example, assuming:

- Toplevel project: `TOPLEVEL_INCLUDE_PREFIX`
- Subproject: `SUBPROJECT_INCLUDE_PREFIX`
- Nested subproject: `NESTED_INCLUDE_PREFIX`

Files would be included like so:

- Toplevel project: `TOPLEVEL_INCLUDE_PREFIX/SomeFile.hpp`
- Subproject: `TOPLEVEL_INCLUDE_PREFIX/SUBPROJECT_INCLUDE_PREFIX/SubFile.hpp`
- Nested subproject: `TOPLEVEL_INCLUDE_PREFIX/SUBPROJECT_INCLUDE_PREFIX/NESTED_INCLUDE_PREFIX/DeepFile.hpp`

And the *src*, *include*, and *template-impl* directories in those projects should contain subdirectories
matching the include directory structure of the project.

### description

**REQUIRED** `String`

A succinct text description of the project. This currently has no effect on project generation and
exists for documentation purposes only.

``` yaml
---
description: "Your project description!"
```

**TODO:** Set project description in CMakeLists.txt.

### version

**REQUIRED** `String`

The sectioned major, minor, and patch version of your project, separated by periods.
may optionally be prefixed with a *v*.

``` yaml
---
version: "1.0.1"
# Or
version: "v1.0.1"
```

### supported_compilers

**REQUIRED** `List<`[CompilerSpecifier](#compiler-specifier)`>`

The list of compilers which this project is guaranteed to support. Compiler-specific build configuration
options can only be added for compilers explicitly listed in this list.

``` yaml
--- 
supported_compilers:
  - MSVC
  - GCC
  - Clang
```

## Build Configuration

### default_build_type

**REQUIRED** [BuildTypeSpecifier](#build-type-specifier)

The project's default build type. This build type is automatically selected when the project is configured
using CMake, if a build type has not already been selected.

The default build type must be present in the **build_configs** list.

> TODO: Link build_configs section once it's written.

``` yaml
---
default_build_type: Debug
```

### build_configs

**REQUIRED** `Map<`[BuildTypeSpecifier](#build-type-specifier)`, BuildConfigMap>`

``` yaml
---
# This is important. If GCC and Clang are not listed here, then trying to set
# compiler specific options for GCC and Clang in build_configs will cause an error.
supported_compilers:
  - GCC
  - Clang
build_configs:
  Debug:
    All:
      defines:
        - DEFINED_FOR_ALL_COMPILERS="Very Nice"
    GCC:
      defines:
        - DEFINED_ONLY_FOR_GCC
      flags:
        - "-Wall"
        - "-Og"
    Clang:
      defines:
        - DEFINED_ONLY_FOR_CLANG
```

Each build configuration (Debug, Release, etc.) can define different attributes per compiler, or for any compiler
used with the configuration. The compiler must be specified using a
[CompilerSelectionSpecifier](#compiler-selection-specifier), which includes all compilers available as a normal
[CompilerSpecifier](#compiler-specifier) as well as the special value `All` (meaning the
configuration applies to all compilers).

Only compilers explicitly listed in the project's [supported_compilers](#supportedcompilers) can be used to
configure compiler-specific options. The `All` option can always be used.

Once a compiler (or `All`) is selected, these options can be set on it:

#### build_configs 'defines'

A list of defines added to the build for the given configuration. Do not write them with a leading `-D`.

``` yaml
build_configs:
  Debug:
    All:
      defines:
        - SOME_THING
        - SOME_BOOLEAN=1
        - SOME_INT=12
        - SOME_STRING="Something is defined here"
```

#### build_configs 'flags'

A list of compiler flags to be added to the build for the given configuration. Flags must be fully prefixed
(leading `-` for GCC and Clang, or leading `/` for MSVC).

``` yaml
supported_compilers: [ GCC, MSVC ]
build_configs:
  Release:
    MSVC:
      flags: [ /O2 ]
    GCC:
      flags:
        - "-O2"
```

## languages

**REQUIRED** `Map<`[LanguageSpecifier](#language-specifier)`,` [LanguageConfig](#language-configuration-options)`>`

The language configuration options for this project. Currently, both `C` and `Cpp` configurations
are required even if one of the languages isn't used.

Example:

``` yaml
---
languages:
  C:
    standard: 11
  Cpp:
    standard: 17
```

### Language Configuration Options

#### standard

**REQUIRED** `Integer`

The language standard which outputs (and files) in the project will be built with.
This sets the standard for the current project and subproject, but does not change the
standard used to compile dependencies.

``` yaml
# Allowed standards by language. Choose one
C: [99, 11, 17]
Cpp: [11, 14, 17, 20]
```

## output

> TODO: This is going to be a long section
