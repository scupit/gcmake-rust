# TODO

There are a whole bunch of things which need doing. This is the place to list them.

- Rename `predefined_dependencies` to something more intuitive. These dependencies are not gcmake projects,
  but can be configured to work with gcmake by providing a 'yaml dependency glue' config. Those glue configs
  should be contained in a separate repository, should function as a sort of "registry" updateable by the
  gcmake tool.

## Priorities

- [ ] Automatically include the export header in generated compiled library files.
- [ ] Add a flag which prints whether a dependency or project can be trivially cross-compiled.
- [ ] Add CLI commands for cleaning and updating the dep-cache. Not exactly sure how updating should work yet.

## Configuration TODO

### General

Add way to detect when this project is being built as a subproject.
(Maybe `CMAKE_SOURCE_DIR` !== `CMAKE_CURRENT_SOURCE_DIR`)

Support for:

- Intel C/C++ compiler
- NVidia CUDA compiler?
- Emscripten?

- [ ] Add "library group" project type which allows building multiple libraries in the same
project, organized as *components*. See [SFML](https://github.com/SFML/SFML) for a good example of this.

### Quality of Life

- [ ] Add color printing. This will make output so much nicer to read.

### Compiler Cheat Sheet

- [ ] It would be great to have a cheat sheet full of per-compiler details. For each compiler this project
supports, the cheat sheet should detail:
  - [ ] Common and useful compiler flags, with explanations
  - [ ] Common and useful linker flags, if necessary
  - [ ] Commonly used defines, with explanations
  - [ ] An example list of flags for use as a starting point, per build configuration
  - [ ] An example list of defines for use as a starting point, per build configuration

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
- [ ] CURL
- [ ] Curses (ncurses)
- [ ] Doxygen
- [ ] FreeType
- [x] GLEW
- [ ] LibLZMA
- [ ] OpenAL-soft
- [x] OpenGL
- [ ] OpenSceneGraph (maybe) (NOTE: has cmake package config file)
- [ ] OpenSSL
- [ ] PNG
- [x] SDL2
- [ ] SQLite (3)
- [x] Threads
- [ ] TIFF
- [x] wxWidgets
- [ ] Vulkan
- [ ] ZLIB

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
- [ ] [imgui](https://github.com/ocornut/imgui)
- [x] [GLM (OpenGL Mathematics)](https://github.com/g-truc/glm)
- [x] [cxxopts](https://github.com/jarro2783/cxxopts)
- [x] [CLI11](https://github.com/CLIUtils/CLI11)
- [x] [ftxui](https://github.com/ArthurSonzogni/FTXUI)
- [x] [pugixml](https://github.com/zeux/pugixml)
- [ ] [mimalloc](https://github.com/microsoft/mimalloc)
- [ ] [magic_enum](https://github.com/Neargye/magic_enum)
- [ ] [argparse](https://github.com/p-ranav/argparse)

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
