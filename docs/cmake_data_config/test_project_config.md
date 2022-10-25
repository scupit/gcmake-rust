# Test Project Configuration

> This page explains specific nuances for configuring test projects.

## Supported Property Subset

- [include_prefix](properties/properties_list.md#include_prefix)
- [description](properties/properties_list.md#description)
- [version](properties/properties_list.md#version)
- [output](properties/properties_list.md#output)
- [prebuild_config](properties/properties_list.md#prebuild_config)

## Nuances

1. The root project must specify a [test_framework](properties/properties_list.md#test_framework),
  otherwise an error will be thrown when trying to generate or configure a test project.
2. Test projects cannot have subprojects.
3. Test projects can only build executables. No sense building a test that can't be run.
4. Each test executable automatically has access to both the specified test framework
  and all code used to build the library or executables in the current project (same level only).
  See [output.md](properties/output.md) for some additional info.
5. The [output.requires_custom_main](properties/output.md#requires_custom_main) property only affects
  test project executables.

## Include Prefix Accumulation

Test project prefix accumulation works almost the same as
[subproject prefix accumulation](subproject_config.md#include-prefix-accumulation).
The difference is that we add an additional `TEST` part before the test project's include
prefix in order to differentiate between test code and regular project code.

This rule is also explained under the
[include_prefix property](properties/properties_list.md#include-prefix-accumulation).

For example, given a test project with include_prefix `MY_TEST_PREFIX` and its
parent project (assume this parent is the root project) with include_prefix `PARENT_PREFIX`,
the test project's full include prefix would be `PARENT_PREFIX/TEST/MY_TEST_PREFIX`. As a result, the
test project would use the directories:

- src/PARENT_PREFIX/TEST/MY_TEST_PREFIX/
- include/PARENT_PREFIX/TEST/MY_TEST_PREFIX/
- resources/PARENT_PREFIX/TEST/MY_TEST_PREFIX/

## Running the Tests

Use CMake to configure the project build and ensure the project's tests are enabled. Build the project,
then run `cpack` from the build directory. All tests should be run automatically.
