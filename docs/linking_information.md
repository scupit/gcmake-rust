# Linking

> This documentation page contains explanations for several important linking concepts relevant to gcmake.

If you're looking for how to specify links in [cmake_data.yaml](cmake_data.md), check out
[linking to outputs in cmake_data.yaml](cmake_data.md#output-link).

If you're looking for link specifier formats, check out the sections on
[output link specification](cmake_data.md#linksection) and
[link specifier string](cmake_data.md#link-specifier-string).

## Linked Library Inheritance

> [This great answer from StackOverflow](https://stackoverflow.com/questions/26037954/cmake-target-link-libraries-interface-dependencies)
> explains this topic in the context of CMake very well.

When linking libraries to an output, you need to know two things:

1. Whether the linked library will be *compiled with* your output.
2. Whether the linked library is exposed as part of your output's inheritable public interface.
    (i.e. Your output is a library, and the linked library is #included in your library's header files).

Knowing this, the way a library is linked to your output can be placed into one of three categories:

| Compiled with your code | Included in your code's public header files | Link category |
| :---------------------: | :-----------------------------------------: | :------------ |
| ✅                     | ✅                                          | `PUBLIC`      |
| ✅                     |                                             | `PRIVATE`     |
|                         | ✅                                         | `INTERFACE`   |

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
