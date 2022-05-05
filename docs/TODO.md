# TODO

There are a whole bunch of things which need doing. This is the place to list them.

- Rename `predefined_dependencies` to something more intuitive. These dependencies are not gcmake projects,
  but can be configured to work with gcmake by providing a 'yaml dependency glue' config. Those glue configs
  should be contained in a separate repository, should function as a sort of "registry" updateable by the
  gcmake tool.
- Set fetchcontent_quiet to true after first config if all subprojects/dependencies are cloned correctly.
  That way git info doesn't clog up the output prompt each configure.

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

- [ ] Add color printing. This will make output so much nicer to read through.

### Refactoring

- [x] Unify file path cleaning, so that paths are always relative to the project root, in the project root,
      and have no leading slashes. Ex: `pre-build.cpp` instead of `/pre-build.cpp`

### Pre-build script

- [x] Add support for a pre-build C++, C, or Python 3 script. The script should be automatically built and
      run before each recompilation.
- [x] `resources` directory in each project root is copied into the build tree before the actual build.
      **TODO: need to be 100% sure this runs after the pre-build scripts somehow. Maybe run it as POST_BUILD on the pre-build target.**

### Targets

- [x] **Namespaced output targets**
- [ ] Support for header-only libraries.
- [ ] Defines and flags per target.

### Testing

- [ ] Add support for automated testing with CMake's built-in ctest.

### Installation

- [x] Configure installation
- [x] Export configuration (figure out how this is different from installation)
- [x] Automatically create a CMake package config file (\<projectName>Config.cmake)

### Generation TODO

- [x] Add ability to generate header, source, and template-impl files. Must support C and C++.
- [ ] Generate *.gitignore* file if it doesn't exist. Ignore:
  - .vscode/
  - build/
- [ ] Ability to specify linked dependencies as a map of project names, each with its own dependency list.
- [ ] (MAYBE) Add all warning flags to Release builds as well.

Support for:

- [ ] `.clang-format`
- [ ] `.clang-tidy`
- [ ] valgrind? (not sure if this needs a configuration file or not, needs research)
- [ ] CPack installer generation

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

#### new

- [ ] `new clang-format` command which generates a default .clang-format if it doesn't exist.

### External libraries TODO

- **Move dependency yaml configurations to separate files, in their own repository**, since other library
  compatibility configurations shouldn't be tied to the tool itself. Add command `update-compat` which clones
  or pulls that repo to *HOME/.gcmake/gcmake-lib-configs*. This tool can then read from there (or a given
  custom location with a new flag).
- [x] Remove default `latest_stable_release_tag` in dependency yaml configuration. This project shouldn't manage default lib versions.
- [ ] Add support for bringing external libraries into the project.

### IMPORTANT NOTE

**Currently, supported external libraries can only be linked statically. Need to add support for**
**copying shared libraries to the correct location.**

Types of libraries which need support, from easiest to hardest:

  1. Another gcmake (this project) project
  2. Project which already has a [pre-written cmake find module](https://cmake.org/cmake/help/v3.22/manual/cmake-modules.7.html#find-modules)
  3. CMake project which can be added using *add_subdirectory*
  4. CMake project which can't use add_subdirectory (must be built and installed on the system separately)
  5. Non-CMake projects which can be downloaded

### Libraries I want to explicitly support for convenience

Pre-written CMake find modules:

- [ ] Boost
- [ ] CURL
- [ ] Curses (ncurses)
- [ ] Doxygen
- [ ] FreeType
- [ ] GLEW
- [ ] OpenGL
- [ ] OpenSSL
- [ ] SDL (not sure why these are listed separately in the docs, I'll have to research that.)
  - [ ] SDL_image
  - [ ] SDL_mixer
  - [ ] SDL_net
  - [ ] SDL_sound
  - [ ] SDL_ttf
- [ ] SQLite (3)
- [ ] wxWidgets
- [ ] Vulkan
- [ ] ZLIB

Other CMake projects:

- [x] [nlohmann json](https://github.com/nlohmann/json)
- [x] [SFML](https://www.sfml-dev.org/)
- [x] [fmt](https://github.com/fmtlib/fmt)
- [ ] [JUCE](https://juce.com/)
- [ ] [yaml-cpp](https://github.com/jbeder/yaml-cpp)
- [ ] [GLFW](https://www.glfw.org/)
- [ ] [OpenCV](https://opencv.org/)
- [ ] [ffmpeg](https://www.ffmpeg.org/)
- [ ] [TensorFlow](https://www.tensorflow.org/)
- [ ] [imgui](https://github.com/ocornut/imgui)
- [ ] [GLM (OpenGL Mathematics)](https://github.com/g-truc/glm)

Non-CMake projects:

- [ ] [Asio](https://think-async.com/Asio/)
