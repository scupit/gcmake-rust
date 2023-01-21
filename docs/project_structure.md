# Project Structure

> This page describes the GCMake project structure and explains why certain choices were made.
>
> **TODO:** Add more detail, such as why only one library can be built per single project.
> Also explain some of the additonal CMake things GCMake facilitates, such as PGO and IPO.

## The Structure

| Directory | Description |
| --------- | ----------- |
| `src/FULL_INCLUDE_PREFIX/` | Project source files (*.c*, *.cpp*) and private headers/template-implementations (*.private.h*, *.private.hpp*, *.private.tpp*) go here. The files will be recursively found and added to the build. |
| `include/FULL_INCLUDE_PREFIX/` | Project header files (*.h*, *.hpp*) and template implementation files (*.tpp*, *.inl*) go here. The files will be recursively found and added to the build. |
| `resources/FULL_INCLUDE_PREFIX/` | Any assets needed by the project at runtime go here. The `resources/` directory recursively copied into the build directory at build time, and is also fully installed as part of the installation tree. |
| `subprojects/` | Subprojects go in this directory. Each subproject in this directory is automatically found and configured by GCMake as a subproject. Instead of creating these by hand, use `gcmake-rust new subproject 'your-subproject-name'` |
| `tests/` | Test projects go in this directory. Each test project in this directory is automatically found and configured by GCMake. Instead of creating these by hand, use `gcmake-rust new test 'your-test-name'` |
| `docs/` | Documentation configuration files such as *Doxyfile.in*, *conf.py.in*, and *index.rst* go in this directory. When the project is [configured to use a documentation generator](documenting_your_project.md#configuration), this directory is searched for any configuration files. |
| `cmake/` | **AUTO-CONFIGURED:** The directory which contains GCMake's CMake utility scripts. This is auto-generated every time GCMake configures the project. This should be committed in your source control. |

| File | Description |
| ---- | ----------- |
| `cmake_data.yaml` | This is the [GCMake configuration file](cmake_data_config/cmake_data.md). |
| `pre_build.py` \| `pre_build.c` \| `pre_build.cpp` \| `pre_build.cpp2` | **Optional** [pre-build script](pre_build_scripts.md) |
| `LICENSE` \| `LICENSE.md` \| `LICENSE.txt` | The file specifying the project's license. This will also be embedded in some graphical installers. |
| `CMakeLists.txt` | **AUTO-CONFIGURED:** The file which facilitates CMake builds. This paired with the `cmake/` directory is what makes the magic happen. This should be committed in your source control. |
| `Config.cmake.in` | **AUTO-CONFIGURED:** The configuration template for a CMake installation. This file allows CMake to discover an installation of your project, and as a result allows other CMake projects to use your project installation as a CMake dependency with just a single *find_package* call. This should be committed in your source control. |
| `install-deb-development-packages.sh` | **AUTO-CONFIGURED:** A helper file for installing all debian packages which may be needed for developing the project. This should be committed in your source control. |

## File Extensions

| File Type | Directory | C extensions | C++ extensions |
| --------- | --------- | ------------ | -------------- |
| Source | *src/FULL_INCLUDE_PREFIX* | `.c` | `.cpp`, `.cc`, `.cxx` |
| Header | *include/FULL_INCLUDE_PREFIX* | `.h` | `.hpp`, `.hh`, `.hxx` |
| Private Header | *src/FULL_INCLUDE_PREFIX* | `.private.h` | `.private.hpp`, `.private.hh`, `.private.hxx` |
| Template Implementation | *include/FULL_INCLUDE_PREFIX* | N/A | `.tpp`, `.txx`, `.tcc`, `.inl` |
| Private Template Implementation | *src/FULL_INCLUDE_PREFIX* | N/A | `.private.tpp`, `.private.txx`, `.private.tcc`, `.private.inl` |

### Private Headers

Some projects may choose to split their header files into public/private sections, so that their
"public headers" only contain full definitions for code which is part of their project's
public interface. These headers must contain *.private* before their actual extension
([see above](#file-extensions)).

To generate private headers for an existing project, use `--private` in combination with the `gen-file`
command. For example:

``` sh
gcmake-rust gen-file cpp --private --which hst MyPrivateFile some/nested/AnotherPrivateFile
```

will generate these files:

- src/FULL/INCLUDE_PREFIX/MyPrivateFile.cpp
- src/FULL/INCLUDE_PREFIX/MyPrivateFile.private.hpp
- src/FULL/INCLUDE_PREFIX/MyPrivateFile.private.tpp
- src/FULL/INCLUDE_PREFIX/some/nested/AnotherPrivateFile.cpp
- src/FULL/INCLUDE_PREFIX/some/nested/AnotherPrivateFile.private.hpp
- src/FULL/INCLUDE_PREFIX/some/nested/AnotherPrivateFile.private.tpp

Note how the headers and template implementation files all contain *.private* in their extension.
Also, everything is located in the *src/* directory. This is because private headers are not
part of your project's public interface, and therefore shouldn't be placed in the *include/* directory.

Also NOTE that **private headers and private template-implementation files are not documented by default**.
To document them with your project, set [documentation.include_private_headers](./cmake_data_config/properties/properties_list.md#documentation) to `true`.

``` yaml
# ... rest of project config
documentation:
  generator: Doxygen # Just an example
  include_private_headers: true
```

## CMake Options

> **TODO:** Document the rest of the CMake options provided by GCMake, both globally and per-project.

``` yaml
project_name: my-project
# ...rest of project config
```

| Name | Type | Default | Example | Description |
| ---- | ---- | ------- | ------- | ----------- |
| `${project_name}_BUILD_DOCS` | *Boolean* | `OFF` | `-Dmy-project_BUILD_DOCS=ON` | When `ON`, documentation for your project will be built and installed if your project is [configured to do so](./documenting_your_project.md#configuration). |
| `${project_name}_BUILD_TESTS` | *Boolean* | `ON` if being built as toplevel, `OFF` otherwise | `-Dmy-project_BUILD_TESTS=ON` | When `ON`, test executables for your project will be built and installed if your project has tests and [specifies a test framework](./cmake_data_config/properties/properties_list.md#test_framework). |

## Recommended .gitignore

``` .gitignore
.cache/
.vscode/
.mypy_cache/

build/
dep/
docs/api/

Doxyfile
conf.py
```
