# Documenting Your Project

> This page explains how to document your project using [Doxygen](https://www.doxygen.nl/)
> or [Sphinx](https://www.sphinx-doc.org/en/master/) documentation generators.

If you're looking for a good guide on how to do this in raw CMake, check out
[this fantastic Microsoft devblog post](https://devblogs.microsoft.com/cppblog/clear-functional-c-documentation-with-sphinx-breathe-doxygen-cmake/).

## Configuration

[Doxygen](https://www.doxygen.nl/) and [Sphinx](https://www.sphinx-doc.org/en/master/) are
currently the only two supported documentation generators.

For a full explanation of documentation configuration options, see the
['documentation' section in properties_list.md](./cmake_data_config/properties/properties_list.md#documentation).
Otherwise, the sections below explain how to use any supported documentation generator with your project.

### Using Doxygen

Make sure Doxygen is downloaded and installed on your system. The latest releases can be found
on [Doxygen's downloads page](https://www.doxygen.nl/download.html).

Once Doxygen is installed, configure your root project to use Doxygen:

``` yaml
# ... rest of root project configuration
documentation:
  generator: Doxygen
```

then generate the default *docs/Doxyfile.in* with `gcmake-rust gen-default doxyfile`.
If you already have a *docs/Doxyfile.in*, just reconfigure the project with `gcmake-rust`
so it knows to use Doxygen.

Some projects may want to use some *dummy.hpp* file to generate a custom home page with Doxygen.
For example, [SFML does just that](https://github.com/SFML/SFML/blob/master/doc/mainpage.hpp)
with a mainpage.hpp file. See [gcmake-test-project/basic-tests](/gcmake-test-project/basic-tests/)
for a working example of how to do this with GCMake. (NOTE that the basic-tests project uses Sphinx,
but still generates Doxygen documentation. After generating documentation, you can open the
Doxygen-generated index.html to see the homepage).

### Using Sphinx

> **NOTE:** Sphinx requires Doxygen to work with C/C++. Make sure you've already installed
> one of the [Doxygen releases](https://www.doxygen.nl/download.html).

Our default [Sphinx](https://www.sphinx-doc.org/en/master/) configuration makes use of
[Breathe](https://breathe.readthedocs.io/en/latest/) (required),
[Exhale](https://exhale.readthedocs.io/en/latest/),
and the [Sphinx 'Read the Docs' Theme](https://sphinx-rtd-theme.readthedocs.io/en/stable/). Sphinx
and Breathe are both required, but the others are optional. All can be installed using Python's pip:

``` sh
python -m pip install sphinx breathe exhale sphinx_rtd_theme
```

Once those are installed, configure your root project to use Sphinx for its documentation generator:

``` yaml
# ... rest of root project configuration
documentation:
  generator: Sphinx
```

then generate the default sphinx configuration files with `gcmake-rust gen-default sphinx-config`.
If you already have *docs/index.rst* and *docs/conf.py.in*, just reconfigure the project with
`gcmake-rust` so it knows to use the Sphinx configuration.

## Caveats

1. Project documentation is not built by default. It must be explicitly enabled using
`-D<your-project-name>_BUILD_DOCS=ON` when configuring a CMake build. This is to ensure your project build
will work even if the person building it doesn't have Doxygen or another documentation generator installed.

Example:

``` sh
cmake -B build -Dgcmake-basic-tests_BUILD_DOCS=ON
```
