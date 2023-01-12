# Common Issues and Pitfalls

> As I discover more compilation pitfalls and issues with the GCMake tool, I will list them
> here.

## Interesting Errors

### The procedure entry point _ZSt28__throw_bas_array_new_lengthv could not be located in the dynamic link library

On Windows, this error is caused by an executable trying to load the wrong *libstdc++-6.dll*.

This error usually happens when more than one *libstdc++-6.dll" is exposed by PATH.
Possible duplicates might be found in a Neovim bin directory or a Strawberry Perl installation bin
directory, among other places.
**If your pre-build scripts are failing for seemingly no reason, this could be the issue.**

**Solutions:**

1. Make sure only the MinGW *libstdc++-6.dll* is exposed by PATH. 
  To find all *libstdc++-6.dll" files on your system, check out the
  fantastic [voidtools "Everything"](https://www.voidtools.com/) search tool.

2. Put a copy of MinGW's *libstdc++-6.dll* in the binary directory so the executable doesn't need to
  look in PATH for an alternative one.

## Emscripten

Instead of duplicating them here, see the [Emscripten doc page](./emscripten.md) for 
Emscripten usage information and caveats.

## Windows Symlinks

By default, Windows requires administrator permissions to create symlinks. This is annoying.
Turning Windows Developer mode on will allow you (and CMake) to create symlinks without
requiring developer permissions.

## Global Strawberry Perl Install while MinGW is in system PATH

If your MinGW bin directory is path of your system PATH on Windows, globally installing
[Strawberry Perl](https://strawberryperl.com/) will cause compilations to fail with
*undefined reference to __imp\** errors.
[This is a known issue](https://github.com/StrawberryPerl/Perl-Dist-Strawberry/issues/11).

My workaround for this issue was to uninstall Strawberry Perl, then use a
[portable install](https://strawberryperl.com/releases.html) instead. Instead of adding
it to PATH, I just made a `perl.ps1` script act as an alias to Perl's `portableshell.bat`.

## Configuration fails because a dependency repository can't be cloned

Sometimes dependency repositories become corrupted or are rendered invalid somehow.
To fix, read the documentation section on
[managing dependency repositories](managing_dependency_repos.md) then try following the suggestions
for [handling corrupted or invalid dependency repositories](managing_dependency_repos.md#handling-corrupted-or-invalid-repositories).
