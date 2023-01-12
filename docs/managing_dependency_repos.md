# Managing Dependency Repositories

## Updating Repositories

GCMake-rust doesn't currently have a built-in command to update all dependency git repositories at once.
However, you can use the [gcmake-update-deps.ps1](/gcmake-update-deps.ps1) PowerShell script (Windows)
or the [gcmake-update-deps.sh](/gcmake-update-deps.sh) script (Unix/Non-Windows) to do that
until the command is integrated into *gcmake-rust* itself.

### Windows PowerShell Example

``` powershell
# "git pull" the master branch of all dependency repositories already cloned in
# ~/.gcmake/dep-cache/*/git_repo and remove all invalid repositories
gcmake-update-deps.ps1

# Just remove all invalid repositories without updating the valid ones. This is useful
# when working offline.
gcmake-update-deps.ps1 -cleanOnly
```

### Unix (Non-Windows) Example

``` sh
# "git pull" the master branch of all dependency repositories already cloned in
# ~/.gcmake/dep-cache/*/git_repo and remove all invalid repositories
gcmake-update-deps.sh

# Just remove all invalid repositories without updating the valid ones. This is useful
# when working offline.
gcmake-update-deps.sh -c
```

## Handling corrupted or invalid repositories

Sometimes cached dependency git repositories will become corrupted or rendered invalid in some way. This
often happens when CMake fails to initially clone a dependency repository into the cache.
Corrupted repositories cause project builds to repeatedly fail because they cannot be cloned properly.

Running `gcmake-update-deps -cleanOnly` or `gcmake-update-deps -c`
([see above](#managing-dependency-repositories)) should fix the issue by
removing invalid repositories from the dependency cache. If your build still fails after running that
command, try deleting your project's the *dep/* directory as well as the build directory you are
currently using. Removing those forces CMake to re-populate the missing dependency repositories with
valid ones.
