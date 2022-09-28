# Auto Generated Export Macro Headers

> This page explains
> [auto generated export headers](https://cmake.org/cmake/help/latest/module/GenerateExportHeader.html)
> and how to use them.

## About

An [export header](https://cmake.org/cmake/help/latest/module/GenerateExportHeader.html) is automatically
generated for every compiled library output in the project at CMake configure time.

Each header is generated at
`THE_BUILD_DIRECTORY/generated_export_headers/FULL_INCLUDE_PREFIX/<target-name>_export.h`
and installed to `include/FULL_INCLUDE_PREFIX/<target_name>_export.h`.

The export macro will be the uppercase target name suffixed with `_EXPORT`, but with all whitespace
and dashes `-` replaced by underscores `_`.

Example: target name `my-lib` becomes the macro `MY_LIB_EXPORT`.

The `gcmake-rust target-info` command can be used to display the export header include path for a library.

## Example

Assume we have the following project configuration:

``` yaml
# ... rest of the configuration
include_prefix: MY_PREFIX
output:
  my-lib:
    output_type: CompiledLib
    entry_file: my-lib.hpp
```

Run `gcmake-rust target-info -e my-lib` to get the export header include path:

`MY_PREFIX/my-lib_export.h`. Great, now we can use it in the main file:

``` c++
// my-lib.hpp
#include <iostream>
#include "MY_PREFIX/my-lib_export.h"

// Example use for the MY_LIB_EXPORT macro.
void MY_LIB_EXPORT some_public_function() {
  std::cout << "Aw yeah, very nice\n";
}
```

## Why is this necessary?

Windows requires DLLs to explicitly mark functions as part of the library's public interface using
`__declspec(dllexport)` when building, and as `__declspec(dllexport)` when being consumed. This
"export macro header" automatically creates the macro for you, and CMake facilitates the proper
defines behind the scenes automatically as well.

Marking functions and classes using the export macro isn't required for static libraries, however
it is recommended. If you ever wanted to also build the static library as a Shared library, you'd
have to go back through and annotate every function and class by hand... doesn't sound fun, right?

CMake does have the ability to facilitate
[exporting all symbols by default](https://cmake.org/cmake/help/latest/variable/CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS.html#variable:CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS),
however that tends to cause issues.
