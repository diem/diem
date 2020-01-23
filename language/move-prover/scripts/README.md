There are three experimental scripts in this folder.
1. `install-dotnet.sh` installs Dotnet Core.
2. `install-boogie.sh` installs the latest version of Boogie from nuget.org.
3. `install-z3.sh` installs a specific version of Z3.

`install-dotnet.sh` must be executed before `install-boogie.sh`.  There is no other dependency.

The commands `boogie` and `z3` should be in your path once these scripts have finished successfully.
