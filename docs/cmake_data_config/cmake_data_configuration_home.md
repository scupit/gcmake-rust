# cmake_data Configuration Home

`cmake_data.yaml` is the GCMake project configuration file. It is used to describe
configuration elements and project metadata which cannot be inferred from the
project structure itself.

Each GCMake project should have a `cmake_data.yaml` in
its root directory. This includes subprojects and test projects.

[Generate a new project](overview.md#common-uses) using the tool itself for a quick working example.
Alternatively, a complex working example exists in the
[GCMake Test Project repository](https://github.com/scupit/gcmake-test-project).

## Configuration by project type

Configuration is done slightly differenty dependening on whether the project is a root project, subproject, or
test project. There are also several nuances to keep in mind for each type.

> **TODO:** Write these documents.

- [Root project](root_project_config.md)
- [Subproject](subproject_config.md)
- [Test project](test_project_config.md)

## Available properties

See [properties_list.md](properties/properties_list.md) for a list of properties supported by `cmake_data.yaml`.
