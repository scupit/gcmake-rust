# cmake_data.yaml Properties List

> This page describes the full list of toplevel properties available to GCMake projects.

**NOTE** that different project types support different subsets of these properties. See the
respective pages on [root project configuration](../root_project_config.md),
[subproject configuration](../subproject_config.md), and
[test project configuration](../test_project_config.md) for a list of which properties are
supported for a specific project type.

## Property List

### Properties supported by all project types

| Property | Description |
| -------- | ----------- |
| [include_prefix](#include_prefix) | Sets the project's base include prefix. This affects the project directory names. |
| [description](#description) | A short description of the project |
| [version](#version) | Three-part project version, optionally prefixed with a *v* |
| [output](#output) | Defines what the project actually builds |
| [prebuild_config](#prebuild_config) | Configuration for executable (non-Python) pre-build scripts. |

### Root project only properties

| Property | Description |
| -------- | ----------- |
| [name](#name) | The project's name identifier, no whitespace. |
| [vendor](#vendor) | The project's vendor. Usually your name or organization. |
| [supported_compilers](#supported_compilers) | A list of compilers which are known to successfully compile the project |
| [languages](#languages) | Configuration metadata (such as language standard) for the C and C++ languages |
| [default_build_type](#default_build_type) | Selects the project's default build configuration |
| [documentation](#documentation) | Options for docmenting the project, such as which documentation generator the project will use. |
| [features](#features) | Configure a set of high-level project "features" which can be used as constraints throughout the yaml configuration. Inspired by [Rust's Cargo "features"](https://doc.rust-lang.org/cargo/reference/features.html). |
| [predefined_dependencies](#predefined_dependencies) | Imports a non-GCMake dependency into the project. Only [pre-configured dependencies](../../predefined_dependency_doc.md) with a directory in the [predefined dependency configuration repository](/gcmake-dependency-configs/) are supported. |
| [gcmake_dependencies](#gcmake_dependencies) | Imports other GCMake projects as dependencies into the build. |
| [test_framework](#test_framework) | Sets the test framework to be used for all the project's tests |
| [global_defines](#global_defines) | A set of compiler defines which are always added to the project's build. |
| [global_properties](#global_properties) | Miscellaneous configurable project properties which don't really have their own category |
| [installer_config](#installer_config) | Additional configuration for installer and shortcut generation |
| [build_configs](#build_configs) | The set of build configurations for the project. This includes compiler flags, linker flags, and defines. |

## Information by Property

### name

> *Root project only*
>
> **REQUIRED** `String`

Name of the project. *Cannot contain spaces*.

``` yaml
name: the-project-name
```

### include_prefix

> All project types
>
> **REQUIRED** `String`

The project's base include prefix. *Cannot contain spaces*.

The include prefix directly affects the file inclusion path for a project. This is necessary for
"namespacing" a project's files directly, so that it is always clear which project a file is
being included from. That being said, it's a good idea to **make the include prefix similar to**
**the project name, so that developers can easily associate the include path with your project.**

#### Include Prefix Accumulation

[Subprojects](../subproject_config.md#include-prefix-accumulation) and
[test projects](../test_project_config.md#include-prefix-accumulation) both have special
rules for how their specified base include prefix (`include_prefix`) is integrated into the
project, and how it affects directory structure.

For example, a root project which specifies

``` yaml
include_prefix: ROOT_PREFIX
```

will use these directories:

- include/ROOT_PREFIX
- src/ROOT_PREFIX

A *subproject of that root project* which specifies

``` yaml
include_prefix: SUBPROJECT_PREFIX
```

will use these directories:

- include/ROOT_PREFIX/SUBPROJECT_PREFIX
- src/ROOT_PREFIX/SUBPROJECT_PREFIX

And a *test project inside that subproject* which specifies:

``` yaml
include_prefix: THE_TEST_PREFIX
```

will use these directories:

- include/ROOT_PREFIX/SUBPROJECT_PREFIX/TEST/THE_TEST_PREFIX
- src/ROOT_PREFIX/SUBPROJECT_PREFIX/TEST/THE_TEST_PREFIX

### version

> *Root project only*
>
> **REQUIRED** `Version String`

The project version, provided as a three part string. The version can optionally be prefixed with a `v`.

``` yaml
version: "1.6.3"
```

or

``` yaml
version: v1.6.3
```

### description

> *Root project only*
>
> **REQUIRED** `String`

A concise description of the project. This is also used as the description for some generated installers.

``` yaml
description: Wow, project so nice
```

### output

> All project types
>
> **REQUIRED** `Map<String, OutputConfigurationObject>`

This property describes the execuables or libraries built by the project.

For a full explanation, see [output.md](output.md).

#### Concise executable example

``` yaml
output:
  my-exe:
    type: Executable
    entry_file: main.cpp
  another-program:
    type: Executable
    entry_file: a-second-main.cpp
```

#### Concise compiled library example

``` yaml
output:
  my-compiled-lib:
    type: CompiledLib
    # type: StaticLib
    # type: SharedLib
    entry_file: my-compiled-lib.hpp
```

#### Concise header-only library example

``` yaml
output:
  its-header-only:
    type: HeaderOnlyLib
    entry_file: its-header-only.hpp
```

### vendor

> *Root project only*
>
> **REQUIRED** `String`

The project vendor. This would usually be either your name or organization.

``` yaml
vendor: scupit
```

### supported_compilers

> *Root project only*
>
> **REQUIRED** `Set<CompilerSpecifierString>`

The set of all compilers which the project supports being compiled by. Build configuration flags and
defines can only be set for compilers defined in this list.

Allowed values:

- GCC
- Clang
- MSVC
- Emscripten

``` yaml
supported_compilers:
  - GCC
  - Clang
  - MSVC
  - Emscripten
```

### languages

> *Root project only*
>
> **REQUIRED** `LanguageConfigMap`

Describes the language features required by the project. At present, `standard` is the only property
that can be configured for each language.

**NOTE:** Currently both `c` and `cpp` must be configured even if the project doesn't use both C and C++.

| Property  |  c  |  cpp |
| --------- | --- | ---- |
| `standard` | `90` \| `99` \| `11` \| `17` \| `23` | `98` \| `11` \| `14` \| `17` \| `20` \| `23` |

``` yaml
languages:
  c:
    # Using exact_standard is not recommended.
    # exact_standard: 23
    min_standard: 99
  cpp:
    # Using exact_standard is not recommended.
    # exact_standard: 20
    min_standard: 17
```

### documentation

> *Root project only*
>
> **OPTIONAL** `DocumentationConfigObject`

Configures documentation-related options for the project. This is mainly useful for setting
the project's documentation generator. For an explanation on actually documenting the project
plus examples for configuring each supported generator, see
[documenting_your_project.md](../../documenting_your_project.md).

| Field | Type | Default | Description |
| ----- | ---- | ------- | ----------- |
| `generator` | Generator name *String* (`Doxygen` or `Sphinx`) | *No default* | Specifies which documentation generator the project will use. |
| `headers_only` | *boolean* | `true` | Whether to only document header files. When `false`, implementation files ( *.c*, *.cpp*, etc.) will also be processed by the documentation generator. |
| `include_private_headers` | *boolean* | `false` | Whether to document private headers. When `true`, private header and template-implementation files will also be processed by the documentation generator. |

#### Doxygen Example

``` yaml
documentation:
  generator: Doxygen
```

#### Sphinx Example

``` yaml
documentation:
  generator: Sphinx
```

#### Example which also includes cpp files

``` yaml
documentation:
  generator: Doxygen
  # Also include .c and .cpp files in documentation.
  # This is not recommended.
  headers_only: false
```

### default_build_type

> *Root project only*
>
> **REQUIRED** `BuildTypeString`

Selects the default configuration used to build the project from any configuration already defined in
the project's [build_configs](#build_configs) section. That means this value can be one of `Debug`, `Release`,
`MinSizeRel`, or `RelWithDebInfo`. However, specifying a build configuration which wasn't used in the
build_config section will result in an error.

``` yaml
default_build_type: Release
# This will result in an error, since a MinSizeRel configuration was not defined.
# default_build_type: MinSizeRel
supported_compilers:
  - GCC
build_configs:
  Debug:
    GCC:
      compiler_flags: [ -Og ]
  Release:
    GCC:
      compiler_flags: [ -O3 ]
```

### features

> *Root project only*
>
> **OPTIONAL** `Map<FeatureName, FeatureConfigObject>`

Features are explained in their own document [features.md](./features.md).

### predefined_dependencies

> *Root project only*
>
> **OPTIONAL** `Map<PredefinedDependencyName, PredepConfigObject>`

This is the section where non-GCMake dependencies are imported into the build.
To consume other GCMake projects, to add GCMake projects to the build, use the
[gcmake_dependencies](#gcmake_dependencies) property instead.

In the map, each dependency name must have a matching configuration directory in the
[predefined dependency configuration repository](/gcmake-dependency-configs/).

| Dependency type | Example | Description |
| --------------- | ------- | ----------- |
| CMake Subdirectory | [SFML](/gcmake-dependency-configs/SFML/) | The dependency will be copied into `YOUR_PROJECT_ROOT/dep/DEP_NAME` and built as a subdirectory of your project. The download step will be done using either Git or HTTPS, depending on the [dependency configuration options](#subdirectory-dependency-configuration-options) specified. |
| CMake Installed Module | [SDL2](/gcmake-dependency-configs/SDL2/) | A CMake project which must be manually built and installed on the system manually before use. Internally, the project is located using CMake's [find_package](https://cmake.org/cmake/help/latest/command/find_package.html#id4) in *Config* mode. |
| CMake Find Module | [OpenGL](/gcmake-dependency-configs/) | A library which is already installed on the system. These are usually system libraries. Currently, only a **subset of** [CMake Find Modules](https://cmake.org/cmake/help/latest/manual/cmake-modules.7.html#find-modules) are supported. In the future I'd like to be able to configure custom find modules as well, but that currently isn't supported. |
| CMake Components Module | [wxWidgets](/gcmake-dependency-configs/wxWidgets/) | This is the same as the CMake Find Module dependency type, except that the internal CMake `find_package` call specifies `COMPONENTS`. |

Usage example

``` yaml
predefined_dependencies:
  SFML:
    # Since SFML is a subdirectory dependency, either a git_tag or commit_hash must be specified
    # in order to keep versioning consistent. To always use the latest version, you could just
    # specify the "master" branch for the git tag.
    # git_tag: master
    git_tag: "2.5.1"
    # commit_hash: "2f11710abc5aa478503a7ff3f9e654bd2078ebab"
  nlohmann_json:
    # Downloads the source code as an archive (zip, tar.gz) using URL mode insted of Git.
    # This is useful for large repositories such as nlohmann_json where the source code archive
    # is much smaller than the repository with full git history.
    file_version: v3.11.2
  imgui:
    git_tag: v1.88
    # file_version: v1.88.0
  SDL2: { }
  OpenGL: { }
  wxWidgets: { }
```

#### Subdirectory dependency configuration options

Subdirectory dependencies can either be cloned as a Git repository, or downloaded as an archive file
(zip, tar.gz, etc.) and unpacked. Specifying configuration options for a download mode will result in that
download mode being used. **NOTE** that dependencies are not required to support any download methods.
For a list of download methods available to a dependency, run `gcmake-rust predep-info -d the-dep-name`.
For example, `gcmake-rust predep-info -d SFML nlohmann_json`

> If any Git mode options are specified, then Git will be used to download the repository.

| Git mode option | Example value | Description |
| --------------- | ------------- | ----------- |
| `git_tag` | `"2.5.1"` or `master` | The tag or branch to be checked out after the repo is cloned. This is essentially how the project "version" is specified. If this property is not specified, then `commit_hash` must be specified. |
| `commit_hash` | `"2f11710abc5aa478503a7ff3f9e654bd2078ebab"` | The commit to be checked out after the repo is cloned. If this property is not specified, then `git_tag` must be specified. |
| `repo_url` | `git@github.com:scupit/gcmake-rust.git` | **Optional** alternate repository URL. Each predefined subdirectory dependency already has a default git URL. This property just overrides it. |

> If any URL mode options are specified, then the repository will be downloaded as an archive and unpacked.

| URL mode option | Example value | Description |
| --------------- | ------------- | ----------- |
| `file_version` | `"2.5.1"` or `v2.5.1` | Specifies the version of the release archive file to be downloaded. **This must always be given as a three-part version, even if the repository itself doesn't use a three-part version.** The given version will be transformed behind the scenes into a valid URL pointing to an existing archive file. For example, use `1.88.0` or `v1.88.0` for the ImGUI *v1.88* release. |

Any dependency added to the project can then be linked to any output or executable pre-build script
throughout the entire project tree. See the [linking docs](../linking.md) for specifics.

Example:

``` yaml
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    link:
      - SFML::{ system, window, graphics }
predefined_dependencies:
  SFML:
    # Either git_tag or commit_hash is required to be speciifed.
    git_tag: "2.5.1"
    # commit_hash: "2f11710abc5aa478503a7ff3f9e654bd2078ebab"
```

### gcmake_dependencies

> *Root project only*
>
> **OPTIONAL** `Map<String, GCMakeDepConfigObject>`

This is the section where external GCMake projects can be imported and consumed as dependencies.

The gcmake_dependencies property is described in its own document
[gcmake_dependencies.md](./gcmake_dependencies.md).

### prebuild_config

> All project types
>
> **OPTIONAL** `PreBuildConfigObject`

This property has its own page at [prebuild_config.md](prebuild_config.md)

### test_framework

> *Root project only*
>
> **OPTIONAL** `PredefinedDependencyEntry`

Sets the test framework used for all unit tests (test projects) in the entire project tree.

The test framework is specified in the exact same way as a predefined dependency, but has
some additional properties:

1. Only one test framework can be specified per root project.
2. The test framework is automatically linked to every test executable in the project.
3. Only [Catch2](https://github.com/catchorg/Catch2),[GoogleTest](https://github.com/google/googletest),
  and [doctest](https://github.com/doctest/doctest) are supported as test frameworks.
4. When using GCMake to generate a test project, a framework-specific main file will be generated.

For working examples of using a test framework in a project, see the
[Catch2 usage example project](/gcmake-test-project/using-catch2/),
[GoogleTest usage example project](/gcmake-test-project/using-googletest/),
and the [doctest usage example project](/gcmake-test-project/using-doctest/)

Catch2 example:

``` yaml
test_framework:
  Catch2:
    git_tag: v3.1.0
```

GoogleTest example:

``` yaml
test_framework:
  GoogleTest:
    git_tag: v1.12.2
```

doctest example:

``` yaml
test_framework:
  doctest:
    git_tag: v2.4.9
```

### global_defines

> *Root project only*
>
> **OPTIONAL** `List<String>`

Adds a list of compiler defines which will be added to every compiled item in the project tree
(including pre-build scripts, test executables, libraries, etc.) for all build configurations
and compilers. **NOTE** that these are *not* added to dependency projects.

For a full explanation on how compiler defines should be written, see the
[compiler definition format](../data_formats.md#compiler-defines) page. Essentially, write compiler
defines the same way you would on the command line, but without the leading `-D`.

``` yaml
global_defines:
  - SOME_BOOLEAN_DEFINE
  - SOME_NUMBER_DEFINE=2
  - SOME_STRING_DEFINE="This is awesome"
```

### global_properties

> *Root project only*
>
> **OPTIONAL** `Map<PropertyName, PropertyValue>`

Specify miscellaneous project-wide configuration options.

| Property | Type | Default | Description |
| -------- | ---- | ------- | ----------- |
| `ipo_enabled_by_default_for` | List of build configuration names | `[]` | The list of build configurations which should have interprocedural optimization turned on by default. When a configuration is listed, the corresponding[CMAKE_INTERPROCEDURAL_OPTIMIZATION_<CONFIG_NAME>](https://cmake.org/cmake/help/latest/variable/CMAKE_INTERPROCEDURAL_OPTIMIZATION.html) variable will default to *ON*. |
| `are_language_extensions_enabled` | boolean | `false` | Allows or disallows the use of C and C++ compiler language extensions. When `true`, [CXX_EXTENSIONS](https://cmake.org/cmake/help/latest/prop_tgt/CXX_EXTENSIONS.html) and [C_EXTENSIONS](https://cmake.org/cmake/help/latest/prop_tgt/C_EXTENSIONS.html) will be set to *ON* for each item built by your project. |
| `default_compiled_lib_type` | `Static` \| `Shared` | `Shared` | The default type for any `CompiledLib` output created in the whole project tree, including dependencies. This value internally dictates the default value of *BUILD_STATIC_LIBS* and [BUILD_SHARED_LIBS](https://cmake.org/cmake/help/latest/variable/BUILD_SHARED_LIBS.html) variables. |

``` yaml
# Example which overrides all the default values.
global_properties:
  ipo_enabled_by_default_for:
    - Release
    - RelWithDebInfo
  are_language_extensions_enabled: true
  default_compiled_lib_type: Static
```

### installer_config

> *Root project only*
>
> **OPTIONAL** `InstallerConfigObject`

Provides additional specific installer configuration for the project. Installer configurations (including
the default configuration) are applied to all installers wherever possible.

**NOTE** that installer configurations are generated regardless of whether this property is defined or
not. This is just a way to override some default values, and to do some additional useful configuration
such as creating shortcuts.

| Property | Type | Default | Description |
| -------- | ---- | ------- | ----------- |
| `title` | String | [project name](#name) | Overrides the displayed project title when running graphical installers. This is good for setting a nicer, readable project name such as *"My Great Project"* in place of *"my-great-project"*, since GCMake project names can't contain spaces. |
| `description` | String | [project description](#description) | Overrides the description used in installers. |
| `name_prefix` | String | [project name](#name) | Overrides the default installer package prefix. This also affects the default installation directory name. |
| `shortcuts` | `Map<OutputName,` [ShortcutConfigObject](#shortcut-configuration-object)`>` | Empty | Describes a set of shortcuts to create when running Windows installers. Each key in the map must exactly match the name of an executable output created in the project or its subprojects. |

#### Shortcut configuration object

| Property | Type | Description |
| -------- | ---- | ----------- |
| `name` | String | **REQUIRED** Name of the shortcut to create. Ideally this should be similar to the name of the executable the shortcut points to. |

For example, a project with the configuration:

``` yaml
name: my-great-project
descripttion: My awesome description
vendor: scupit
version: 0.0.1
default_build_type: Release

languages:
  c:
    min_standard: 11
  cpp:
    min_standard: 17

supported_compilers:
  - Clang
  - MSVC
  - GCC

output:
  my-executable:
    output_type: Executable
    entry_file: main.cpp

build_config:
  Debug: {}
  Release: {}
```

might create a Windows NSIS installer called `my-great-project-0.0.1-win64.exe` with the title
`my-great-project` which creates no shortcuts and tries to install to
`C:\Program Files\my-great-project 0.0.1` by default.

However, if we add the installer configuration:

``` yaml
# ... rest of the project configuration above
installer_config:
  title: My Great Project
  name_prefix: noice-project
  description: My alternate description
  shortcuts:
    # NOTE that this key is the exact same name as the project's output executable
    # 'my-executable'. 
    my-executable:
      name: My Shortcut
```

then the project would create a Windows NSIS installer called `noice-project-0.0.1-win64.exe` with the title
`My Great Project` which creates the desktop shortcut `My Shortcut` (points to *my-executable*) and tries
to install to `C:\Program Files\noice-project 0.0.1` by default.

### build_configs

> *Root project only*
>
> **REQUIRED** `Map<BuildConfigName, BuildConfigurationObject>`

This is the section for defining build configurations. Things like compiler flags and Debug/Release builds
are defined in this section.

See this property's page at [build_configs.md](build_configs.md) for details.
