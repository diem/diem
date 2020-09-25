# Installation Steps

If you have used one of the older installation methods for the prover tools, read the next section first, then
continue here.

If you already have a Libra development environment running, and just want to add prover tools,
run (in the Libra root directory):

```shell script
./scripts/dev_setup.sh -yp
. ~/.profile
```

This command should work on MacOS and Linux flavors like Ubuntu or CentOS. (Windows is currently not supported).

The `dev-setup.sh` command can be used to set up other parts of the Libra tool chain; use `-h` for more information.
Specifically, if you have a fresh Libra enlistment and machine, you can use the following to install basic build
tools like Rust together with prover tools:

```shell script
./scripts/dev_setup.sh -typ
. ~/.profile
```


# Older Installations

Older installation methods of the prover tools stored z3, dotnet, and boogie at `/usr/local/bin` and
`/usr/local/share/dotnet`, which required root access. The new method described above stores all needed
tools in the user's `$HOME` directory. This is the preferred way since it avoids messing with root access.

If you still have the tools at the public location, and/or have a custom installation of .Net, you may want
to consider to remove those to avoid any version mismatch, specifically if you plan on calling `boogie` and `z3`
directly from the command line.

Also **remove any definitions of environment variables `Z3_EXE` and `BOOGIE_EXE`** from your `.bashrc` or elsewhere.
The correct variable definitions have been automatically added to your `.profile` when calling `dev-setup.sh`.
