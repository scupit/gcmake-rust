# features

> This page describes the [features](./properties_list.md#features) root project configuration property
> used in cmake_data.yaml. These are **project-level features*, not language features. For language
> features, see [output.language_features](./output.md#language_features)

## General Info

> *Root project only*
>
> **OPTIONAL** `Map<FeatureNameString, FeatureConfigObject>`

Inspired by [Rust's Cargo "features"](https://doc.rust-lang.org/cargo/reference/features.html),
"Features" are best thought of as your project's set of configurable optional functionality. When combined
with [constraint expressions](../data_formats.md#constraint-specifier), they can be used to configure things
such as *optional library support*, conditional linking, optionally built targets, and conditional compiler
flags and defines.

Once a feature is defined, it can be used in any
[constraint expression](../data_formats.md#constraint-specifier) in the cmake_data.yaml.

| Property | Description |
| -------- | ----------- |
| `default` | **Required** *boolean* or [constraint expression](../data_formats.md#constraint-specifier) *string* which determines whether the feature is enabled by default. A constraint expression like `(( not windows ))` means the feature is enabled by default only on systems where the constraint holds. |
| `enables` | **Optional** list of other features the feature will transitively enable if it is enabled itself. Each entry may begin with a [constraint expression](../data_formats.md#constraint-specifier). Transitive enablement only occurs when the constraint expression holds. |

## Enabler Expressions

A feature's `enable` list can enable other features in the same project:

``` yaml
features:
  second:
    # Even though this is set to false, it will be enabled by 'first' since
    # 'first' is enabled by default.
    default: false
  first:
    default: true
    enables:
      - second
```

and/or features in a [gcmake_dependency project](./properties_list.md#gcmake_dependencies):

``` yaml
features:
  second:
    # Even though this is set to false, it will be enabled by 'first' since
    # 'first' is enabled by default.
    default: false
  first:
    default: true
    enables:
      - second
      # This will only work if "some-dep" is an existing gcmake_dependency that
      # has a feature called "the-dep-feature"
      - some-dep/the-dep-feature

gcmake_dependencies:
  some-dep:
    repo_url: git@some-site.com:my/repo.git
    git_tag: v1.0.0
```

> **NOTE** that gcmake_dependency features can also be configured when the dependency is imported. That
> is explained in the [gcmake_dependency property page](./gcmake_dependencies.md#general-info)

and/or features declared by a [predefined dependency](./properties_list.md#predefined_dependencies):

``` yaml
features:
  fancy-fonts:
    default: false
    # Enables imgui's 'freetype' feature whenever this project's
    # 'fancy-fonts' feature is enabled.
    enables:
      - imgui/freetype

predefined_dependencies:
  # Dependency import is always explicit: enabling a dependency's feature never
  # imports the dependency (or the dependencies its feature requires) automatically.
  freetype:
    git_tag: VER-2-13-3
  imgui:
    git_tag: v1.92.9b
```

A predefined dependency's features can also be enabled directly on its import entry
(`features: [ freetype ]`) - see
[the predefined_dependencies property](./properties_list.md#predefined_dependencies). Names are
resolved against gcmake_dependencies first, then predefined dependencies. Trying to enable a feature
the dependency doesn't declare will result in a project load error.

## Constraining when a feature is enabled

`enables` entries and the `default` specifier can carry a [constraint expression](../data_formats.md#constraint-specifier).

``` yaml
features:
  with-freetype:
    # Only enabled by default when C99 is available.
    default: (( c:99 ))
    enables:
      # Whenever this project's `with-freetype` feature is enabled, imgui's `freetype` feature
      # is transitively enabled, but only when building for a target other than Windows.
      - (( not windows )) imgui/freetype

predefined_dependencies:
  freetype:
    git_tag: VER-2-13-3
  imgui:
    git_tag: v1.92.9b
```

These constraints may not use feature predicates (like `(( feature:some-feature ))` ). However, the rest of the
predicate types (system predicates, compiler predicates) are fully supported.

**Listing the same enabler twice with different constraints is allowed; the feature is enabled wherever either constraint holds.**

The `features` lists on
[predefined dependency](./properties_list.md#predefined_dependencies) and
[gcmake_dependency](./gcmake_dependencies.md#general-info) import entries follow the same rules.

## Example

Full Example:

``` yaml
features:
  all-exes:
    default: true
  use-fmt:
    default: false
  fancy-printing:
    default: false
    enables:
      - my-tui-lib/color
      - use-fmt

predefined_dependencies:
  fmt:
    git_tag: "9.1.0"

gcmake_dependencies:
  my-tui-lib:
    repo_url: git@some-site.com:my/repo.git
    git_tag: v1.2.0
    # See the gcmake_dependencies page for an explanation of this.
    use_default_features: false
    features:
      - some-feature

global_defines:
  - (( feature:fancy-printing )) IS_FANCY_PRINTING_ENABLED=1

output:
  (( feature:all-exes )) additional-exe:
    output_type: Executable
    # File must be located at: src/FULL_INCLUDE_PREFIX/additional-main.cpp
    entry_file: additional-main.cpp
    link:
      - (( feature:use-fmt )) fmt::fmt
  display-tables:
    output_type: Executable
    # File must be located at: src/FULL_INCLUDE_PREFIX/main.cpp
    entry_file: main.cpp
    link:
      - (( feature:fancy-printing )) my-tui-lib::my-tui-lib
```
