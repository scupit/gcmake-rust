# CppFront Integration

> This page explains GCMake's support for [cppfront](https://github.com/hsutter/cppfront)
> and how to use *.cpp2* files in your projects.

CppFront is Herb Sutter's experimental compiler from a potential second, alternative C++ syntax
to today's modern C++ syntax. For a full explanation on cppfront, see the
[cppfront GitHub repository](https://github.com/hsutter/cppfront) and the
["Can C++ be 10x Simpler and Safer?"](https://www.youtube.com/watch?v=ELeZAKCN4tY) CppCon 2022 talk.

For configuration and installation instructions, see the
[gcmake dependency configuration README for cppfront](/gcmake-dependency-configs/cppfront/README.md).

## Quick Links

- [My CppFront CMake wrapper](https://github.com/scupit/cppfront-cmake-wrapper): This is the compatibility repository which allows GCMake to easily use CppFront. It is also just a standalone way to build and install CppFront on your system.
- [gcmake-dependency-configs/cppfront](https://github.com/scupit/gcmake-dependency-configs/tree/develop/cppfront)

## Using CppFront in a GCMake Project

CppFront is essentially supported out-of-the-box in GCMake. To use it, just:

1. List *cppfront* as a predefined_dependency of your project.

``` yaml
# ... rest of project configuration
predefined_dependencies:
  cppfront:
    git_tag: master
```

2. Change any (but not necessarily all) *.cpp* file extensions to *.cpp2*. That includes the `entry_file` of any executable output as ell as and *pre_build.cpp* scripts. No issues should arise because *.cpp2* files allow both "C++1" and "C++2" syntax mixed within the same file.

When not cross-compiling, the CppFront compiler will be built as part of your project by default and
wll transform all *.cpp2* files before the rest of your project is built. If you'd rather use a pre-existing
CppFront installation on your system, set the `EMBED_CPPFRONT` CMake cache variable to `OFF` before
configuring. This will use *find_package* to search for cppfront instead of building it as a subproject.

``` sh
cmake -DEMBED_CPPFRONT=OFF
```

### Cpp2 Project Generation

*Cpp2* is already listed as a language option in the subproject and root project generators.
Just step through one of the initializers to see it.

``` sh
gcmake-rust new root-project 'my-project-name'
# OR (run inside an existing GCMake project)
gcmake-rust new subproject 'my-subproject-name'
```

You can also specify *--cpp2* explicitly to use it and skip the language prompt.

``` sh
gcmake-rust new root-project 'my-project-name' --cpp2 
# OR (run inside an existing GCMake project)
gcmake-rust new subproject 'my-subproject-name' --cpp2
```

These are really just C++ projects which make use of CppFront's cpp2. You'll be able to use regular cpp
files as usual as well.

### Cpp2 File Generation

``` sh
gcmake-rust gen-file cpp2 SomeFile Another/NestedFile
```

## CppFront with Emscripten

CppFront works just fine with Emscripten, except for one catch.
**When using Emscripten, cppfront cannot be embedded in the project.** `EMBED_CPPFRONT` will be set
to `OFF` by default, and cannot be turned on. CMake will always search for an existing installation
on the system using *find_package*.
This is necessary because
[Emscripten's file system won't write files to disk by default](https://emscripten.org/docs/api_reference/Filesystem-API.html#file-systems).
The file system must be explicitly mapped from within the program code itself. Rather than deal with this,
just use an existing installation. To build and install CppFront on your system, see the
[build/install instructions in my CppFront CMake wrapper repository](https://github.com/scupit/cppfront-cmake-wrapper#default-build-and-install).

## Installing CppFront on your system

``` sh
git clone 'git@github.com:scupit/cppfront-cmake-wrapper.git'
cd cppfront-cmake-wrapper
cmake -B build/ -DCMAKE_BUILD_TYPE=Release # ... any other CMake options such as -G 'Ninja'
cmake --build build/ --parallel
sudo cmake --install build/ # Or run in an Administrator prompt in Windows
```

Optionally check out the
[in-depth build/install instructions in my CppFront CMake wrapper repository](https://github.com/scupit/cppfront-cmake-wrapper#default-build-and-install).
