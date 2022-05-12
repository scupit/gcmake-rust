# cmake_data.yaml

> This is the documentation page for the **cmake_data.yaml** file.

`cmake_data.yaml` is the configuration file for GCMake projects. It is used to describe
the configuration elements and project metadata which cannot be inferred from the
project structure itself.

See the [GCMake Test Project](https://github.com/scupit/gcmake-test-project) for a fully working
*cmake_data.yaml* example.

## General Project Information

These options are for basic project information, such as the project name, description, and include prefix.

> **TODO:** document cmake_data.yaml options for both regular projects and subprojects.

### name

> **REQUIRED** `String`

Name of the project. Cannot contain spaces.

``` yaml
---
name: the-project-name
```

### include_prefix

> **REQUIRED** `String`

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

> **REQUIRED** `String`

A succinct text description of the project. This currently has no effect on project generation and
exists for documentation purposes only.

``` yaml
---
description: "Your project description!"
```

**TODO:** Set project description in CMakeLists.txt.

### version

> **REQUIRED** `String`

The sectioned major, minor, and patch version of your project, separated by periods.
may optionally be prefixed with a *v*.

``` yaml
---
version: "1.0.1"
# Or
version: "v1.0.1"
```

### supported_compilers

> **REQUIRED** `List<`[CompilerSpecifier](#compiler-specifier)`>`

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

> **REQUIRED** [BuildTypeSpecifier](#build-type-specifier)

The project's default build type. This build type is automatically selected when the project is configured
using CMake, if a build type has not already been selected.

The default build type must be present in the **build_configs** list.

> TODO: Link build_configs section once it's written.

``` yaml
---
default_build_type: Debug
```

### build_configs

> **REQUIRED** `Map<`[BuildTypeSpecifier](#build-type-specifier)`, BuildConfigMap>`

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

> **REQUIRED** `Map<`[LanguageSpecifier](#language-specifier)`,` [LanguageConfig](#language-configuration-options)`>`

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

> **REQUIRED** `Integer`

The language standard which outputs (and files) in the project will be built with.
This sets the standard for the current project and subproject, but does not change the
standard used to compile dependencies.

``` yaml
# Allowed standards by language. Choose one
C: [99, 11, 17]
Cpp: [11, 14, 17, 20]
```

## output

> **REQUIRED** `Map<String,` [OutputItemConfig](#TODO)`>`

The compiled outputs to be produced by the project, mapped by name.

``` yaml
# Executable project example
# ---------------------------------------- 
# Produces two executables: my-exe and another-exe
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
  another:
    output_type: Executable
    entry_file: another.cpp
```

``` yaml
# Library project example
# ---------------------------------------- 
# Produces a library called very-useful.
# Explanations on the different library output_type options are provided later in this section.
output:
  my-awesome-library:
    output_type: Library
    # output_type: SharedLib
    # output_type: StaticLib
    entry_file: main.cpp
```

### Output Rules and Constraints

**A single project may either produce a single library or any number of executables, not both.**
However, subprojects are not required to produce the same output type as their parent project.

> This forces projects to be "modularized" into subprojects, and enforces 'separation of concerns'
> on the project level. As a result, this structure rule also eliminates inter-project circular dependencies.

For example, when writing a library it may be useful to provide one or more executables which
make use of the library's functionality. Such a project could be laid out as follows:

``` txt
parent_project/ -> Executables
  L-- subprojects/
      L-- library_subproject/ -> Library
```

The parent project makes use of the library produced by its library_subproject. Both the library
and executables are produced by the project.

### output_type

> **REQUIRED** [OutputTypeSpecifier](#output-type-specifier)

Specifies what the output item should actually produce.

If a [library output type](#library-output-types)
is specified, then the project is assumed to be a library project and can only contain that library output.
Otherwise if an [executable output type](#executable-output-types) is specified, then the project is considered
to be an an executable project and can contain any number of *executable* outputs. A full explanation of these
rules is given in the [output rules and constraints](#output-rules-and-constraints) section.

For a list and description of all available output types, see the [OutputTypeSpecifier](#output-type-specifier)
section.

#### Library Project Output Example

``` yaml
# A library output item is declared. That means this is a library project, so the project can only
# contain a single library target.
output:
  my-awesome-library:
    # Static or shared library. The type is selected at CMake configure time.
    output_type: Library
    entry_file: lib.hpp

    # Shared library
    # output_type: SharedLib

    # Static library
    # output_type: StaticLib
```

#### Executable Project Output Example

``` yaml
# At least one executable output item is declared. That means this is an executable project,
# and the project can contain many executables if desired.
output:
  my-awesome-executable:
    output_type: Executable
    entry_file: main.cpp
  other:
    output_type: Executable
    entry_file: other.cpp
```

### output entry_file

> **REQUIRED** `String`

The output's entry file, relative to the root directory.

For an executable output, this is the main file (usually `main.c` or `main.cpp`). For a library output,
the entry file should include the library's entire public interface. Entry files should ideally be placed
in the project's root directory.

The library entry file is currently required for convenience, as it allows the "entire library" to be
included with a single header. However, in the future it will be used as a convenient way to create a
precompiled header for the entire library.

#### Output Entry File Suggestions

1. Place entry files in the project root.
2. Executable entry files should contain the word `main`. Examples include `main.cpp` or `another-main.c`.
3. Library entry file name should match the project/subproject name. Example: a library called *mind-reader*
should have an entry file called `mind-reader.hpp`. This makes it clear which project the entry file is being
included from.

#### Output Entry File Config Examples

``` yaml
output:
  my-awesome-executable:
    output_type: Executable
    entry_file: main.cpp
  another:
    output_type: Executable
    entry_file: another-main.cpp
```

``` yaml
output:
  my-awesome-library:
    output_type: Library
    entry_file: my-awesome-library.hpp
```

### output link

> Optional `List<`[LinkSpecifier](#link-specifier)`>`

The list of libraries to link to the output item. Libraries must be namespaced with their
"project" name. See the [link specifier section](#link-specifier) for a full in-depth
explanation.

Link specifiers can be written in one of two formats:

1. `dependency-name::library-name`
2. `dependency-name::{ first-library-name, second-library-name, etc }`

Where *dependency name is explicitly listed in cmake_data.yaml* as one of:

- [subprojects](./missing_link.md)
- [predefined_dependencies](./missing_link.md)
- [gcmake_dependencies](./missing_link.md)

``` yaml
subprojects:
  - nested-lib
predefined_dependencies:
  SFML:
    git_tag: "2.5.1"
  nlohmann_json:
    git_tag: "v3.10.4"
gcmake_dependencies:
  gcmake-test-project:
    repo_url: "git@github.com:scupit/gcmake-test-project.git"
    commit_hash: "ee752d23db561793511b5723750ebab9b78ef78e"
output:
  my-complex-exe:
    output_type: Executable
    entry_file: main.cpp
    link:
      # Notice that the left side of each link specifier exactly matches
      # a name listed in one of the dependency section above.
      - nested-lib::file-helper
      - nlohmann_json::nlohmann_json
      - SFML::{ system, window, main }
      - gcmake-test-project::{ dll-lib, toggle-lib }
```

### output flags

**TODO:** Implement flags specific to output items.

### output defines

**TODO:** Implement compiler defines specific to output items.

## Using Dependencies

The `gcmake-rust` project defines a dependency as *"a project or group of functionality
which is required to build the toplevel project in its entirety"*.

There are three types of dependencies a project can have:

1. [**Subproject**](#subprojects): Subprojects are considered local dependencies of the main project
    even though they exist in (and are built as part of) the same project tree.
2. [**GCMake Dependency**](#gcmake-dependencies): A project which also use the `gcmake-rust` *cmake_data.yaml*
    config system.
3. [**Predefined Dependency**](#predefined-dependencies): A dependency which was made available to `gcmake-rust`
    using the [predefined dependency compatibility layer](./predefined_dependency_doc.md).
    For now this is either a CMake project, or a library already installed on the system which has an
    existing CMake "find module" written for it.
    (TODO: Change the name 'predefined dependency' to something more fitting and intuitive).

### subprojects

> Optional `List<String>`

A list of *case sensitive* directory names present in the project's *subprojects/* directory. Listing a
subproject here "imports" it into the project. Once a subproject is imported, it is built as part of the
main project and its libraries are made available to link to.

> Wait, but subprojects already exist in a single directory and can be retrieved automatically.
> Isn't manually writing them redundant, extra work?

In some sense, yes. Listing subprojects explicitly is redundant. However, doing so serves these two purposes:

1. The explicit list acts as a "whitelist". It ensures that only the listed projects will be used, and that
any change to a subproject directory name will be detected and reported upon running *gcmake*.
(The subproject will be detected as missing due to the name change.)

2. The subproject is immediately identifiable as a link namespace.

For example, when skimming through the *cmake_data.yaml*, seeing a listed subproject named `my-awesome-lib`
shows the reader that the subproject *subprojects/my-awesome-lib* exists (or should exist) and is built
as part of the main project. They can also assume `my-awesome-lib` can be used for linking; ie:
`my-awesome-lib::its-library`.

For more information on linking, see the [output link](#output-link) section.

``` yaml
# project-root/
#   L-- subprojects/
#       L-- my-awesome-lib/
#       L-- another-subproject/
subprojects:
  - my-awesome-lib
  - another-subproject

# Example use in linking
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    link:
      - my-awesome-lib::{ its-library, its-other-library }
```

### gcmake-dependencies

> Optional `Map<String,` [GCMakeDependencyData](#gcmake-dependency-data-object)`>`

Specifies other `gcmake-rust` C/C++ projects to be consumed by this project as dependencies.
These will be cloned into *dep/\<given project name\>*. For an explanation of all dependency
types, see [Using Dependencies](#using-dependencies).

> **NOTE:** The additional validation provided by `gcmake-rust` only works on these libraries
> once they are cloned. This means **an initial CMake configuration run should be done in order**
> **to clone the repository** before running *gcmake*, when possible.
>
> When adding a gcmake-dependency to your project, do these steps in order.
>
> 1. *Add the dependency to* `gcmake-dependencies` *list*, then run `gcmake`. This adds the appropriate
>     FetchContent section to the CMakeLists.txt so that CMake will know to clone the repository.
> 2. *Configure (or reconfigure) the CMake build.* This causes the repository to be cloned in its
>     respective *dep/* location in the project tree.
> 3. *Run* `gcmake` once more. Now that the dependency repository is cloned, `gcmake` is able to carry out its
>     additional validation steps on the dependency project.

``` yaml
gcmake-dependencies:
  my-other-gcmake-project:
    repo_url: git@github.com:some-user/my-other-gcmake-project.git
    git_tag: v1.1.0
    # Either git_tag or git_hash must be specified.
    # commit_hash: ee752d23db561793511b5723750ebab9b78ef78e

# Linking example
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    link:
      - my-other-gcmake-project::its-lib
```

#### GCMake Dependency Data Object

The required configuration for a specified [gcmake dependency](#gcmake-dependencies).

- `repo_url`: **REQUIRED** url location of the dependency repository.
- `git_tag`: The version tag the repo should be cloned at. Must be a string, and may contain a leading *'v'*.
              **REQUIRED if commit_hash is not specified**.
- `commit_hash`: The commit hash the repo should be cloned at. **REQUIRED if git_tag is not specified**.

``` yaml
gcmake-dependencies:
  my-other-gcmake-project:
    repo_url: git@github.com:some-user/my-other-gcmake-project.git
    git_tag: v1.1.0
    # Either git_tag or git_hash must be specified.
    # commit_hash: ee752d23db561793511b5723750ebab9b78ef78e
```

### predefined-dependencies

> **TODO:** Write this section.

## Data Type Reference

Reference for special values in cmake_data.yaml. Most of these "special values" are
just restricted strings.

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

### Output Type Specifier

*Case sensitive* type of an output item produced by the project. See [output_type](#outputtype) for
usage details.

> Header-only libraries are currently not supported, but should be added in the near future (once I figure out
> a good way to create them using CMake).

#### Executable Output types

- `Executable`: An executable binary

#### Library output types

- `Library`: Either a static or shared library. The type is selected at CMake configure time.
- `StaticLib`: A static library
- `SharedLib`: A shared libary (DLL)

### Link Specifier

A specially formatted String describing which libraries to [link to an output item](#output-link) or
[link to a pre-build script](./missing_link.md). Link specifiers can be written in two formats:

1. `project_name::library_name`
2. `project_name::{ first_library, second_library, etc }`

**project_name** is the *case sensitive* name of a subproject or dependency defined on the current project,
which is explicitly listed in one of the project properties:

- [subprojects](./missing_link.md)
- [predefined_dependencies](./missing_link.md)
- [gcmake_dependencies](./missing_link.md)

**library_name** (or each library name in the list) is the *case sensitive* name of a library/target exposed
by the given project or dependency.

> In the future, the [`show linkable`](./TODO.md#show) command will print a list of dependencies which can
> be linked to the current project. **However, this has yet to be implemented.**
