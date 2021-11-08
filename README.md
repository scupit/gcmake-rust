# gcmake-rust
**TODO: Finish the README**

## Project goals
Configuration file: `cmake_data.yaml`

Used to generate: `CMakeLists.txt`.

**This project has three goals**:
  1. Provide a simple, readable, and intuitive build configuration format for general C and C++ projects.

CMake is a fantastic tool which covers almost all bases. However, writing a CMakeLists.txt is tedious, and
maintaining one well can become difficult due to the unintuitive syntax. I don't need all the configuration options
CMake has to offer for most projects either. *I wanted a tool that would allow me to abstract over CMake for
"the general case"*, so that projects could be easily created and configured without hassle in an easily readable manner.

  2. Use a uniform project structure

  3. Provide simple project management tools, such as project generation and file creation.

## How to use
`gcmake-rust help` or `gcmake-rust --help` will describe how to use the tool. 

`gcmake-rust new <project-name>` will walk you through generating a new project. The new project
will contain an *cmake_data.yaml* to be used as a starting point, as well as a *CMakeLists.txt* file generated
using the yaml file. 

Run `gcmake-rust` in the project root to generate CMakeLists.txt for the project and its subprojects
using *cmake_data.yaml*. You can also use `gcmake-rust <project-root-dir>` to do the same from
outside the project root.

# cmake_data.yaml

### TODO: Document the configuration of cmake_data.yaml in another markdown file.

**Linking format**:
- ProjectName::LibName
- ProjectName::{ FirstLibName, SecondLibName, ... }

Subprojects cannot currently be linked to each other. This restriction makes generating CMakeLists.txt easier
by preventing circular dependencies. However, this might change in the future.

The linking restriction might change because it is sometimes useful to link a general purpose
subproject to other subprojects. Think formatting libraries, project-wide macros, etc. If a subproject
can reference its parent project, then it should be able to link with *other subprojects in the parent project only.*
In doing this, I'll also have to check for circular dependencies. Luckily making a dependency graph
with all the project data already there shouldn't be too hard.

# TODO

## Configuration TODO

### General
Add way to detect when this project is being built as a subproject.
(Maybe `CMAKE_SOURCE_DIR` !== `CMAKE_CURRENT_SOURCE_DIR`)

### Targets
- **Namespaced output targets**
- Support for header-only libraries.
- Defines and flags per target.

### Installation
- Configure installation
- Export configuration (figure out how this is different from installation)

## Generation TODO
- Add ability to generate header, source, and template-impl files. Must support C and C++.
- Generate *.gitignore* file if it doesn't exist. Ignore:
  - .vscode/
  - build/

Support for:
- `.clang-format`
- `.clang-tidy`
- valgrind?

## CLI TODO

- `dep-graph` command which prints a dependency graph per output target
- `dep-graph <target>` command which prints a dependency graph for the given target
- `show-defines <config-name>` command which prints the defines specified by the buildsystem for a given configuration.
- `show-flags <config-name>` command which prints the compiler flags specified by the buildsystem for a given configuration.
- `new clang-format` command which generates a .clang-format if it doesn't exist.

## External libraries TODO
Add support for bringing external libraries into the project.

Types of libraries which need support, from easiest to hardest:
  1. Another gcmake (this project) project
  2. Project which already has a [pre-written cmake find module](https://cmake.org/cmake/help/v3.22/manual/cmake-modules.7.html#find-modules)
  3. CMake project which can be added using *add_subdirectory*
  4. CMake project which can't use add_subdirectory (must be built and installed on the system separately)
  5. Non-CMake project

### Dependency components
- Git Repo information (or download URL information)
- Library name (exact match)
- Library components (target names)

### Libraries I want to explicitly support for convenience

Pre-written CMake find modules:
* Boost
* CURL
* Curses (ncurses)
* Doxygen
* FreeType
* GLEW
* OpenGL
* OpenSSL
* SDL (not sure why these are listed separately in the docs, I'll have to research that.)
  * SDL_image
  * SDL_mixer
  * SDL_net
  * SDL_sound
  * SDL_ttf
* SQLite (3)
* wxWidgets
* Vulkan
* ZLIB

Other CMake projects:
* [nlohmann json](https://github.com/nlohmann/json)
* [SFML](https://www.sfml-dev.org/)
* [JUCE](https://juce.com/)
* [yaml-cpp](https://github.com/jbeder/yaml-cpp)
* [GLFW](https://www.glfw.org/)
* [OpenCV](https://opencv.org/)
* [ffmpeg](https://www.ffmpeg.org/)
* [TensorFlow](https://www.tensorflow.org/)
* [imgui](https://github.com/ocornut/imgui)
* [Asio](https://think-async.com/Asio/)
* [GLM (OpenGL Mathematics)](https://github.com/g-truc/glm)
