# Pre-build scripts

> This page explains the creation, usage, and nuances of pre-build scripts.

Pre-build scripts can be written in `Python` (just make sure you have a
[Python interpreter](https://www.python.org/downloads/) installed), `C`, and `C++`.

## Things to Keep in Mind

> **Pre-build script working directory is always the project directory**
> **(Same directory as your cmake_data.yaml)**, not the build directory.

### General pre-build script rules

1. Pre-build scripts are run before any output in the immediate project is built.
2. A project can have only one pre-build script. **NOTE** that this doesn't apply to the *project tree*,
  just single project instances. The rule of thumb: there can be at most one pre-build script for every
  cmake_data.yaml in the project tree.
3. Pre-build scripts are not guaranteed to run in any particular order. It's probably a bad idea to have
  one pre-build script depend on the behavior of another pre-build script in the project tree.
4. If a pre-build script is used for code generation (i.e. it generates header, source, or template-impl
  files which will be used to build your project), each of the code files it generates must be explicitly
  listed under the [generated_code](cmake_data_config/properties/prebuild_config.md#generated_code)
  prebuild_config property.

### Executable pre-build script specific rules

1. Executable pre-build scripts can link to libraries built elsewhere in the
  project tree or to dependencies, but not to the library built by the immediate project tree. Libraries
  built in subprojects or parent projects are just fine, however.
2. The pre-build script inherits all of the project's
  [build_configs](cmake_data_config/properties/properties_list.md#build_configs) and
  [global_defines](cmake_data_config/properties/properties_list.md#global_defines) the same as an output
  executable would.
3. These will be built into the same build directory as everything else, but are not included in installations.

## Running only the pre-build scripts

See [cross-compilation caveats](cross_compilation.md#caveats) for an example of why you would want to do this.

GCMake will create a CMake target called `run-pre-build` which can be used to run only the pre-build
steps on some generators.

Example: `ninja run-pre-build`

This is could come in handy when cross-compiling a project which depends
on pre-build code generation, because pre-build executables can't be run when cross compiling.
In that case, you could configure a native project build and run the `run-pre-build` to only build
and execute the pre-build steps (skipping unnecessary compilations entirely). Then, configure a
cross-compilation build. Since your code was generated before, you're all set.

## Using a pre-build script

Pre-build scripts have various configuration options which can be modified with the
project [*prebuild_config* property](cmake_data_config/properties/properties_list.md#prebuild_config).

To add a pre-build script to your project, just add one of these files in your project directory (same
level as *cmake_data.yaml*):

| File | Description |
| ---- | ----------- |
| `pre_build.py` | `Python` pre-build script. This is recommended if your project is going to be cross-compiled. |
| `pre_build.c` | `C` pre-build script. CMake will build and run an executable using the source file as an entry point. |
| `pre_build.cpp` or `pre_build.cpp2` | `C++` or `CppFront's "C++2"` pre-build script. CMake will build and run an executable using the source file as an entry point. |

After adding the file, re-run `gcmake-rust` to regenerate the CMake configuration and you're good to go!

### Pre-build code generation

As stated in [general pre-build script rule](#general-pre-build-script-rules) #4,

> If a pre-build script is used for code generation (i.e. it generates header, source, or template-impl
> files which will be used to build your project), each of the code files it generates must be explicitly
> listed under the [generated_code](cmake_data_config/properties/prebuild_config.md#generated_code)
> prebuild_config property.
