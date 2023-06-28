# The Configuration Directory

The GCMake tool uses the `~/.gcmake` directory as a central location for containing
global project needs, such as a "dependency cache" and a default *.clang-tidy* file.

## Overview

The `.gcmake/` directory is located in the user's home folder. On Windows, this is the
directory contained in the `USERPROFILE` environment variable. On Unix systems, it's in the
directory specified in the `HOME` environment variable.

`.gcmake/` **is not a project-local folder**, meaning it does not contain single project-specific
information and will do nothing if added to a project tree.

## Contents

Currently, `~/.gcmake/` always contains these directories:

1. **dep-cache\/**: The gcmake "dependency cache" directory. [Subdirectory dependencies](./predefined_dependency_doc.md#as_subdirectory) are cloned here so that they can be used offline and reused by other projects.
2. **gcmake-dependency-configs\/**: The [predefined dependency configuration repository](predefined_dependency_doc.md).

### Manual Configuration

These files have special effects when placed in the root of `~/.gcmake`:

| Item | Effect |
| --- | --- |
| `.clang-format` | When a new **root project** is created, this is copied over and used as the default clang format file. |
| `.clang-tidy` | When a new **root project** is created, this is copied over and used as the default Clang Tidy file. |
| `.gitignore` | When a new **root project** is created, this is copied over and used as the default gitignore file. |
