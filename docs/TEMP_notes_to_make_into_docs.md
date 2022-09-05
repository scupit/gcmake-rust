# TEMP Notes, need to turn these into docs later

## Using Zig as the C/C++ compiler

IMPORTANT TODO: [Use this link](https://devdocs.io/cmake~3.21/variable/cmake_lang_compiler) to explain
why zig can be used as a compiler when specified as `'zig;cc'` and `'zig;c++'`

When specifying the compiler in the GUI, use `'zig;cc'` and `'zig;c++'` exactly as written, including quotes
and semicolons (single quotes can be replaced with double quotes).

## Using Zig as the C/C++ Cross Compiler

> Tip: `gcc -dumpmachine`, `clang -dumpmachine`, and `zig cc -dumpmachine` all print out their compilation
> target triple. This is useful for getting the target triple of the current system, assuming the compiler
> is compiling to the current system.

> Another Tip: On linux, use `readelf -d path/to/executable-or-shared-lib | grep 'R.*PATH'` to examine
> the RPATH of an executable or shared library. This is mainly for developing the GCMake tool, since I need
> to make sure the assumed RPATH is correct.

When cross-compiling with Zig, the target triple needs to be included as part of the compiler/flag list.
For example, when cross-compiling to Raspbian 64 bit running on Raspberry Pi, I use
`'zig;cc;--target=aarch64-linux-gnu'` and `'zig;c++;--target=aarch64-linux-gnu'`.

Additional options (set in CMake format... I need to do more research on
[CMake cross-compilation toolchain files](https://cmake.org/cmake/help/latest/manual/cmake-toolchains.7.html)
before I can fully understand how this works).

``` cmake
set( CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY )
set( CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY )

# This doesn't seem to be set when cross-compiling without a toolchain
# file. I also wasn't using find_package, so that might make a difference. Needs research.
set( CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY )

set( CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER )

# Just an example
set( CMAKE_SYSTEM_NAME "Linux" )

```

### Basic toolchain file examples

- [cross-compiling for linux](https://cmake.org/cmake/help/latest/manual/cmake-toolchains.7.html#cross-compiling-for-linux)
- [cross-compiling using Clang](https://cmake.org/cmake/help/latest/manual/cmake-toolchains.7.html#cross-compiling-using-clang)
