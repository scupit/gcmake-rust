# Common Issues and Pitfalls

> As I discover more compilation pitfalls and issues with the GCMake tool, I will list them
> here.

## Global Strawberry Perl Install while MinGW is in system PATH

If your MinGW bin directory is path of your system PATH on Windows, globally installing
[Strawberry Perl](https://strawberryperl.com/) will cause compilations to fail with
*undefined reference to __imp\** errors.
[This is a known issue](https://github.com/StrawberryPerl/Perl-Dist-Strawberry/issues/11).

My workaround for this issue was to uninstall Strawberry Perl, then use a
[portable install](https://strawberryperl.com/releases.html) instead. Instead of adding
it to PATH, I just made a `perl.ps1` script act as an alias to Perl's `portableshell.bat`.
