# Managing Dependency Repositories

GCMake-rust now uses [CPM.cmake](https://github.com/cpm-cmake/CPM.cmake) for managing dependency
downloads and caching for repositories and archives.

## Downloaded Dependency Location

Dependencies are downloaded inside the [GCMake configuration directory](./the_configuration_directory.md#contents) dependency cache folder, which is most likely located at `~/.gcmake/dep-cache/`. CPM's
`CPM_SOURCE_CACHE` variable is set to that directory by default.

### Corrupted dependencies

While unlikely, it's probably possible for a
[subdirectory dependency](./predefined_dependency_doc.md#as_subdirectory) to become corrupted when
downloading. If a dependency is failing to download or just acting weird in general, try deleting it
from the dependency cache and re-running CMake configuration.

For example, if `fmt` is corrupted, delete `~/.gcmake/dep-cache/fmt`.
