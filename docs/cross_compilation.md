# Cross Compilation

> This page has a little explanation on cross-compilation, which projects support cross-compilation,
> and how to cross-compile using [Zig](https://ziglang.org/).

See a working cross-compilation example with a toolchain file at
[gcmake-test-project/reference-parent](/gcmake-test-project/reference-parent/cross_compilation_toolchains/zig_to_RPI-64bit.toolchain.cmake).

**NOTE:** Before reading the rest of this page, make sure you understand
[how to set Zig as the CMake C/C++ compiler](compile_using_zig.md)

> Tip: `gcc -dumpmachine`, `clang -dumpmachine`, and `zig cc -dumpmachine` all print out their compilation
> target triple. This is useful for getting the target triple of the current system, assuming the compiler
> is compiling to the current system.
>
> Another Tip: On linux, use `readelf -d path/to/executable-or-shared-lib | grep 'R.*PATH'` to examine
> the RPATH of an executable or shared library. This is mainly for developing the GCMake tool, since I need
> to make sure the assumed RPATH is correct.

## Basic Overview

Cross-compilation is a pain in the butt when using a compiler toolchain that isn't Zig. Zig makes cross-compilation easy, and
[that will probably always be the case](https://ziglang.org/learn/overview/#cross-compiling-is-a-first-class-use-case).
Therefore I recommend using Zig for cross-compiling C/C++ projects unless you really know what you're doing.

### Which Projects can be Cross Compiled?

GCMake says that a project can be trivially cross-compiled if:

1. The project doesn't require any additional dependency configuration by the eventual user.
2. All subprojects and test projects contained by the project can be trivially cross-compiled.
3. All dependencies of the project can be trivially cross-compiled and are built as part of the project.

**This definition is recursive, and also applies to GCMake projects consumed as dependencies.**

This means your project can still use many predefined subdirectory dependencies such as
[nlohmann_json](/gcmake-dependency-configs/nlohmann_json/) or [fmt](/gcmake-dependency-configs/fmt)
witout losing the ability to be trivially cross compiled.
Dependencies which can be cross compiled contain the line `can_cross_compile: true` in their
[dependency configuration](/gcmake-dependency-configs/).

**In short, if your project uses no dependencies or only uses self-contained dependencies built as**
**a subdirectory of your project, you can probably trivially cross-compile your project.**

However, dependencies such as [wxWidgets](/gcmake-dependency-configs/wxWidgets/) and
[SFML](/gcmake-dependency-configs/SFML/) are marked as projects which block trivial cross compilation.

## Caveats

*Executable pre-build scripts will not be run when cross-compiling* because the final executable will
be incompatible with your host system. For the same reason, built executable outputs and test executables
will also be unable to run on your host machine.

If running pre-build scripts before compilation is required, do a native build first. Running the
`run-pre-build` target (example: `ninja run-pre-build`) will run all pre-build scripts while skipping
all compilations not required to do so. After the scripts have run, then start your cross-compilation build.
See the [pre-build scripts page](pre_build_scripts.md) for more information.

## Using Zig as the C/C++ Cross Compiler

When cross-compiling with Zig, the target triple needs to be included as part of the compiler/flag list.
For example, when cross-compiling to Raspbian 64 bit running on Raspberry PI I use
`'zig;cc;--target=aarch64-linux-gnu'` and `'zig;c++;--target=aarch64-linux-gnu'` when using CMake GUI.

Here is an example of how to do this in a
[CMake cross-compilation toolchain files](https://cmake.org/cmake/help/latest/manual/cmake-toolchains.7.html)
(Taken from [gcmake-test-project/reference-parent](/gcmake-test-project/reference-parent/)):

``` cmake
# Just an example
set( CMAKE_SYSTEM_NAME "Linux" )
set( CMAKE_SYSTEM_PROCESOR "aarch64" )

set( target_triple "aarch64-linux-gnu" )

set( CMAKE_C_COMPILER zig cc )
set( CMAKE_C_COMPILER_TARGET ${target_triple} )
set( CMAKE_CXX_COMPILER zig c++ )
set( CMAKE_CXX_COMPILER_TARGET ${target_triple} )

set( CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER )
set( CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY )
set( CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY )
set( CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY )
```

### Basic toolchain file examples

- [cross-compiling for linux](https://cmake.org/cmake/help/latest/manual/cmake-toolchains.7.html#cross-compiling-for-linux)
- [cross-compiling using Clang](https://cmake.org/cmake/help/latest/manual/cmake-toolchains.7.html#cross-compiling-using-clang)