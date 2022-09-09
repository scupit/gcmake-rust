# cmake_data.yaml Properties List 

## All Toplevel Properties


### Root Project Only

- [name](#name)
- vendor
- default_build_type
- installer_config
- languages
- supported_compilers
- test_framework
- global_defines
- predefined_dependencies
- gcmake_dependencies
- build_configs
- prebuild_config

### All Project Types

- [include_prefix](#includeprefix)
- description
- version
- output

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

For example, a root project which specifies

``` yaml
include_prefix: ROOT_PREFIX
```

will use these directories:

- include/ROOT_PREFIX
- src/ROOT_PREFIX
- template-impls/ROOT_PREFIX

A *subproject of that root project* which specifies

``` yaml
include_prefix: SUBPROJECT_PREFIX
```

will use these directories:

- include/ROOT_PREFIX/SUBPROJECT_PREFIX
- src/ROOT_PREFIX/SUBPROJECT_PREFIX
- template-impls/ROOT_PREFIX/SUBPROJECT_PREFIX

And a *test project inside that subproject* which specifies:

``` yaml
include_prefix: THE_TEST_PREFIX
```

will use these directories:

- include/ROOT_PREFIX/SUBPROJECT_PREFIX/TEST/THE_TEST_PREFIX
- src/ROOT_PREFIX/SUBPROJECT_PREFIX/TEST/THE_TEST_PREFIX
- template-impls/ROOT_PREFIX/SUBPROJECT_PREFIX/TEST/THE_TEST_PREFIX
