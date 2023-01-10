# Updating Dependency Repositories

GCMake-rust doesn't currently have a built-in command to update all dependency git repositories at once.
However, you can use the [gcmake-update-deps.ps1](/gcmake-update-deps.ps1) PowerShell script (Windows)
or the [gcmake-update-deps.sh](/gcmake-update-deps.sh) script (Unix/Non-Windows) to do that
until the command is integrated into *gcmake-rust* itself.

## Windows PowerShell Example

``` powershell
# "git pull" the master branch of all dependency repositories already cloned in
# ~/.gcmake/dep-cache/*/git_repo and remove all invalid repositories
gcmake-update-deps.ps1

# Just remove all invalid repositories without updating the valid ones. This is useful
# when working offline.
gcmake-update-deps.ps1 -cleanOnly
```

## Unix (Non-Windows) Example

``` sh
# "git pull" the master branch of all dependency repositories already cloned in
# ~/.gcmake/dep-cache/*/git_repo and remove all invalid repositories
gcmake-update-deps.sh

# Just remove all invalid repositories without updating the valid ones. This is useful
# when working offline.
gcmake-update-deps.sh -c
```
