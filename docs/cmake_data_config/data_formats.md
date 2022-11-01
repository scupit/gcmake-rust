# Data Formats

> This page describes the input formats for several types of data including compiler flags, compiler defines,
> link specifiers, target selection specifiers, and system specifiers.

<!-- TODO: At this point constraint expressions should probably have their own page.
      That way I'd have more space to explain how they can be used to facilitate optional dependencies
      when paired with features.
-->
## Constraint Specifier

Constraint specifiers tell GCMake to only include or use certain data under the given conditions
or "constraints". Internally, these are mapped one-to-one to either
[CMake's generator expressions](https://cmake.org/cmake/help/latest/manual/cmake-generator-expressions.7.html)
or regular CMake conditional expressions, depending on the usage context.

Pre-defined constraint values such as `windows` and `unix` are the main building blocks of constraint
expressions. However, [project features](./properties/features.md) can also be used.

| Pre-defined Constraint | Meaning |
| ---------- | ------- |
| `android` | Targeting an Android system |
| `windows` | Targeting a Windows system |
| `linux` | Targeting a Linux system |
| `macos` | Targeting a MacOS system |
| `unix` | Targeting a Unix machine |
| `mingw` | Using a MinGW compiler |
| `gcc` | Using a GCC compiler |
| `clang` | Using a Clang compiler |
| `msvc` | Using a MSVC compiler |

**NOTE** that the `gcc`, `clang`, and `msvc` constraints probably won't be used often because
build configurations are already compiler-specific.

### Constraints with Features

[Project features](./properties/features.md) are very powerful when paired with constraint expressions.
Unlike pre-defined constraint values like `windows` and `mingw`, features must be defined by the project
before they can be used in constraint expressions.

``` yaml
features:
  color:
    default: true
  fancy-printing:
    default: true

global_defines:
  # Example usage for single features
  - (( feature:colors )) IS_COLOR_FEATURE_ENABLED=1
  - (( feature:fancy-printing )) IS_FANCY_PRINTING_FEATURE_ENABLED=1
  # Trying to reference a feature which hasn't been defined by the project will result in an error.
  # - (( feature:undefined-feature )) THIS_IS_AN_ERROR

predefined_dependencies:
  fmt:
    git_tag: "9.1.0"

output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    link:
      # Only link the fmt library if the fancy-printing feature is enabled.
      # Also, constraint expressions used when linking determine whether a library
      # will be loaded or not. In this case, fmt will only be cloned if the fancy-printing
      # feature is enabled.
      - (( feature:fancy-printing )) fmt::fmt
```

### Constraint examples

Constraint expressions are written in double parentheses `((...))`. Here are some examples:

| Expression | English |
| ---------- | ------- |
| `(( unix ))` | The information is included when targeting any Unix-based system. |
| `((not windows))` | The information is included when targeting any non-Windows system. |
| `((windows and (clang or gcc)))` | The information is included when targeting a Windows system and compiling using either Clang or GCC. |
| `(( feature:colors and feature:fancy-printing ))` | The information is included when both the *colors* and *fancy-printing* features of your project are enabled. |

> Make sure to use parentheses in order to guarantee precedence is correct. I haven't implemented expression
> precedence yet, but it's on the TODO list.

### Constraint Specifier Use Cases

Constraint specifiers can currently be used on [compiler_flags](properties/build_configs.md#compiler_flags),
[linker_flags](properties/build_configs.md#linker_flags), [defines](properties/build_configs.md#defines),
and [link specifiers](linking.md#formats), and [output](properties/output.md) items themselves.

One great use case for system specifiers is to use different compile defines depending on the operating
system you are targeting.

``` yaml
global_defines:
  - ((windows)) MY_SYSTEM="Wow, I'm on Windows!"
  - ((not windows)) MY_SYSTEM="Heck yeah, not on Windows!"
```

Another use case is to constrain when libraries are linked. For example, *SFML::main* can only be used
on Windows, so we'll want to make sure to only link it when targeting Windows:

``` yaml
predefined_dependencies:
  SFML:
    # ...
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    link:
      - SFML::{ system, ((windows)) main }
```

The linking use case is especially useful for optional library support.
**Libraries are only loaded if they are actually going to be used, so it is possible to optionally support a library by constraining all instances of linking that library to only happen when a feature is enabled.**

## Compiler Flags

Compiler flags should be written exactly as if you were passing them to your compiler on the command line.
They can optionally be prefixed with a [system specifier](#constraint-specifier).

To specify compiler flags for a specific output or build configuration, see the
[build_configs project property](properties/properties_list.md#build_configs) and the
[compiler_flags](properties/build_configs.md#compiler_flags) section of build configuration.

``` yaml
supported_compilers: [ GCC, MSVC ]
build_configs:
  Debug:
    GCC:
      compiler_flags:
        - -Og
        - -g
        - -Wall
        - ((unix)) -Wextra
    MSVC:
      compiler_flags: [ /Od, /W4, /DEBUG ]
```

## Linker Flags

Linker flags should be written exactly as if you were passing them to your linker on the command line.
However, don't include flags like `-Xlinker` which the compiler uses to pass flags on to the linker.
CMake will facilitate that automatically.
They can optionally be prefixed with a [system specifier](#constraint-specifier).

To specify linker flags for a specific output or build configuration, see the
[build_configs project property](properties/properties_list.md#build_configs) and the
[linker_flags](properties/build_configs.md#linker_flags) section of build configuration.

``` yaml
supported_compilers:
  - GCC
build_configs:
  Debug:
    GCC:
      linker_flags:
        - -s
```

## Compiler Defines

Defines should be written exactly as if you were passing them to your compiler on the command line,
just without the leading `-D` or `/D`. They can optionally be prefixed with a
[system specifier](#constraint-specifier).

To specify global compiler defines for your project, see the
[global_defines property](properties/properties_list.md#global_defines). To specify compiler defines
for a specific output or biuld configuration, see the
[build_configs project property](properties/properties_list.md#build_configs) and the
[defines](properties/build_configs.md#defines) section of build configuration.

``` yaml
supported_compilers: [ GCC, MSVC ]

global_defines:
  - JUST_DEFINED                        # Just defines, doesn't assign a value
  - DEFINED_WITH_VALUE=1                # Define and assign a value
  - DEFINED_WITH_STRING="Noice String"  # Define and assign a String
  - ((windows)) IS_WINDOWS

build_configs:
  Debug:
    GCC:
      defines:
        - DEFINED_FOR_GCC
        - ((unix)) GCC_DEBUG_ON_UNIX=1
    MSVC:
        - MSVC_AND_DEBUG=1
```

## Link Specifier

Link specifiers select dependency libraries to be linked.

> For general linking information, see [linking.md](linking.md).

Link specifiers can be given in either of two formats:

1. Single format: `namespace::libname`
2. Multi format: `namespace::{ lib1, lib2 }`

Link specifiers may also be prefixed with [system specifiers](data_formats.md#constraint-specifier):

- `((linux or macos)) namespace::libname`
- `((unix)) namespace::{ lib1, lib2 }`
- `namespace::{ lib1, ((windows)) lib2 }`

However, a system specifier may only prefix the entire link specifier or individual libraries in
a multi-link specifier, but not both.

``` txt
For example, this is invalid:

((windows)) namespace::{ lib1, ((mingw)) lib2 }
```
