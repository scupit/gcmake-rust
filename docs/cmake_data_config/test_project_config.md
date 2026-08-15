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
4. Each test executable automatically has access to the specified test framework, and inherits code,
  links, and defines from **exactly one** output of the project being tested.
  See [Automatic Inheritance](#automatic-inheritance) below.
5. The [output.requires_custom_main](properties/output.md#requires_custom_main) and
  [output.inherits_from_exe](properties/output.md#inherits_from_exe) properties only affect
  test project executables.

## Automatic Inheritance

The purpose of tests is to verify assumptions developers make about a project's code. Each test must
therefore have access to the source code, libraries, and configuration used to build a project's library
or executables. GCMake handles this inheritance automatically;
**Each test executable always inherits source code, linked libraries, and compiler defines from exactly one output of the project being
tested (its direct parent project).** This is done automatically when possible, but sometimes requires minimal configuration from you:

- If the parent project defines a **single output** (its library, or its only executable), that
  output is inherited automatically. Nothing needs to be configured.
- If the parent project defines **multiple executables**, each test executable must specify which
  executable it inherits from using [inherits_from_exe](properties/output.md#inherits_from_exe):

``` yaml
# tests/my-test/cmake_data.yaml, where the parent project defines
# executables 'first-exe' and 'second-exe'.
output:
  my-test:
    output_type: Executable
    entry_file: main.cpp
    inherits_from_exe: first-exe
```

Regular [link](properties/output.md#link) entries may still be used to add extra dependencies to a
test executable, but they never replace the inheritance described above.

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

**Resource Directory Enforcement**: Assets must be placed in the properly prefixed `resources/PARENT_PREFIX/TEST/MY_TEST_PREFIX/` directory in order to be copied to the build. GCMake will warn if assets are found outside this directory, but won't copy them to the build.

## Running the Tests

Use CMake to configure the project build and ensure the project's tests are enabled. Build the project,
then run `cpack` from the build directory. All tests should be run automatically.
