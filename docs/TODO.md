# TODO

There are a whole bunch of things which need doing. This is the place to list them.

- Rename `predefined_dependencies` to something more intuitive. These dependencies are not gcmake projects,
  but can be configured to work with gcmake by providing a 'yaml dependency glue' config. Those glue configs
  should be contained in a separate repository, should function as a sort of "registry" updateable by the
  gcmake tool.
- [x] Set fetchcontent_quiet to true after first config if all subprojects/dependencies are cloned correctly.
  That way git info doesn't clog up the output prompt each configure.

## Priorities

- [x] For compiled libraries (`StaticLib`, `SharedLib`, and `CompiledLib`), links must be explicitly
        categorized as 'public' or 'private'. Add a section in the docs to explain this. See
        [this great StackOverflow answer](https://stackoverflow.com/questions/26037954/cmake-target-link-libraries-interface-dependencies)
        for the difference between PUBLIC, PRIVATE, and, INTERFACE in CMake.
- [x] Header-only library support. `HeaderOnly`
- [x] When generating CMakeLists.txt, make sure to use correct access specifiers.
        Headers should be PRIVATE for executables, INTERFACE for header-only libraries, and
        PUBLIC for compiled libraries. Links should be PRIVATE for executables, INTERFACE for
        header-only libraries, and PUBLIC or PRIVATE for compiled libraries (see above).
- [x] Change `Library` output type to `CompiledLib` or `BinaryLib`, so it won't be confused with
        header-only libraries.
- [x] When generating a library project, name the entry_file the same as the project name.
- [x] Add a separate docs page explaining the linking system, and how links are inherited from
        compiled libraries.
- [ ] **Propagate DLL copies from installed gcmake projects on Windows**. *Projects just*
        *using CMake should be able to find_package a gcmake project and have the proper DLLs be copied*
        *to a build directory automatically*. DLL installation can't be guaranteed in this instance.
        However, DLL installation is already guaranteed for gcmake projects, since dependencies which
        are gcmake projects are required to be built as subproject dependencies.
- [x] CPack installer generation

[Available CPack Generators](https://cmake.org/cmake/help/latest/cpack_gen/archive.html)

[CPack Docs Page](https://cmake.org/cmake/help/latest/module/CPack.html#variable:CPACK_PACKAGE_DESCRIPTION_SUMMARY)

- [ ] CTest tests and *\<project name\>_BUILD_TESTS* variable. Tests should be in *tests/* directory,
        Each test should have its own directory.
- [ ] Generate a placeholder header and source file when creating a new compiled library. Currently,
        the intial CMakeLists generation step for the new compiled library project fails because the
        project doesn't contain any source files; just the 'entry point' header.
- [ ] Separate the `new` command into `project` and `subproject`. This opens the door for adding other
        things, such as `new test`.
-  Improvements to installers, such as:
  - [ ] Ability to specify an installer icon (where applicable)
  - [ ] Name the uninstaller `Uninstall_<project name>` (where applicable)
- [ ] Expose system endianness to the project using `CMAKE_<LANG>_BYTE_ORDER`.
- [ ] Figure out cross-compiling

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
- [x] Separate *flags* build config option into *compiler_flags* and *linker_flags*.

### Quality of Life

- [ ] Add color printing. This will make output so much nicer to read.

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
- [x] Support for header-only libraries.
- [x] Defines and flags per target.

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
- [x] Ability to specify linked dependencies as a map of project names, each with its own dependency list.
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

#### new

- [ ] `new clang-format` command which generates a default .clang-format if it doesn't exist.

### External libraries TODO

- [x] Remove default `latest_stable_release_tag` in dependency yaml configuration. This project shouldn't manage default lib versions.
- [x] Add support for bringing external libraries into the project.

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
- [x] GLEW
- [ ] OpenAL-soft
- [X] OpenGL
- [ ] OpenSceneGraph (maybe) (NOTE: has cmake package config file)
- [ ] OpenSSL
- [X] SDL2
- [ ] SQLite (3)
- [X] Threads
- [x] wxWidgets
- [ ] Vulkan
- [ ] ZLIB

Other CMake projects:

- [ ] [Qt6](https://www.qt.io/product/qt6)
- [x] [nlohmann json](https://github.com/nlohmann/json)
- [x] [SFML](https://www.sfml-dev.org/)
- [x] [fmt](https://github.com/fmtlib/fmt)
- [ ] [JUCE](https://juce.com/)
- [ ] [yaml-cpp](https://github.com/jbeder/yaml-cpp)
- [x] [glfw3](https://www.glfw.org/)
- [ ] [OpenCV](https://opencv.org/)
- [ ] [ffmpeg](https://www.ffmpeg.org/)
- [ ] [TensorFlow](https://www.tensorflow.org/)
- [ ] [imgui](https://github.com/ocornut/imgui)
- [x] [GLM (OpenGL Mathematics)](https://github.com/g-truc/glm)

Other projects:

- [x] [stb](https://github.com/nothings/stb)
