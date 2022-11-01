# gcmake_dependencies

> This page describes the [gcmake_dependencies](./properties_list.md#gcmake_dependencies) property
> root project configuration property used in cmake_data.yaml.

## General Info

> *Root project only*
>
> **OPTIONAL** `Map<DepNameString, GCMakeDependencyConfig>`

GCMake dependencies are very similar to
[predefined subdirectory dependencies](./properties_list.md#subdirectory-dependency-configuration-options)
in that they are downloaded and built directly as part of the project.

**GCMake dependencies are only able to use a Git configuration.** They support all *Git mode options*
provided by [predefined subdirectory dependencies](./properties_list.md#subdirectory-dependency-configuration-options).
NOTE that **GCMake dependencies are required to specify a `repo_url`** because the GCMake tool doesn't provide
a default one.

| Property | Example | Description |
| -------- | ---- | ----------- |
| `repo_url` | `git@github.com:scupit/gcmake-rust.git` | The URL/identifier Git uses to clone the repository |
| `git_tag` | `"3.1.0"` \| `v3.1.0` \| `develop` \| `origin/develop` | Tag or branch to check out after cloning. This is required if a *commit_hash* is not specified. |
| `commit_hash` | `"2f11710abc5aa478503a7ff3f9e654bd2078ebab"` | The specific commit hash to check out after cloning. This is required if *git_tag* is not specified. |
| `use_default_features` | `true` \| `false` | **Optional:** When `false`, turns off all features of the dependency which would otherwise be enabled by default. Defaults to `true` if not specified. |
| `features` | `[use-fmt, extra-functions]` | **Optional:** List of features to enable on the dependency when it is imported. For full control over which features are enabled by default, pair this with `use_default_features: false`. |

Minimal Example:

``` yaml
# ... Rest of the project config
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    link:
      # my-project must contain both first-lib and second-lib libraries
      - my-project::{ first-lib, second-lib }
  gcmake_dependencies:
    my-project:
      repo_url: git@some-site:noice/some-great-project
      git_tag: v1.0.0
```

Full Example:

``` yaml
# ... Rest of the project config
output:
  my-exe:
    output_type: Executable
    entry_file: main.cpp
    link:
      # Assumes my-project 
      - my-project::{ first-lib, second-lib }
gcmake_dependencies:
  my-project:
    repo_url: git@some-site:noice/some-great-project
    git_tag: v1.0.0
    # Disables all features which would be enabled in the dependency by default.
    use_default_features: false
    # This will only work if the dependency actually has features named
    # 'use-fmt' and 'additional-funcs'.
    features:
      - use-fmt
      - additional-funcs
```

## Project Checks and Validation

**NOTE:** Validation (name duplicate checks, linking verification, etc.) is disabled for
GCMake dependencies until they are cloned into *dep/* during a CMake configuration run.
This is because a GCMake project's *cmake_data.yaml* must be present for validation to
occur, and that's only possible if the repo is actually cloned.

As a result, `gcmake-rust` should be re-run after all repos are cloned by CMake so that
target link namespaces can be properly written.
