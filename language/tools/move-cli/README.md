# Move CLI

This is a tool that provides an easy way to interact with Move, to
experiment writing and running Move code, and to experiment developing new
tools useful for Move development. To reflect this, the Move CLI commands are grouped into three main subcommands:
* **base commands**: are basic Move commands such as compiling or checking a set of modules. These do not rely on a Move Adapter implementation nor implementation of storage.
* **sandbox commands**: are commands that allow you to write Move modules and scripts, write and run scripts and tests, and view the resulting state of execution in a local sandboxed environment.
* **experimental commands**: are experimental commands that are currently in development.

## Installation
```shell
$ cargo install --path diem/language/tools/move-cli
```
or
```shell
$ cargo install --git https://github.com/diem/diem move-cli --branch main
```

This will install the `move` binary in your Cargo binary directory. On
macOS and Linux this is usually `~/.cargo/bin`. You'll want to make sure
this location is in your `PATH` environment variable.

Now you should be able to run the Move CLI:

```shell
$ move
Move 0.1.0
CLI frontend for Move compiler and VM

USAGE:
    move [FLAGS] [OPTIONS] <SUBCOMMAND>
  ...
```

We'll go through the most common Move CLI commands and flags here, however
you can find the complete list of commands available by calling `move
--help`.  Additionally, the complete list of flags and options available
for each Move CLI command can be found by passing the `--help` flag to it,
i.e., `move <command> --help`.

## Base Commands

The base commands provide wrappers with sane defaults around other commands
that are provided either by various Move tools, compiler, or prover.

The `move compile` command will compile either a specific set of modules, or all modules under the passed-in directory:

```shell
$ move compile <move_file_1> ... <move_file_n>
$ move compile <dir_1> ... <dir_n>
```

compiled modules will by default be stored in the `build` directory. You can
change where the output bytecode is saved by passing the the optional
`--build-dir` flag:

```shell
$ move --build-dir <path_to_save_to> compile <move_file_1> ... <move_file_n>
$ move --build-dir <path_to_save_to> compile <dir_1> ... <dir_n>
```

You can pass the `--check` flag to the `move compile` command to only compile and typecheck either a specific set of
modules, or all modules under the passed-in directory without emitting any compiled bytecode:

```shell
$ move compile --check <move_file_1> ... <move_file_n>
$ move compile --check <dir_1> ... <dir_n>
```

## Sandbox Commands

The sandbox allows you to experiment with writing and running Move code without
validators, a blockchain, or transactions. Persistent data is stored on-disk in
a directory structure that mimics the Move memory model

### Project structure

Each Move CLI project with a given `name` should have the following structure to
it:

```
name/
├── src # Directory containing all Move source modules
│   ├ ...
│   └── Module.move
└── scripts # Directory containing all Move scripts
    ├ ...
    └── script.move
```

Let's now create a Move project that we'll use for the code in this README and `cd` into it:

```shell
$ mkdir readme
$ cd readme
$ mkdir src
$ mkdir scripts
```

### Compiling and running scripts

Let's first start out with a simple script that prints its `signer`:

```rust
script {
use Std::Debug;
fun main(account: signer) {
    Debug::print(&account)
}
}
```

Place this in a file named `debug_script.move` under `scripts` and try

```shell
$ move sandbox run scripts/debug_script.move --signers 0xf
[debug] (&) { 0000000000000000000000000000000F }
```

The `--signers 0xf` argument indicates which account address(es) have signed
off on the script. Omitting `--signers` or passing multiple signers to this
single-`signer` script will trigger a type error.

### Passing arguments

The CLI supports passing non-`signer` arguments to `move sandbox run` via `--args`. The following argument types are supported:
* `bool` literals (`true`, `false`)
* `u64` literals (e.g., `10`, `58`)
* `address` literals (e.g., `0x12`, `0x0000000000000000000000000000000f`)
* hexadecimal strings (e.g., `'x"0012"'` will parse as the `vector<u8>` value `[00, 12]`)
* ASCII strings (e.g., `'b"hi"'` will parse as the `vector<u8>` value `[68, 69]`)

### Publishing new modules

When executing a transaction script you'll often want to call into different Move
modules like in the example above with the `Debug` module. New modules can be added to the `src`
directory in the directory where the CLI is being invoked (or a directory
of your choosing specified via the `--source-dir` flag). The `move sandbox run`
command will compile and publish each module source file in this directory
before running the given script. You can also compile and publish modules
separately if you want as well.

Try saving this code in `src/Test.move`:

```rust
address 0x2 {
module Test {
    use Std::Signer;

    struct Resource has key { i: u64 }

    public fun publish(account: &signer) {
        move_to(account, Resource { i: 10 })
    }

    public fun write(account: &signer, i: u64) acquires Resource {
        borrow_global_mut<Resource>(Signer::address_of(account)).i = i;
    }

    public fun unpublish(account: &signer) acquires Resource {
        let Resource { i: _ } = move_from(Signer::address_of(account));
  }
}
}
```

Now, try

```shell
$ move sandbox link
```

This will cause the CLI to compile and typecheck the modules under
`src`, but it won't publish the module bytecode under `storage`. You can
compile and publish the module by running the `move sandbox publish` command
(here we pass the `-v` or verbose flag to get a better understanding of what's
happening):

```shell
$ move sandbox publish -v
Compiling Move modules...
Found and compiled 1 modules
Publishing a new module 00000000000000000000000000000002::Test (wrote 253 bytes)
Wrote 253 bytes of module ID's and code
```

Now, if we take a look under `storage`, we will see the published bytecode
for our `Test` module:

```shell
$ ls storage/0x00000000000000000000000000000002/modules
Test.mv
```

We can also inspect the compiled bytecode using `move sandbox view`:

```shell
$ move sandbox view storage/0x00000000000000000000000000000002/modules/Test.mv
module 2.Test {
struct Resource has key {
  i: u64
}

public publish() {
  0: MoveLoc[0](Arg0: &signer)
  1: LdU64(10)
  2: Pack[0](Resource)
  3: MoveTo[0](Resource)
  4: Ret
}
public unpublish() {
  0: MoveLoc[0](Arg0: &signer)
  1: Call[3](address_of(&signer): address)
  2: MoveFrom[0](Resource)
  3: Unpack[0](Resource)
  4: Pop
  5: Ret
}
public write() {
  0: CopyLoc[1](Arg1: u64)
  1: MoveLoc[0](Arg0: &signer)
  2: Call[3](address_of(&signer): address)
  3: MutBorrowGlobal[0](Resource)
  4: MutBorrowField[0](Resource.i: u64)
  5: WriteRef
  6: Ret
}
}
```

You can also run the Move CLI with certain predefined modules or in
different [_modes_](#using-the-cli-with-modes-and-genesis-state) (such as
the `Debug` module above), in addition to defining your own Move modules,
we'll touch on this at the end of the README.

### Updating state

Let's exercise our new `Test` module by running the following script:

```rust
script {
use 0x2::Test;
fun main(account: signer) {
    Test::publish(&account)
}
}
```

This script invokes the `publish` function of our `Test` module, which will
publish a resource of type `Test::Resource` under the signer's account.
Let's first see what this script will change without committing those
changes first. We can do this by passing the `--dry-run` flag:

```shell
$ move sandbox run scripts/test_script.move --signers 0xf -v --dry-run
Compiling transaction script...
Changed resource(s) under 1 address(es):
  Changed 1 resource(s) under address 0000000000000000000000000000000F:
    Added type 0x2::Test::Resource: [10, 0, 0, 0, 0, 0, 0, 0] (wrote 40 bytes)
Wrote 40 bytes of resource ID's and data
      key 0x2::Test::Resource {
           i: 10
      }
Discarding changes; re-run without --dry-run if you would like to keep them.
```

Everything looks good, so we can run this again, but this time commit the
changes by removing the `--dry-run` flag:

```shell
$ move sandbox run scripts/test_script.move --signers 0xf -v
Compiling transaction script...
Changed resource(s) under 1 address(es):
  Changed 1 resource(s) under address 0000000000000000000000000000000F:
    Added type 0x2::Test::Resource: [10, 0, 0, 0, 0, 0, 0, 0] (wrote 40 bytes)
Wrote 40 bytes of resource ID's and data
      key 0x2::Test::Resource {
            i: 10
      }
```

While the verbose flag used above (`-v`) shows resource changes, it is also
possible to view them manually.
We can inspect the newly published resource using `move sandbox view` since
the change has been committed:

```shell
$ move sandbox view storage/0x0000000000000000000000000000000F/resources/0x00000000000000000000000000000002::Test::Resource.bcs
key 0x2::Test::Resource {
    i: 10
}
```

#### Cleaning state

Since state persists from one call to the Move CLI to another, there will
frequently be times where you want to start again at a clean state.  This
can be done using the `move sandbox clean` command which will remove the
`storage` directory:

```shell
$ move sandbox view storage/0x0000000000000000000000000000000F/resources/0x00000000000000000000000000000002::Test::Resource.bcs
resource 0x2::Test::Resource {
        i: 10
}
$ move sandbox clean
$ move sandbox view storage/0x0000000000000000000000000000000F/resources/0x00000000000000000000000000000002::Test::Resource.bcs
Error: `move sandbox view <file>` must point to a valid file under storage
```

### Testing with the Move CLI

The Move CLI also has a built-in testing framework. Each test is run
independently in its own sandbox so state does not persist from one test to
another.

Each test is structured as a directory consisting of an `args.txt` file that
specifies a sequence of Move CLI commands that should be run in that
directory, and whose structure piggybacks on the Move CLI project structure defined above.
Additionally, there must be an `args.exp` file that contain the expected
output from running the sequence of Move CLI commands specified in the
`args.txt` file for that test.

For example, if we wanted to create a Move CLI test that reran all of the
commands that we've seen so far, we could do so by adding an `args.txt`
to the `readme` directory that we created at the start and that we've been
adding scripts and modules to:

```
readme/
├── args.txt
├── scripts
│   ├── debug_script.move
│   └── test_script.move
└── src
    └── Test.move
```

And, where the `args.txt` file contains the following Move CLI commands:

```shell
$ cd ..
$ cat readme/args.txt
## Arg files can have comments!
sandbox run scripts/debug_script.move --signers 0xf
sandbox run scripts/debug_script.move --signers 0xf --mode bare
sandbox link
sandbox publish
sandbox view storage/0x00000000000000000000000000000002/modules/Test.mv
sandbox run scripts/test_script.move --signers 0xf -v --mode bare
sandbox view storage/0x0000000000000000000000000000000F/resources/0x00000000000000000000000000000002::Test::Resource.bcs
```

We can then use the `move sandbox test` command and point it at the `readme` directory to run each of these
Move CLI commands for us in sequence:

```shell
$ move sandbox test readme
...<snipped output>
0 / 1 test(s) passed.
Error: 1 / 1 test(s) failed.
```

However, as we see this test will fail since there is no `args.exp` file for the test
yet. We can generate this expectation file by setting the `UPDATE_BASELINE`
(or `UB` for short) environment variable when running the test:

```shell
$ UPDATE_BASELINE=1 move sandbox test readme
1 / 1 test(s) passed.
```

There should now be an `args.exp` file under the `readme` directory that
contains the expected output of running the sequence of Move CLI commands
in the `args.txt` file:

```shell
$ cat readme/args.exp
Command `sandbox run scripts/debug_script.move --signers 0xf`:
[debug] (&) { 0000000000000000000000000000000F }
Command `sandbox run scripts/debug_script.move --signers 0xf --mode bare`:
...
```

The scaffolding for a new test that follows the above structure for tests can be created
by passing the `--create` flag to `move sandbox test` along with the name of the test that you wish to create:

```
$ move sandbox test new_test_name --create
$ tree new_test_name
new_test_name
├── args.txt
├── scripts
└── src
```

#### Testing with code coverage tracking

Code coverage has been an important metric in software testing. In Move CLI, we
address the need for code coverage information with an additional flag,
`--track-cov`, that can be passed to the `move sandbox test` command.

Note: To view coverage information, the Move CLI must be installed with the `--debug` flag;
i.e., `cargo install --debug --path diem/language/tools/move-cli`.

Using our running example to illustrate:
```shell
$ move sandbox test readme --track-cov
1 / 1 test(s) passed.
Module 00000000000000000000000000000002::Test
        fun publish
                total: 5
                covered: 5
                % coverage: 100.00
        fun unpublish
                total: 6
                covered: 0
                % coverage: 0.00
        fun write
                total: 7
                covered: 0
                % coverage: 0.00
>>> % Module coverage: 27.78
```

The output indicates that not only the test is passed, but also that 100%
instruction coverage is observed in the `publish` funciton. This is expected
as the whole purpose of our `test_script.move` is to run the `publish` function.
At the same time, the other two functions, `unpublish` and `write`, are never
executed, making the average coverage 27.78% for the whole `Test` module.

Internally, Move CLI uses the tracing feature provided by the Move VM to record
which instructions in the compiled bytecode are executed and uses this
information to calculate code coverage. Instruction coverage in Move can
usually serve the purpose of line coverage in common C/C++/Rust coverage
tracking tools.

Note that the coverage information is aggregated across multiple `run` commands
in `args.txt`. To illustrate this, suppose that we have another test script,
`test_unpublish_script.move`, under `readme/scripts` with the following
content:

```rust
script {
use 0x2::Test;
fun main(account: signer) {
    Test::unpublish(&account)
}
}
```

We further add a new command to the end of `args.txt`
(`args.exp` needs to be updated too).
```shell
sandbox run scripts/test_unpublish_script.move --signers 0xf -v --mode bare
```

Now we can re-test the `readme` again
```shell
$ move sandbox test readme --track-cov
1 / 1 test(s) passed.
Module 00000000000000000000000000000002::Test
        fun publish
                total: 5
                covered: 5
                % coverage: 100.00
        fun unpublish
                total: 6
                covered: 6
                % coverage: 100.00
        fun write
                total: 7
                covered: 0
                % coverage: 0.00
>>> % Module coverage: 61.11
```

This time, note that the `unpublish` function is 100% covered too and the
overall module coverage is boosted to 61.11%.

### Using the CLI with modes and genesis state

The CLI offers a couple of different _modes_ that it can be run with---each
mode specifies a set of predefined modules that will be used during
compilation and execution. The mode to be used during a CLI action is specified
by passing the `--mode <mode>` flag to the Move CLI. The modes that can be used
are the following:

* **bare:** No predefined modules will be included during the compilation and
  execution of a script or module (but user-defined modules will). E.g., using
  the `debug_script.move` example above:

	```shell
	$ move sandbox run scripts/debug_script.move --signers 0xf --mode bare
	error:

	   ┌── scripts/debug_script.move:2:5 ───
	   │
	 2 │ use Std::Debug;
	   │     ^^^^^^^^^^ Invalid 'use'. Unbound module: '0x1::Debug'
	   │

  error:

    ┌── scripts/debug_script.move:4:5 ───
    │
  4 │     Debug::print(&account)
    │     ^^^^^ Unbound module alias 'Debug'
    │
	```

* **stdlib:** This includes a small set of utility modules published under the
  `0x1` address. **This is the default mode if no mode is supplied**. The set of
  Move modules that are included in this mode are the following:
  [`Debug`](https://github.com/diem/diem/blob/main/language/diem-framework/nursery/Debug.move),
  [`Errors`](https://github.com/diem/diem/blob/main/language/diem-framework/modules/Errors.move),
  [`Event`](https://github.com/diem/diem/blob/main/language/diem-framework/modules/Event.move),
  [`FixedPoint32`](https://github.com/diem/diem/blob/main/language/diem-framework/modules/FixedPoint32.move),
  [`Hash`](https://github.com/diem/diem/blob/main/language/diem-framework/modules/Hash.move),
  [`BCS`](https://github.com/diem/diem/blob/main/language/diem-framework/modules/BCS.move),
  [`Option`](https://github.com/diem/diem/blob/main/language/diem-framework/modules/Option.move),
  [`Vector`](https://github.com/diem/diem/blob/main/language/diem-framework/modules/Vector.move), and
  [`Signer`](https://github.com/diem/diem/blob/main/language/diem-framework/modules/Signer.move).

* **diem:** This includes all of the modules in the `stdlib` mode, along with
  all of the other modules that comprise the Diem Framework as defined
  [here](https://github.com/diem/diem/blob/main/language/diem-framework/modules/doc/overview.md).

#### Running with genesis state

You can run the Move CLI using the modules from the Diem Framework by using
the `--mode diem` mode flag in your commands. However, a number of the Diem
Framework modules require a specific initialization sequence to be run in
genesis in order to function properly.

The `tests/testsuite/liba_smoke` test uses the CLI to run a fairly
realistic Diem genesis setup to initialize these modules, and also contains a
few basic transactions. If you want to experiment with running
different transactions with the Diem semantics, you can use this test as a
starting point, and add your scripts/modules as you wish. You'll need to update
the `args.txt` file as well to exercise these new scripts as well, and then
you can run them just like any other Move CLI test:

```shell
$ move sandbox test ./tests/testsuite/diem_smoke
```

### Detecting breaking changes

The `move sandbox publish` command automatically detects when upgrading a module may lead to a breaking change.
There are two kinds of breaking changes:

* Linking compatibility (e.g., removing or changing the signature of a public function that is invoked by other modules, removing a
struct or resource type used by other modules)
* Layout compatibility (e.g., adding/removing a resource or struct field)

The breaking changes analysis performed by `move sandbox publish` is necessarily conservative. For example, say we `move sandbox publish` the following
module:

```
address 0x2 {
module M {
    struct S has key { f: u64, g: u64 }
}
}
```

and then wish to upgrade it to the following:

```
address 0x2 {
module M {
    struct S has key { f: u64 }
}
}
```

Running `move sandbox publish` on this new version will fail:

```
Breaking change detected--publishing aborted. Re-run with --ignore-breaking-changes to publish anyway.
Error: Layout API for structs of module 00000000000000000000000000000002::M has changed. Need to do a data migration of published structs
```

In this case, we know we have not published any instances of `S` in global storage, so it is safe to re-run `move sandbox publish --ignore-breaking-changes` (as recommended).
We can double-check that this was not a breaking change by running `move sandbox doctor`.
This handy command runs exhaustive sanity checks on global storage to detect any breaking changes that occurred in the past:
* All modules pass the bytecode verifier
* All modules link against their dependencies
* All resources deserialize according to their declared types
* All events deserialize according to their declared types
