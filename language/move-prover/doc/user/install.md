# Installation Steps

If you have used one of the older installation methods for the prover tools, read the next section first, then
continue here.

If you already have a Diem development environment running, and just want to add prover tools,
run (in the Diem root directory):

```shell script
./scripts/dev_setup.sh -yp
. ~/.profile
```

This command should work on MacOS and Linux flavors like Ubuntu or CentOS. (Windows is currently not supported).

Notice that you have to include environment variable definitions in `~/.profile` into your shell. Depending on your
setup, the  `~/.profile` may be already automatically loaded for each login shell, or it may not. If not, you may
need to add `. ~/.profile` to your `~/.bash_profile` or other shell configuration manually.

The `dev-setup.sh` command can be used to set up other parts of the Diem tool chain; use `-h` for more information.
Specifically, if you have a fresh Diem enlistment and machine, you can use the following to install basic build
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
