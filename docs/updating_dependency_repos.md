# Updating Dependency Repositories

GCMake-rust doesn't currently have a built-in command to update all dependency git repositories at once.
However, you can use the [gcmake-update-deps.ps1](/gcmake-update-deps.ps1) PowerShell script to do that
until the command is integrated into *gcmake-rust* itself.

Example usage:

``` powershell
# "git pull" the master branch of all dependency repositories already cloned in
# ~/.gcmake/dep-cache/*/git_repo and remove all invalid repositories
gcmake-update-deps.ps1

# Just remove all invalid repositories.
gcmake-update-deps.ps1 -cleanOnly
```
