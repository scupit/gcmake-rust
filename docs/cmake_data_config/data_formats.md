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

### Constraints With Language Features

Constraint expressions can also be used to check whether certain C, C++, or CUDA
[language features](https://cmake.org/cmake/help/latest/prop_gbl/CMAKE_CXX_KNOWN_FEATURES.html) are
available or in use. The
[list of all available language feature checks](#list-of-supported-language-feature-checks) is available in
the section below.

- C: `(( c:11 ))`
- C++: `(( cpp:11 ))`
- CUDA `(( cuda:11 ))`

``` yaml
languages:
  cpp:
    # The checks below will default to '0' or undefined when using C++98 standard, as the checked 
    # features were introduced in C++11.
    # Comment out 'exact_standard: 98' so that your compiler uses C++11 (or later, if the compiler
    # defaults to a later version), and the checks will pass;
    exact_standard: 98
    min_standard: 11

# ... rest of required config
global_defines:
  - (( cpp:constexpr )) IS_CONSTEXPR_SUPPORTED=1
  - (( not cpp:constexpr )) IS_CONSTEXPR_SUPPORTED=0
  - (( cpp:lambdas )) HAS_LAMBDA_SUPPORT

output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
```

``` c++
#include <cstdlib>
#include <iostream>

int main() {
  if (IS_CONSTEXPR_SUPPORTED) {
    std::cout << "Has support for constexpr!\n";
  }
  else {
    std::cout << "Does NOT have support for constexpr :(\n";
  }

  #ifdef HAS_LAMBDA_SUPPORT
    [&]() {
      std::cout << "Has lambda support! (printed inside a lambda)\n";
    }();
  #endif

  return EXIT_SUCCESS;
}

/*
  Output when standard is C++11:
  ----------------------------------------
  Has support for constexpr!
  Has lambda support! (printed inside a lambda)
*/

/*
  Output when standard is C++98:
  ----------------------------------------
  Does NOT have support for constexpr :(
*/
```

#### List of Supported Language Feature Checks

[Link to all CMake supported C language checks](https://cmake.org/cmake/help/latest/prop_gbl/CMAKE_C_KNOWN_FEATURES.html)

| C Feature | Matching CMake Feature Name |
| ----------- | --------------------------- |
|`c:90` | "c_std_90" |
|`c:99` | "c_std_99" |
|`c:11` | "c_std_11" |
|`c:17` | "c_std_17" |
|`c:23` | "c_std_23" |
|`c:function_prototypes` | "c_function_prototypes" |
|`c:restrict` | "c_restrict" |
|`c:static_assert` | "c_static_assert" |
|`c:variadic_macros` | "c_variadic_macros" |

[Link to all CMake supported C++ language checks](https://cmake.org/cmake/help/latest/prop_gbl/CMAKE_CXX_KNOWN_FEATURES.html)

| C++ Feature | Matching CMake Feature Name |
| ----------- | --------------------------- |
| `cpp:98`  | "cxx_std_98"  |
| `cpp:11`  | "cxx_std_11" |
| `cpp:14` | "cxx_std_14" |
|`cpp:17` | "cxx_std_17" |
|`cpp:20` | "cxx_std_20" |
|`cpp:23` | "cxx_std_23" |
|`cpp:26` | "cxx_std_26" |
|`cpp:template_templates` | "cxx_template_template_parameters" |
|`cpp:alignas` | "cxx_alignas" |
|`cpp:alignof` | "cxx_alignof" |
|`cpp:attributes` | "cxx_attributes" |
|`cpp:auto` | "cxx_auto_type" |
|`cpp:constexpr` | "cxx_constexpr" |
|`cpp:decltype_incomplete_return_types` | "cxx_decltype_incomplete_return_types" |
|`cpp:decltype` | "cxx_decltype" |
|`cpp:default_function_template_args` | "cxx_default_function_template_args" |
|`cpp:defaulted_functions` | "cxx_defaulted_functions" |
|`cpp:defaulted_move_initializers` | "cxx_defaulted_move_initializers" |
|`cpp:delegating_constructors` | "cxx_delegating_constructors" |
|`cpp:deleted_functions` | "cxx_deleted_functions" |
|`cpp:enum_forward_declare` | "cxx_enum_forward_declarations" |
|`cpp:explicit_conversions` | "cxx_explicit_conversions" |
|`cpp:extended_friend_declarations` | "cxx_extended_friend_declarations" |
|`cpp:extern_templates` | "cxx_extern_templates" |
|`cpp:final` | "cxx_final" |
|`cpp:func_identifier` | "cxx_func_identifier" |
|`cpp:generalized_initializers` | "cxx_generalized_initializers" |
|`cpp:inheriting_constructors` | "cxx_inheriting_constructors" |
|`cpp:inline_namespaces` | "cxx_inline_namespaces" |
|`cpp:lambdas` | "cxx_lambdas" |
|`cpp:local_type_template_args` | "cxx_local_type_template_args" |
|`cpp:long_long` | "cxx_long_long_type" |
|`cpp:noexcept` | "cxx_noexcept" |
|`cpp:nonstatic_member_init` | "cxx_nonstatic_member_init" |
|`cpp:nullptr` | "cxx_nullptr" |
|`cpp:override` | "cxx_override" |
|`cpp:range_for` | "cxx_range_for" |
|`cpp:raw_string_literals` | "cxx_raw_string_literals" |
|`cpp:ref_qualified_functions` | "cxx_reference_qualified_functions" |
|`cpp:right_angle_brackets` | "cxx_right_angle_brackets" |
|`cpp:rvalue_refs` | "cxx_rvalue_references" |
|`cpp:sizeof_member` | "cxx_sizeof_member" |
|`cpp:static_assert` | "cxx_static_assert" |
|`cpp:strong_enums` | "cxx_strong_enums" |
|`cpp:thread_local` | "cxx_thread_local" |
|`cpp:trailing_return` | "cxx_trailing_return_types" |
|`cpp:unicode_literals` | "cxx_unicode_literals" |
|`cpp:uniform_init` | "cxx_uniform_initialization" |
|`cpp:unrestricted_unions` | "cxx_unrestricted_unions" |
|`cpp:user_literals` | "cxx_user_literals" |
|`cpp:variadic_macros` | "cxx_variadic_macros" |
|`cpp:variadic_templates` | "cxx_variadic_templates" |
|`cpp:aggregate_default_initializers` | "cxx_aggregate_default_initializers" |
|`cpp:attribute_deprecated` | "cxx_attribute_deprecated" |
|`cpp:binary_literals` | "cxx_binary_literals" |
|`cpp:contextual_conversions` | "cxx_contextual_conversions" |
|`cpp:decltype_auto` | "cxx_decltype_auto" |
|`cpp:digit_separators` | "cxx_digit_separators" |
|`cpp:generic_lambdas` | "cxx_generic_lambdas" |
|`cpp:lambda_init_captures` | "cxx_lambda_init_captures" |
|`cpp:relaxed_constexpr` | "cxx_relaxed_constexpr" |
|`cpp:return_type_deduction` | "cxx_return_type_deduction" |
|`cpp:variable_templates` | "cxx_variable_templates" |

[Link to all CMake supported CUDA language checks](https://cmake.org/cmake/help/latest/prop_gbl/CMAKE_CUDA_KNOWN_FEATURES.html)

| CUDA Feature | Matching CMake Feature Name |
| ----------- | --------------------------- |
|`cuda:03` | "cuda_std_03" |
|`cuda:11` | "cuda_std_11" |
|`cuda:14` | "cuda_std_14" |
|`cuda:17` | "cuda_std_17" |
|`cuda:20` | "cuda_std_20" |
|`cuda:23` | "cuda_std_23" |
|`cuda:26` | "cuda_std_26" |

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
