# Linking

> This page explains how "linking" works in GCMake. See the
> [output.link property](properties/output.md#link) for info on defining links and
> [propagation rules](properties/output.md#propagation).

**TODO:** make a document on target selectors (used to select targets or projects in some commands).

## Formats

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

## Link Order

Some multi-library dependencies require their libraries to be linked in a certain order on some compilers.
Linking those libraries in an incorrect order would result in a link-time error. That's annoying, so
**GCMake automatically links libraries in the correct order so that those errors don't happen.** You don't
have to worry about link order when specifying links.

## Namespace Scope and Resolution

Namespaces are resolved relative to the "current project", which includes subprojects and test projects.
A project has immediate access to its subprojects as well as any dependencies imported in the root project.

Namespaces can be nested as long as the namespace resolves to some project or subproject in the
current project tree (either the root projects or its subprojects). However, namespaces cannot be nested when
resolving a dependency. Trying to nest a dependency namespace will always result in an error because a
consumer shouldn't have to know a dependency's project structure to use its targets.

Namespaces are resolved recursively using the following:

| Value | Action |
| ----- | ------ |
| `super` or `parent` | Steps up one project from a subproject to its parent. If the current namespace context is the root project, an error is thrown. |
| `root` | Sets the current namespace context to the root project. This is often useful for when you want to link to one of your project's library outputs but don't want to tie the link to your project structure. For example, `root::{ any, library, in, the, whole, project, tree }` |
| *Any other String* | Searches for the name of a subproject, predefined dependency, or GCMake dependency which matches the given string. If one is found, then the namespace context moves to the found subproject or dependency. Otherwise an error is thrown. |

## Target Scope

**The project context resolved from a namespace has access to all outputs on an equal level or lower in**
**the project tree.**

That's why specifying the `root` namespace gives you access to every output item in the entire project tree.

### Resolution examples

For example, take a project with the following structure:

``` txt
my-lib
  \- subprojects
      \- first-lib
        \- subprojects
          \- nested-lib
      \- second-lib
      \- third-lib
```

And the following dependency configuration:

``` yaml
gcmake_dependencies:
  some-gcmake-dep:
    # ...
predefined_dependencies:
  SFML:
    # ...
```

Assume that every project builds a single library with the exact same name as the project itself.
With those assumptions in mind, here are some valid link specifiers:

``` yaml
# my-project
output:
  my-lib:
    output_type: CompiledLib
    entry_file: my-lib.hpp
    link:
      # Need to categorize links because this is a compiled library. Links don't need to be categorized
      # for executables or header-only libraries.
      private:
        - first-lib::{ first-lib, nested-lib }
        # first-lib::nested-lib::nested-lib would also link to the nested lib.
        # However due to the target scope rule, first-lib has access to the nested-lib
        # output, which is why the above multi-format specifier works.
        - second-lib::second-lib
        - SFML::{ system, window, ((windows)) main } # Notice the ((windows)) system specifier
        - some-gcmake-dep::{ some-lib, another-lib }
        # This would fail, because we can't nest namespaces when resolving a dependency.
        # some-gcmake-dep::some-lib::some-lib
```

``` yaml
# subprojects/first-lib/subprojects/nested-lib
output:
  nested-lib:
    output_type: CompiledLib
    entry_file: nested-lib.hpp
    link:
      private:
        - parent::first-lib # Same as super::first-lib
        - root::second-lib
        # Same as above -> root::second-lib::second-lib
        # also the same -> parent::parent::second-lib
        # another same  -> parent::parent::second-lib::second-lib
        - SFML::{ system, window, ((windows)) main }
        - some-gcmake-dep::{ some-lib, another-lib }
```

## Conditional Dependencies

[gcmake_dependencies](./properties/gcmake_dependencies.md) and
[predefined_dependencies](./properties/properties_list.md#predefined_dependencies) are lazy-loaded.
This means that the CMake configuration will only attempt to import a dependency if the dependency
will actively be used by the current configuration.

Keeping that in mind, here's an example where [features](./properties/features.md) are used to
make [fmt](https://github.com/fmtlib/fmt) an optional dependency.

``` yaml
features:
  use-fmt:
    default: false

output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    link:
      # When the use-fmt feature is disabled, fmt::fmt will not be linked to my-exe.
      # In that case, nothing in the project ever makes use of fmt, so CMake will never
      # attempt to clone or search for fmt. If use-fmt is enabled, then CMake will clone
      # it and link here as expected. That's essentially how optional dependencies work
      # in GCMake.
      - (( feature:use-fmt )) fmt::fmt
```

## General CMake Linking Explanation

> [This great answer from StackOverflow](https://stackoverflow.com/questions/26037954/cmake-target-link-libraries-interface-dependencies)
> explains this topic in the context of CMake very well.

When linking libraries to an output, you need to know two things:

1. Whether the linked library will be *compiled with* your output.
2. Whether the linked library is exposed as part of your output's inheritable public interface.
    (i.e. Your output is a library, and the linked library is #included in your library's header files).

Knowing this, the way a library is linked to your output can be placed into one of three categories:

| Compiled with your code | Included in your code's public header files | Link category |
| :---------------------: | :-----------------------------------------: | :------------ |
| ✅ | ✅ | `PUBLIC`|
| ✅| | `PRIVATE` |
| | ✅ | `INTERFACE` |

### Link Categories by Output Item Type

Links to an `Executable` are always *private* because an executable is a program in its final form.
Every library linked to an executable is needed by the executable. However, an executable does not pass
on any headers or functionality which can be inherited by another program. Libraries linked to the
executable will only ever be needed by the executable, and therefore fall into the *private* link category.

Links to a header-only library (`HeaderOnlyLib`) are always *interface* because header-only libraries are
not compiled to create an output. However, the headers in a header-only library are always needed
by any output which includes the header-only library. This means the libraries "linked" to the
header-only library are only ever needed when compiling outputs which *depend on the header-only library*,
and therefore fall into the *interface* link category.

Links to a compiled library (`StaticLib`, `SharedLib`, and `Library`) can either be *public* or *private*.

For example, say we're linking *nlohmann_json* to our static library called *my-static-lib*. If
nlohmann_json is #included in any of our library's headers (and/or template impl files), then that means
any library which depends on my-static-lib also needs knowledge of nlohmann_json's headers (i.e. its
public interface). In this case, nlohmann_json needs to be *public* linked to my-static-lib, so
outputs which consume my-static-lib know they require nlohmann_json's headers.

On the other hand, if nlohmann_json is only #included in my-static-lib's source (.c, .cpp) files, then
nlohmann_json is only needed internally by the library. my-static-lib's headers contain no trace of
nlohmann_json, so consumers of my-static-lib do not need any knowlege of nlohmann_json. It's only
needed as part of my-static-lib's private implementation. In this case, nlohmann_json needs to be
*private* linked to my-static lib.
