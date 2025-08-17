# Subproject Configuration

> This page explains specific nuances for configuring subprojects.

## Supported Property Subset

- [include_prefix](properties/properties_list.md#include_prefix)
- [description](properties/properties_list.md#description)
- [version](properties/properties_list.md#version)
- [output](properties/properties_list.md#output)
- [prebuild_config](properties/properties_list.md#prebuild_config)

## Include Prefix Accumulation

In order to ensure `include_prefix` structure matches the project structure,
a subproject's full include prefix is equal to the *full*  include prefix
of its parent project plus the given include_prefix.
**This is recursively true for nested subprojects.**

This rule is also explained under the
[include_prefix property](properties/properties_list.md#include-prefix-accumulation).

For example, given a subproject with include_prefix `SUB_PREFIX` and its
parent project (assume this parent is the root project) with include_prefix `PARENT_PREFIX`,
the subproject's full include prefix would be `PARENT_PREFIX/SUB_PREFIX`. As a result, the
subproject would use the directories:

- src/PARENT_PREFIX/SUB_PREFIX/
- include/PARENT_PREFIX/SUB_PREFIX/
- resources/PARENT_PREFIX/SUB_PREFIX/

**Resource Directory Enforcement**: Assets must be placed in the properly prefixed `resources/PARENT_PREFIX/SUB_PREFIX/` directory in order to be copied to the build. GCMake will warn if assets are found outside this directory, but won't copy them to the build.

**Entry File Placement**: Entry files for subprojects must also follow the accumulated include prefix structure:

- Executable entry files must be placed in the immediate root of `src/PARENT_PREFIX/SUB_PREFIX/`
- Library entry files must be placed in the immediate root of `include/PARENT_PREFIX/SUB_PREFIX/`

**Important:** In the `cmake_data.yaml` configuration, entry files are specified as **filenames only** (e.g., `entry_file: main.cpp`), not full paths. The system automatically resolves these filenames to the correct directories based on the output type and the subproject's accumulated include prefix.
