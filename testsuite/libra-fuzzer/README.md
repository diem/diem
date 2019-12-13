## Fuzzing support for Libra

This crate contains support for fuzzing Libra targets. This support
includes:
* corpus generation with `proptest`
* automatically running failing examples with `cargo test`

### Prerequisites

Install [`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html) if not already available: `cargo install cargo-fuzz`.

### Fuzzing a target

First, switch to the directory this README is in: `cd
testsuite/libra_fuzzer`.

To list out known fuzz targets, run `cargo run list`.

To be effective, fuzzing requires a corpus of existing inputs. This
crate contains support for generating corpuses with `proptest`. Generate
a corpus with `cargo run generate <target>`.

Once a corpus has been generated, the fuzzer is ready to use, simply run:

```
RUSTC_BOOTSTRAP=1 cargo run fuzz <target> -- --release --debug-assertions
```

For more options, run `cargo run -- --help`. Note that `RUSTC_BOOTSTRAP=1` is
required as `cargo fuzz` uses unstable compiler flags.

### Adding a new target

Fuzz targets go in `src/fuzz_targets/`. Adding a new target involves
creating a new type and implementing `FuzzTargetImpl` for it.

For examples, see the existing implementations in `src/fuzz_targets/`.

Remember to add your target to `ALL_TARGETS` in `src/fuzz_targets.rs`.
Once that has been done, `cargo run list` should list your new target.

### Debugging and testing artifacts

If the fuzzer finds a failing artifact, it will save the artifact to a
file inside the `fuzz` directory and print its path. To add this
artifact to the test suite, copy it to a file inside
`artifacts/<target>/`.

`cargo test` will now test the deserializer against the new artifact.
The test will likely fail at first use.

Note that `cargo test` runs each test in a separate process by default
to isolate failures and memory usage; if you're attaching a debugger and
are running a single test, set `NO_FORK=1` to disable forking.

Once the deserializer has been fixed, check the artifact into the
`artifacts/<target>/` directory. The artifact will then act as a
regression test in `cargo test` runs.

### Fuzzing Coverage

To test coverage of our fuzzers you can run the following command with [tarpaulin](https://crates.io/crates/cargo-tarpaulin):

```
CORPUS_PATH=fuzz/corpus cargo tarpaulin -p libra-fuzzer -- coverage
```
### Investigating an artifact

Running the following command (with your own artifact contained in a similar path)
will run the fuzzer with your input.

```
cargo run investigate -- -i artifacts/compiled_module/crash-5d7f403f
```

This is helpful to investigate and debug a binary in order to find the root cause
of a bug.
