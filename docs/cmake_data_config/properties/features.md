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
| `default` | **Required** *boolean* which determines whether the feature is enabled by default. |
| `enables` | **Optional** list of other features the feature will transitively enable if it is enabled itself. |

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

**NOTE** that gcmake_dependency features can also be configured when the dependency is imported. That
is explained in the [gcmake_dependency property page](./gcmake_dependencies.md#general-info)

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
    entry_file: additional-main.cpp
    link:
      - (( feature:use-fmt )) fmt::fmt
  display-tables:
    output_type: Executable
    entry_file: main.cpp
    link:
      - (( feature:fancy-printing )) my-tui-lib::my-tui-lib
```
