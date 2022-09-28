# Project Structure

> This page describes the GCMake project structure and explains why certain choices were made.
>
> **TODO:** Add more detail, such as why only one library can be built per single project.

## The Structure

| Directory | Description |
| --------- | ----------- |
| `src/FULL_INCLUDE_PREFIX/` | Project source files (*.c*, *.cpp*) go here. The files will be recursively found and added to the build. |
| `include/FULL_INCLUDE_PREFIX/` | Project source files (*.h*, *.hpp*) go here. The files will be recursively found and added to the build. |
| `template-impls/FULL_INCLUDE_PREFIX/` | Template implementation files (*.tpp*) go here, if you use them. The files will be recursively found and added to the build. |
| `resources/FULL_INCLUDE_PREFIX/` | Any assets needed by the project at runtime go here. The `resources/` directory recursively copied into the build directory at build time, and is also fully installed as part of the installation tree. |
| `subprojects/` | Subprojects go in this directory. Each subproject in this directory is automatically found and configured by GCMake as a subproject. Instead of creating these by hand, use `gcmake-rust new subproject 'your-subproject-name'` |
| `tests/` | Test projects go in this directory. Each test project in this directory is automatically found and configured by GCMake. Instead of creating these by hand, use `gcmake-rust new test 'your-test-name'` |
| `cmake/` | **AUTO-CONFIGURED:** The directory which contains GCMake's CMake utility scripts. This is auto-generated every time GCMake configures the project. This should be committed in your source control. |

| File | Description |
| ---- | ----------- |
| `cmake_data.yaml` | This is the [GCMake configuration file](cmake_data_config/cmake_data.md). |
| `pre_build.py` \| `pre_build.c` \| `pre_build.cpp` | **Optional** [pre-build script](pre_build_scripts.md) |
| `LICENSE` \| `LICENSE.md` \| `LICENSE.txt` | The file specifying the project's license. This will also be embedded in some graphical installers. |
| `CMakeLists.txt` | **AUTO-CONFIGURED:** The file which facilitates CMake builds. This paired with the `cmake/` directory is what makes the magic happen. This should be committed in your source control. |
| `Config.cmake.in` | **AUTO-CONFIGURED:** The configuration template for a CMake installation. This file allows CMake to discover an installation of your project, and as a result allows other CMake projects to use your project installation as a CMake dependency with just a single *find_package* call. This should be committed in your source control. |
