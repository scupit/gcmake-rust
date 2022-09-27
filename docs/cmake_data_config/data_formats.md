# Data Formats

> This page describes the input formats for several types of data including compiler flags, compiler defines,
> link specifiers, target selection specifiers, and system specifiers.

## System Specifier

System specifiers constrain pieces of data to only be included on certain systems or under certain conditions.
Internally, these are mapped one-to-one to
[CMake's generator expressions](https://cmake.org/cmake/help/latest/manual/cmake-generator-expressions.7.html).

| Constraint | Meaning |
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

System specifiers are written in double parentheses `((...))`. Here are some examples:

`(( windows and (clang or gcc) ))`: The information is included when targeting a Windows system and
compiling using either Clang or GCC.

`(( windows and not mingw ))`: The information is included when targeting a Windows system, but only
when using a compiler other than MinGW.

> Make sure to use parentheses in order to guarantee precedence is correct. I haven't implemented expression
> precedence yet, but it's on the TODO list.

### System Specifier Use Case

System specifiers can currently be used on [compiler_flags](properties/build_configs.md#compilerflags),
[linker_flags](properties/build_configs.md#linkerflags), [defines](properties/build_configs.md#defines),
and [link specifiers](linking.md#formats). You probably won't have to use them much, but they can sometimes
be useful.

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

## Compiler Flags

Compiler flags should be written exactly as if you were passing them to your compiler on the command line.
They can optionally be prefixed with a [system specifier](#system-specifier).

To specify compiler flags for a specific output or build configuration, see the
[build_configs project property](properties/properties_list.md#buildconfigs) and the
[compiler_flags](properties/build_configs.md#compilerflags) section of build configuration.

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
They can optionally be prefixed with a [system specifier](#system-specifier).

To specify linker flags for a specific output or build configuration, see the
[build_configs project property](properties/properties_list.md#buildconfigs) and the
[linker_flags](properties/build_configs.md#linkerflags) section of build configuration.

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
[system specifier](#system-specifier).

To specify global compiler defines for your project, see the
[global_defines property](properties/properties_list.md#globaldefines). To specify compiler defines
for a specific output or biuld configuration, see the
[build_configs project property](properties/properties_list.md#buildconfigs) and the
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

Link specifiers may also be prefixed with [system specifiers](data_formats.md#system-specifier):

- `((linux or macos)) namespace::libname`
- `((unix)) namespace::{ lib1, lib2 }`
- `namespace::{ lib1, ((windows)) lib2 }`

However, a system specifier may only prefix the entire link specifier or individual libraries in
a multi-link specifier, but not both.

``` txt
For example, this is invalid:

((windows)) namespace::{ lib1, ((mingw)) lib2 }
```
