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

For example, given a subproject with include_prefix `SUB_PREFIX` and its
parent project (assume this parent is the root project) with include_prefix `PARENT_PREFIX`,
the subproject's full include prefix would be `PARENT_PREFIX/SUB_PREFIX`. As a result, the
subproject would use the directories:

- src/PARENT_PREFIX/SUB_PREFIX/
- include/PARENT_PREFIX/SUB_PREFIX/
- template-impls/PARENT_PREFIX/SUB_PREFIX/
- resources/PARENT_PREFIX/SUB_PREFIX/
