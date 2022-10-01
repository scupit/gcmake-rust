# build_configs

> This page describes the [project-level build_configs property](properties_list.md#build_configs), which is
> used to define build configuration info for the project.

## Format

> **REQUIRED** `Map<BuildConfigName, Map<CompilerSelector, BuildConfigurationObject>>`

GCMake projects are required to define at least one build configuration, and can define up to all four.

Valid build configuration names are:

- `Debug`
- `Release`
- `MinSizeRel`
- `RelWithDebInfo`

``` yaml
build_configs:
  Debug: { }
  Release: { }
  MinSizeRel: { }
  RelWithDebInfo: { }
```

### Per-compiler Configuration

Flags and defines can be configured for every compiler supported by the
[supported_compilers](properties_list.md#supported_compilers) property. However, **the project is only allowed**
**to define configurations for compilers which are explicitly listed in the project's**
**`supported_compilers` list**.

In the case where common configuration needs to be added for every compiler in a build type, use the
`AllCompilers` selector. This is mainly useful for adding build-type-specific compiler definitions.

``` yaml
supported_compilers: [ GCC, Clang ]
build_configs:
  Release:
    AllCompilers: {}
    GCC: {}
    Clang: {}
  Debug:
    GCC: {}
    Clang: {}
    # This would throw an error, since MSVC was not listed as a supported compiler.
    # MSVC: {}
```

### Final format

The final build configuration format boils down to this:

``` yaml
build_configs:
  <BUILD_TYPE>:
    <COMPILER_SELECTOR>:
      defines: []
      compiler_flags: []
      linker_flags: []
  <BUILD_TYPE>:
    <COMPILER_SELECTOR>:
      # ...
```

### Full example

``` yaml
supported_compilers:
  - GCC
  - Clang
  - MSVC
build_configs:
  Debug:
    AllCompilers:
      defines:
        - DEBUG_MODE=1
        - BUILD_MODE="Debug aw yeah"
    GCC:
      compiler_flags: [ -Og, -g, -Wall, -Wextra, -Wconversion, -Wuninitialized, -pedantic, -pedantic-errors]
      defines:
        - GCC_DEBUG=1
    Clang:
      compiler_flags: [ -Og, -g, -Wall, -Wextra, -Wconversion, -Wuninitialized, -pedantic, -pedantic-errors]
      defines:
        - CLANG_DEBUG=1
    MSVC:
      compiler_flags: [ /Od, /W4, /DEBUG ]
      defines:
        - MSVC_DEBUG=1
  Release:
    AllCompilers:
      defines:
        - RELEASE_MODE=1
        - NDEBUG
        - BUILD_MODE="Release"
    GCC:
      compiler_flags: [ -O3 ]
      linker_flags: [ -s ]
      defines:
        - GCC_RELEASE=1
    Clang:
      compiler_flags: [ -O3 ]
      linker_flags: [ -s ]
      defines:
        - CLANG_RELEASE=1
    MSVC:
      compiler_flags: [ /O2, /GL ]
      defines:
        - MSVC_RELEASE=1
```

## Property List

| Property | Description |
| -------- | ----------- |
| [defines](#defines) | Section for specifying compiler defines |
| [compiler_flags](#compiler_flags) | Section for specifying compiler flags |
| [linker flags](#link_time_flags) | Section for specifying linker flags |
| [linker flags](#linker_flags) | Section for specifying linker flags |

### defines

> *Optional* `List<CompilerDefineString>`

The list of defines to be added for the selected configuration and compiler.

Defines should be written exactly as you would pass them on the command line, without the
leading flag prefix `-D`.  They may also be prefixed with a
[system specifier](../data_formats.md#system-specifier). Further formatting information is detailed
in [data_formats.md](../data_formats.md#compiler-defines).

``` yaml
supported_compilers: [ GCC, MSVC ]
build_configs:
  Debug:
    AllCompilers:
      defines:
        - IN_DEBUG_MODE=1
    GCC:
      defines:
        - MY_STATE="GCC debug"
        - ((windows)) DEFINED_ON_WINDOWS_ONLY
    MSVC:
      defines:
        - MY_STATE="MSVC debug"
```

### compiler_flags

> *Optional* `List<CompilerFlagString>`

The list of flags to be passed to the given compiler for the selection configuration.

Compiler flags should be written exactly as you would pass them on the command line.
They may also be prefixed with a [system specifier](../data_formats.md#system-specifier).
Further formatting information is detailed in [data_formats.md](../data_formats.md#compiler-flags).

**NOTE:** compile_flags passed to Emscripten will automatically also be passed at link time.
Most Emscripten flags (especially [optimization flags](https://emscripten.org/docs/compiling/Building-Projects.html#building-projects-with-optimizations))
passed to the compiler should also be passed at link time (but not to the linker, there's apparently a
difference), so this is done automatically.

``` yaml
supported_compilers:
  - GCC
  - MSVC
build_configs:
  Debug:
    GCC:
      compiler_flags: [ -Og, -g, -Wall ]
    MSVC:
      compiler_flags: [ /Od, /W4, /DEBUG ]
  Release:
    GCC:
      compiler_flags: [ -O3 ]
    MSVC:
      compiler_flags: [ /O2 ]
```

### link_time_flags

> *Optional* `List<CompilerFlagString>`

The list of flags to be passed at link time for the selection configuration.

These flags should be written exactly as you would pass them on the command line.
They may also be prefixed with a [system specifier](../data_formats.md#system-specifier).
Further formatting information is detailed in [data_formats.md](../data_formats.md#compiler-flags).

``` yaml
supported_compilers:
  - Emscripten
build_configs:
  Debug:
    Emscripten:
      link_time_flags: [ --shell-file, src/custom-shell.html ]
  Release:
    Emscripten:
      link_time_flags: [ --shell-file, src/custom-shell.html ]
```

### linker_flags

> *Optional* `List<`[LinkerFlagString](../data_formats.md#linker-flags)`>`

The list of flags to be passed to the linker for the selection configuration.

Linker flags should be written exactly as you would pass them on the command line.
However, don't include flags like `-Xlinker` which compilers use to pass flags on to the linker.
CMake will facilitate that automatically.
They may also be prefixed with a [system specifier](../data_formats.md#system-specifier).
Further formatting information is detailed in [data_formats.md](../data_formats.md#linker-flags).

``` yaml
supported_compilers:
  - GCC
  - MSVC
build_configs:
  Debug:
    GCC:
      linker_flags: [ -s ]
  Release:
    GCC:
      linker_flags: [ -s ]
```
