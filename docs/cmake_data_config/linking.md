# Linking

> This page explains how "linking" works in GCMake. See the
> [output.link property](properties/output.md#link) for info on defining links and
> [propagation rules](properties/output.md#propagation).

**TODO:** make a document on target selectors (used to select targets or projects in some commands).

## Formats

Link specifiers can be given in either of two formats:

1. Single format: `namespace::libname`
2. Multi format: `namespace::{ lib1, lib2 }`

Link specifiers may also be prefixed with [system specifiers](data_formats.md#system-specifier):

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
