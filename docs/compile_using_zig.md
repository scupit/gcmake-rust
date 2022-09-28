# Using Zig as a C/C++ Compiler

> This page explains how to use [Zig](https://ziglang.org/) as a C/C++ compiler out of the box with CMake.

To build a project using Zig as a C/C++ compiler:

1. Tell CMake to use the Zig compilers using one of the methods below.
2. Run the build as you normally would.

## Setting the compiler using Environment Variables

powershell

``` powershell
$env:CC="zig cc"
$env:CXX="zig c++"
```

bash

``` bash
CC="zig cc"
CXX="zig c++"
```

Once the environment variables are set, configuring a *ninja* or *make* (or whatever non-MSVC build system
you use) build should default to the zig C and C++ compilers.

## Setting the compiler in CMake GUI

After reading the
[CMAKE_\<LANG>_COMPILER doc page](https://cmake.org/cmake/help/latest/variable/CMAKE_LANG_COMPILER.html)
I realized that CMake GUU probably just passes the given compilers to the build using *CMAKE_C_COMPILER* and
*CMAKE_CXX_COMPILER*. We can take advantage of this.

When *initially configuring a build* in CMake GUI, select the option to use alternate/custom compilers.
Write `'zig;cc'` for the C compiler and `'zig;c++'` for the C++ compiler. Those must be input exactly as
written, inclusing quotes and semicolons (although the single quotes can be replaced with double quotes).
