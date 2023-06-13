# output

> This page describes the [output](properties_list.md#output) configuration property in cmake_data.yaml

## General Output Rules

1. A single project may create either executables or a library, but not both.
2. A project may only create a single library, not multiple.
3. The first two rules only apply to *single projects*, **NOT** the whole project tree.
  For example, a root project which creates a library can have multiple subprojects which each create
  a single library, as well as multiple subprojects which each create several executables.

## Propagation

`HeaderOnlyLib` libraries always pass their compiler defines (including project global defines) and "linked"
dependencies to their consumers.

`CompiledLib`, `SharedLib`, and `StaticLib` libraries pass their compiler defines and *public* linked
dependencies to their consumers.

## Test Executables

Executables build by test projects have a few additional properties:

1. They automatically declare the project's test framework as a dependency.
2. If the project builds a library, then that library is automatically linked to each test executable.
  If the project builds executables, then all dependencies for each executable are automatically linked to
  each test executable. Essentially, test executables automatically have access to the project's test framework
  and all code written for the project.

## Property list

| Property | Is required | Type | Description |
| -------- | ----------- | ---- | ----------- |
| [output_type](#output_type) | **Required** | [Output Item Type](#output_type) | Dictates the output item's type (executable vs library, and which library type) |
| [entry_file](#entry_file) | **Required** | Relative file name | Sets the output item's entry point. |
| [windows_icon](#windows_icon) | *Optional* | Relative file name (relative to root project) | Sets an executable's Windows icon. |
| [emscripten_html_shell](#emscripten_html_shell) | *Optional* | Relative file name (relative to root project) | Sets a [custom HTML shell file](https://emscripten.org/docs/tools_reference/emcc.html#emcc-shell-file) for an executable when building with Emscripten. |
| [link](#link) | *Optional* | `List<`[LinkSpecifier](../data_formats.md#link-specifier)`>` | This section is used to link libraries to your output. |
| [language_features](#language_features) | *Optional* | `List<`[LanguageFeatureSpecifier](../data_formats.md#language-feature-specifier)`>` | Specifies a list of language features required to build the item successfully. |
| [build_config](#build_config) | *Optional* | `Map` (see the property for more info.) | Define additional build configuration which is specific to the output item only. |
| [defines](#defines) | *Optional* | `List<String>` | This is an 'alias' property for setting compiler defines for a target which are always applied (AllConfigs, AllCompilers). |
| [requires_custom_main](#requires_custom_main) | *Optional* | boolean | **Applies to test executables only.** Dictates whether or not the test executable must provide its own main function. |

### output_type

> **REQUIRED** `OutputItemType`

The type of output to be created. Must be one of:

| Type | Rules Subtype | Description |
| ---- | ------- | ----------- |
| `Executable` | Executable | Creates an executable |
| `CompiledLib` | Compiled Binary Library | Creates a library which can be set to either Static or Shared. |
| `StaticLib` | Compiled Binary Library | Creates a static library |
| `SharedLib` | Compiled Binary Library | Creates a shared library (DLL) |
| `HeaderOnlyLib` | Header-only library | Creates a header-only library |

``` yaml
output:
  my-exe:
```

### entry_file

> **REQUIRED** Relative file name `String`

Sets the output's entry point.

Entry files are specified relative to the project's directory.

An **Executable** entry file must be a source file (*.c* or *.cpp*), while any **library** entry file
must be a header (*.h* or *.hpp*).

Executable example:

``` yaml
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
```

Library example:

``` yaml
output:
  my-lib:
    output_type: CompiledLib
    # output_type: HeaderOnlyLib
    # output_type: StaticLib
    # output_type: SharedLib
    entry_file: my-lib.hpp
```

### windows_icon

> *Optional* Relative file path `String`

**IMPORTANT!** The windows icon path is resolved **relative to the root project**, not to the project
which the output is built from. This means that icon files should best placed somewhere in the root project
for easy access.

This property sets an executable output's icon to the given `.ico` file. As noted above, remember that
this relative icon file path is resolved relative to the root project's directory, not the current project.

For example, assume the root project contains the directory `icons/` and the icon file `icons/Smiley.ico`.
This configuration:

``` yaml
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    windows_icon: icons/Smiley.ico
```

sets *my-exe*'s icon to `YOUR_ROOT_PROJECT_DIR/icons/Smiley.ico` regardless of whether *my-exe* is defined
in the root project or a subproject.

### emscripten_html_shell

> *Optional* Relative file path `String`

**IMPORTANT!** The windows icon path is resolved **relative to the root project**, not to the project
which the output is built from. This means that icon files should best placed somewhere in the root project
for easy access.

For example, assume the root project contains the directory `shell-files/` and the icon file
`shell-files/my-awesome-shell-file.html`. This configuration:

``` yaml
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    emscripten_html_shell: shell-files/my-awesome-shell-file.html
```

sets *my-exe*'s icon to `YOUR_ROOT_PROJECT_DIR/shell-files/my-awesome-shell-file.html` regardless of
whether *my-exe* is defined in the root project or a subproject.

### link

> *Optional* `List<`[LinkSpecifier](../data_formats.md#link-specifier)`>`

This section is used to link libraries to your outputs and pre-build script.

For a full explanation of how linking works in GCMake, see [linking.md](../linking.md).

Dependencies are consumed differently by each output type. These tables give a basic explanation
of how dependencies are consumed and propagated depending on the output type. In this scenario,
it's best to think of dependency **propagation** as the way knowledge of a dependency's headers
and compiler defines are passed to its consumers.

| Category name | Explanation |
| ------------- | ----------- |
| *public* | The dependency is compiled as part of the output item, and is propagated to anything which depends on the output item. |
| *interface* | The dependency is NOT compiled as part of the output item, but is propagated to anything which depends on the output item. Header-only libraries always consume dependencies this way. |
| *private* | The dependency is compiled as part of the output item, but is not propagated. Executables always consume dependencies this way. |

| Output type | Dependency consumption category | Propagation explanation |
| ----------- | ------------------------------- | ----------------------- |
| `Executable` | *private* | Libraries are always compiled as part of an executable, but are never propagated. This is because executables are the final form of a program. Nothing ever links to an executable (as far as I'm aware). |
| `HeaderOnlyLib` | *interface* | Libraries "linked" to a header-only library output are always propagated to each output which "links" to the header-only library. This is because header-only libraries are not compiled directly. The header files that make up the library are consumed by each dependent output, which means any library needed by the header-only library is automatically part of the header-only library's public interface. |
| `CompiledLib`, `StaticLib`, `SharedLib` | Explicitly categorized as either *public* or *private* | Dependencies will always be compiled as part of a compiled library. However, consumers of your library will not always need knowledge of your library's dependencies. If a dependency is only referenced in implementation files (*.c*, *.cpp*) but not in any headers, then the dependency can be considered part of your library's *private* interface, and doesn't need to be propagated. However, any dependency which is referenced in any of your library's header files must be transitively exposed to any consumer of your library, since the consumer makes use of your headers. In that case, the dependency is part of your library's *public* interface, and must be propagated. |

Executable example:

``` yaml
predefined_dependencies:
  SFML:
    git_tag: "2.5.1"
  fmt:
    git_tag: "9.1.0"
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    link:
      - fmt::fmt
      # This is a good example of combining a system specifier and a link specifier
      - SFML::{ system, window, ((windows)) main }
```

Header-only library example:

``` yaml
predefined_dependencies:
  fmt:
    git_tag: "9.1.0"
output:
  my-headeronly-lib:
    output_type: HeaderOnlyLib
    entry_file: my-headeronly-lib.hpp
    link:
      - fmt::fmt
```

Here, `fmt::fmt` is propagated to any output which links *my-headeronly-lib*.

Compiled library example:

``` yaml
predefined_dependencies:
  SFML:
    git_tag: "2.5.1"
  fmt:
    git_tag: "9.1.0"
output:
  my-compiled-lib:
    output_type: CompiledLib
    # output_type: StaticLib
    # output_type: SharedLib
    entry_file: my-compiled-lib.hpp
    link:
      public:
        - fmt::fmt
      private:
        - SFML::{ system, window, ((windows)) main }
```

Here, `fmt::fmt` is propagated to any library which links to *my-compiled-lib*. The linked SFML libraries,
however, are not propagated as part of the link interface. This means that those SFML libraries will not
be automatically linked to any output which links to *my-compiled-lib*. However, the SFML library outputs
will still be built and installed because they might be needed (if built as shared libraries).

### language_features

> *Optional* `List<`[LanguageFeatureSpecifier](../data_formats.md#language-feature-specifier)`>`

This property is used to specify the language features required to build an output item. When specified,
CMake will check **at configure time** whether your compiler supports the specified features.

<!-- TODO: Add a command for printing supported compile features -->

Specified features are required
*unless constrained using a [constraint expression](../data_formats.md#constraint-specifier)*, in which case
they will only be required when the constraint expression evaluates to *true*.

**Language features follow the same inheritance rules as [linked libraries](#link)**, and therefore must be
[categorized the same way](#link) as well.

Basic executable example:

``` yaml
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    language_features:
      - cpp::{ lambdas, decltype }
```

More complex executable example:

``` yaml
features:
  full-constexpr:
    default: true

output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    language_features:
      - cpp::{ lambdas, decltype }
      # This one means: Require support for the C++ constexpr and relaxed_constexpr LANGUAGE features
      # when the PROJECT feature "full-constexpr" is enabled.
      - (( feature:full-constexpr )) cpp::{ constexpr, relaxed_constexpr }
```

Compiled library example:

``` yaml
output:
  my-compiled-lib:
    output_type: CompiledLib
    # output_type: StaticLib
    # output_type: SharedLib
    entry_file: my-compiled-lib.hpp
    language_features:
      public:
        - cpp::constexpr
      private:
        - cpp::relaxed_constexpr
```

### build_config

> *Optional* `Map<BuildTypeSelector, BuildConfigObject>`

This property is used to define additional build configuration specifically for an output item.

This is configured the same way as the [project build_configs property](properties_list.md#build_configs),
except for these two details:

1. Configurations can only be specified for build types already explicitly defined in the project-level
  build_configs. This means a `Release` build configuration cannot be defined specifically for an output
  if the root project only defines a `Debug` configuration.
2. In addition to existing configuration names (`Debug`, `Release`, `MinSizeRel`, and `RelWithDebInfo`),
  A configuration called `AllConfigs` can be used as well. `AllConfigs` is used to specify common build
  config info for all configurations defined by the root project. As a result, `AllConfigs` can always
  be used because GCMake guarantees that the root project defines at least one valid build configuration.

Example:

``` yaml
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    build_config:
      # This is the way to create a "global" compiler definition which is only defined for this executable.
      AllConfigs:
        AllCompilers:
          defines:
            - THIS_IS_ONLY_DEFINED_FOR_THIS_EXECUTABLE="true story"
      Debug:
        GCC:
          defines:
            - DEBUG_GCC_ONLY_DEFINE
      # This would fail because a MinSizeRel configuration does not exist in the project's build_configs.
      # MinSizeRel: {}
build_configs:
  Debug: {}
  Release: {}
```

### defines

> *Optional* `List<String>`

This property is used to add compiler defines to the target which will always be used. This is essentially
an 'alias' for *build_configs -> AllConfigs -> AllCompilers -> defines*.

For example, this:

``` yaml
build_configs:
  Debug: {}
  Release: {}
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    defines:
      - (( msvc )) AM_I_USING_MSVC=1
      - SOME_STRING_CONSTANT="This will always be used, but is specific to the target"
```

is equivalent to this:

``` yaml
build_configs:
  Debug: {}
  Release: {}
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    build_config:
      AllConfigs:
        MSVC:
          defines: 
            - AM_I_USING_MSVC=1
        AllCompilers:
          defines:
            - SOME_STRING_CONSTANT="This will always be used, but is specific to the target"
```

### requires_custom_main

> *Optional* `boolean`
>
> Default: `false`

Dictates whether the test executable must provide its own main function and facilitate its own test
argument parsing. **NOTE:** This property applies to test executables only.

All three [test frameworks](properties_list.md#test_framework) supported by GCMake have two modes:

1. The test framework automatically generates a main function and initializes itself. This is the
  most commonly used mode.
2. The test framework leaves the main function and framework initialization up to you. This is less
  commonly used, but is needed for testing GUI code and for some complex pre-test setup.

``` yaml
output:
  my-exe-in-test-project:
    output_type: Executable
    entry_file: main.cpp
    requires_custom_main: true
```
