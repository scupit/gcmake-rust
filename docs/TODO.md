# TODO

There are a whole bunch of things which need doing. This is the place to list them.

- Rename `predefined_dependencies` to something more intuitive. These dependencies are not gcmake projects,
  but can be configured to work with gcmake by providing a 'yaml dependency glue' config. Those glue configs
  should be contained in a separate repository, should function as a sort of "registry" updateable by the
  gcmake tool.

## Priorities

- [ ] Add CLI commands for cleaning and updating the dep-cache. Not exactly sure how updating should work yet.
- [ ] Basic documentation generation support:
  - [x] Documentation with [Doxygen](https://www.doxygen.nl/index.html)
  - [x] Document library entry files.
  - [x] Ability to use a dummy *.hpp* file for home page generation
  - [x] Ability to document just the header files.
  - [x] Validator to ensure certain important/required lines are present in *Doxyfile.in*, such as `PROJECT_NAME = @PROJECT_NAME@`
  - [x] Default *Doxyfile.in* generator command. Something like `gen-default doxyfile`.
  - [Doxygen + Sphinx + Breathe + read-the-docs](https://devblogs.microsoft.com/cppblog/clear-functional-c-documentation-with-sphinx-breathe-doxygen-cmake/) documentation system
  - [ ] Additional markdown page usage with sphinx
  - [ ] Make docs/ dir part of CMake installation
  - [ ] Documentation for the new `documentation` configuration options.

## Configuration TODO

### General

Support for:

- Intel C/C++ compiler?
- NVidia CUDA compiler?

- [ ] Now that minimal installs are implemented, add ability to specify exactly which executables are installed.
- [ ] Use CMake 3.25 ([Release Notes Here](https://cmake.org/cmake/help/v3.25/release/3.25.html)) for these features:
  - *LINUX* and *CMAKE_HOST_LINUX* variables
- [ ] Maybe support cppfront *.h2* headers, once they become a little more "stable".

### CLI TODO

#### dep-graph

The command set for viewing dependency graph info.

- [ ] `dep-graph` command which prints a dependency graph for each target in the current project.
- [ ] `dep-graph <target>` command which prints a dependency graph for the given target.

#### show

The command set for viewing project metadata.

- [ ] `show linkable` shows available targets per subproject and dependency for the current project.
        Allow a `--from <project-or-dep-name>` flag to specify that only targets/libraries from
        the given subproject/dependency should be printed.
- [ ] `show defines <config-name>` prints the defines specified by the buildsystem for a
        given configuration.
- [ ] `show flags <config-name>` prints the compiler flags specified by the buildsystem for
        a given configuration.
- [ ] `show metadata <project-path>` prints metadata for a project.
- [ ] `show structure <project-path>` prints the full structure of a project, starting from the toplevel
        one. The given project should be marked.

#### check

- [ ] `check config` displays whether the cmake_data.yaml is correct and works with the current project.
- [ ] `check cmake-version` gets the current CMake version and the required CMake version, and whether
        the current CMake version is new enough.

### Libraries I want to explicitly support for convenience

- Ideally anything listed in the [awesome-cpp repository](https://github.com/fffaraz/awesome-cpp) which either
  supports cross-platform CMake builds out of the box or is fairly easy to add. That repository is a fantastic
  list.

#### Pre-written CMake find modules

- [ ] Boost
- [ ] BZip2
- [x] CURL
- [ ] Doxygen
- [x] GLEW
- [ ] LibLZMA
- [ ] OpenAL-soft
- [x] OpenGL
- [ ] OpenSceneGraph (maybe) (NOTE: has cmake package config file)
- [x] OpenSSL
- [ ] PNG
- [x] SDL2
- [x] SQLite (3)
- [x] Threads
- [ ] TIFF
- [x] wxWidgets
- [ ] Vulkan
- [x] ZLIB

#### Other CMake projects

- [ ] [Qt6](https://www.qt.io/product/qt6)
- [x] [nlohmann json](https://github.com/nlohmann/json)
- [x] [SFML](https://www.sfml-dev.org/)
- [x] [fmt](https://github.com/fmtlib/fmt)
- [ ] [JUCE](https://juce.com/)
- [x] [yaml-cpp](https://github.com/jbeder/yaml-cpp)
- [x] [glfw3](https://www.glfw.org/)
- [ ] [OpenCV](https://opencv.org/)
- [ ] [ffmpeg](https://www.ffmpeg.org/)
- [ ] [TensorFlow](https://www.tensorflow.org/)
- [x] [GLM (OpenGL Mathematics)](https://github.com/g-truc/glm)
- [x] [cxxopts](https://github.com/jarro2783/cxxopts)
- [x] [CLI11](https://github.com/CLIUtils/CLI11)
- [x] [ftxui](https://github.com/ArthurSonzogni/FTXUI)
- [x] [pugixml](https://github.com/zeux/pugixml)
- [ ] [mimalloc](https://github.com/microsoft/mimalloc)
- [x] [magic_enum](https://github.com/Neargye/magic_enum)
- [x] [argparse](https://github.com/p-ranav/argparse)
- [x] [FreeType](https://freetype.org/index.html)
- [ ] [drogon](https://github.com/drogonframework/drogon) (This looks like it might take some work)
- [x] [re2](https://github.com/google/re2)
- [x] [zstd](https://github.com/facebook/zstd) CMake project, but use custom find module
- [ ] [Hyperscan](https://github.com/intel/hyperscan)
- [x] [Crow](https://github.com/CrowCpp/Crow)
- [ ] [libharu](https://github.com/libharu/libharu)
- [ ] [concurrencpp](https://github.com/David-Haim/concurrencpp)
- [ ] [opus](https://opus-codec.org/)
- [ ] [Vorbis](https://xiph.org/vorbis/)

#### Support when FetchContent ready

- [ ] [GLM (The actual repo, not a fork)](https://github.com/g-truc/glm)

#### Testing Frameworks

- [x] [Catch2](https://github.com/catchorg/Catch2)
- [x] [doctest](https://github.com/doctest/doctest)
- [x] [GoogleTest](https://github.com/google/googletest)

#### Cryptography libraries

- [ ] [botan](https://github.com/randombit/botan)
- [ ] [crpytopp](https://github.com/weidai11/cryptopp)

#### Other projects

- [x] [stb](https://github.com/nothings/stb)
- [x] [imgui](https://github.com/ocornut/imgui)
- [x] [brotli](https://github.com/google/brotli)
