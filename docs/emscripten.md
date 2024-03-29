# Using Emscripten

> This page details how to use [Emscripten](https://emscripten.org/) to compile GCMake programs and
> some details to keep in mind when doing so.

**NOTE:** This document assumes [Emscripten was installed using emsdk](https://emscripten.org/docs/getting_started/downloads.html#installation-instructions-using-the-emsdk-recommended),
which is the recommended install method.

> Also, **NodeJS** is required to run built emscripten "executables" (which are .js files) locally.
> This means that pre-build scripts will not be run unless you have NodeJS installed. If that isn't
> an issue, then you don't need NodeJS.

## General Notes

> [Emscripten settings flags reference](https://github.com/emscripten-core/emscripten/blob/main/src/settings.js)

### Asyncify

Projects generated by GCMake will use [Emscripten's asyncify](https://emscripten.org/docs/porting/asyncify.html)
by default.

``` yaml
# Example usage of the flag. Generated projects will use this flag for Emscripten in all build
# configurations by default.
build_configs:
  Debug:
    Emscripten:
      link_time_flags:
        - -sASYNCIFY
```

I opted to make this the default for now so that new projects "just work". You can generate a new project
and make asynchronous calls without any additional configuration. However,
*it's important to note that Asyncify increases code size and incurs at least a small performance penalty*.
**If you don't need asynchronous calls to JS and want smaller code, it's recommended to remove the `-sASYNCIFY` flag from your build configuration.**

## Compiling using Emscripten + CMake

Instead of specifying Emscripten's compilers directly to CMake, you must use [Emscripten's toolchain file](https://github.com/emscripten-core/emscripten/blob/main/cmake/Modules/Platform/Emscripten.cmake)
`Emscripten.cmake`. This file is located somewhere in the Emscripten install directory, usually
somewhere like `YOUR-emdsk-ROOT/upstream/emscripten/cmake/Modules/Platform/Emscripten.cmake`.

Run the command `which emcc` (or `where emcc` on Windows) to get an idea of where your emsdk
root directory is.

On first CMake configure, specify the Emscripten toolchain file to CMake. You should be all set, and
your build should work.

### Why the toolchain file?

Emscripten is a cross-compiler. When cross-compiling,
[CMake toolchain files](https://cmake.org/cmake/help/latest/manual/cmake-toolchains.7.html#introduction)
are used to tell CMake about compiler and target environment details which can't automatically be inferred
or must be explicitly customized. Emscripten's toolchain file both makes Emscripten compatible with CMake's
features and sets up its cross-compilation environment all in one step.

## EMSCRIPTEN_MODE

GCMake specifies two Emscripten modes: `WITH_HTML` and `NO_HTML`.

> `WITH_HTML` is the default `EMSCRIPTEN_MODE`.

| Mode | Description |
| ---- | ----------- |
| `WITH_HTML` | Builds the .js and .wasm files, and automatically creates a .html wrapper file for running the .js and .wasm in the browser. **NOTE** that *the .js file can often also be run with NodeJS*. |
| `NO_HTML` | Only the .js and .wasm files are built. |

## Running the Build Files with NodeJS

The .js file produced by an Emscripten build can often run locally using NodeJS. For example:
`node your-build-dir/bin/Debug/your-exe.js`. Just NOTE that this isn't always the case.

## Running the Built Files in the Browser

> Your project must produce an HTML file to be run in the browser. Make sure `EMSCRIPTEN_MODE`
> is set to `WITH_HTML`.

**If you plan to debug your app using source maps, read ["debugging with WASM source maps"](#debugging-with-wasm-source-maps) first.**

You can run your project in a browser by hosting the build output directory in a http server:

``` sh
# Hosts your-build-directory/ at http://localhost:8080/
python -m http.server --directory your-build-directory/ --bind localhost 8080
```

or by using Emscripten's built-in runner tool `emrun`:

``` sh
emrun bin/Debug/your-exe.html
```

then opening the built HTML file in the browser. Example URL:
`http://localhost:8080/bin/Debug/your-exe.html`

### Debugging with WASM Source Maps

Emcripten builds can be debugged in the browser using .wasm source maps (generated when using `-gsource-maps`).

**CAVEAT:** Source maps contain the *relative path from the build output directory to your source files.*
This means that **your server must host a directory which contains both the project source directory and the build directory.**

For example, with the directory structure:

``` txt
/home/sky/
  \- Documents/
      \- my-build-dir/
          \- bin/
              \- Debug/
                \- my-exe.html
                \- my-exe.js
                \- my-exe.wasm
                \- my-exe.wasm.map
      \- my-project/
        \- src/
        \- include/
        \- main.cpp
```

your server could be hosted at `/home/sky/Documents`:

``` sh
python -m http.server --directory '/home/sky/Documents/' --bind localhost 8080
```

and the build acessed at `http://localhost:8080/my-build-dir/bin/Debug/my-exe.html`

This way, the relative paths specified in each WASM source map will correctly resolve to the
path of each source file on the local server.

## Using a Custom HTML Shell File

> [emscripten_html_shell](cmake_data_config/properties/output.md#emscripten_html_shell)

Sometimes when using Emscripten you might want to
[custom HTML shell file](https://emscripten.org/docs/tools_reference/emcc.html#emcc-shell-file). The main
benefit to specifying a custom shell file is *full control over page customization*.

For example, the [emscripten-custom-html](/gcmake-test-project/emscripten-custom-html/) example project
configures an [xterm-style browser console](https://www.npmjs.com/package/xterm) for displaying output
printed to stdout. This allows the executable's [FTXUI](https://github.com/ArthurSonzogni/FTXUI) output
to be printed in full color (in the browser) as if it were being printed to a native terminal emulator.

See the [emscripten_html_shell](cmake_data_config/properties/output.md#emscripten_html_shell) property
for an explanation of how to actually use a shell file.
