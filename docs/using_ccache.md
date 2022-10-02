# CCache

[CCache](https://ccache.dev/) is a fantastic tool for
[speeding up compile and recompilation times](https://ccache.dev/performance.html).

It is best supported on Linux and MacOS using either GCC or Clang, but has
[good support for several other platforms and compilers](https://ccache.dev/platform-compiler-language-support.html)
as well. For instance, I used it to compile wxWidgets on Windows using MinGW-w64 GCC without issue.

## Using CCache in a CMake Build

The best way to use CCache in a CMake build is to use it as a
[compiler launcher](https://cmake.org/cmake/help/latest/envvar/CMAKE_LANG_COMPILER_LAUNCHER.html)
by setting the `CMAKE_C_COMPILER_LAUNCHER` and `CMAKE_CXX_COMPILER_LAUNCHER` **environment variables**
to use `ccache`.

### Initial Project Configure

First, set the environment variables for the current shell session.

Windows (PowerShell):

``` ps1
# PowerShell
$env:CMAKE_C_COMPILER_LAUNCHER='ccache'
$env:CMAKE_CXX_COMPILER_LAUNCHER='ccache'
```

Linux/MacOS (bash):

``` bash
CMAKE_C_COMPILER_LAUNCHER='ccache'
CMAKE_CXX_COMPILER_LAUNCHER='ccache'
```

Then run the **initial CMake configuration**. Upon first configuration, CMake will read the values
of `CMAKE_C_COMPILER_LAUNCHER` and `CMAKE_CPP_COMPILER_LAUNCHER` and use those to launch the respective
C and C++ compilers.

``` sh
# Example
cmake -B build/ -DCMAKE_BUILD_TYPE=Release
```

Then build the project as usual.

### Subsequent Project Configure

If running a *subsequent CMake configuration in a project where ccache wasn't used as the compiler launcher*,
you'll need to set the CMake cache variables directly instead:

``` sh
cmake -B build/ -DCMAKE_BUILD_TYPE=Release -DCMAKE_C_COMPILER_LAUNCHER=ccache -DCMAKE_CXX_COMPILER_LAUNCHER=ccache
```

Then build the project as usual.
