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

To list out known fuzz targets, run `cargo run --bin libra-fuzzer list`.

To be effective, fuzzing requires a corpus of existing inputs. This
crate contains support for generating corpuses with `proptest`. Generate
a corpus with `cargo run --bin libra-fuzzer generate <target>`.

Once a corpus has been generated, the fuzzer is ready to use, simply run:

```
RUSTC_BOOTSTRAP=1 cargo run --bin libra-fuzzer fuzz <target> -- --release --debug-assertions
```

For more options, run `cargo run --bin libra-fuzzer -- --help`. Note that `RUSTC_BOOTSTRAP=1` is
required as `cargo fuzz` uses unstable compiler flags.

### Adding a new target

Fuzz targets go in `src/fuzz_targets/`. Adding a new target involves
creating a new type and implementing `FuzzTargetImpl` for it.

For examples, see the existing implementations in `src/fuzz_targets/`.

Remember to add your target to `ALL_TARGETS` in `src/fuzz_targets.rs`.
Once that has been done, `cargo run --bin libra-fuzzer list` should list your new target.

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
cargo run --bin investigate -- -i artifacts/compiled_module/crash-5d7f403f
```

This is helpful to investigate and debug a binary in order to find the root cause
of a bug.

### Google OSS-Fuzz Integration

To integrate our fuzzers with [Google OSS-Fuzz](https://github.com/google/oss-fuzz) project,
we need to have one binary per fuzzer.
For this, build.rs can create a fuzzer binary based on an environement variable.
Use it as such:

```sh
FUZZ_TARGET="consensus_proposal" cargo build --manifest-path fuzz/Cargo.toml --bin fuzzer_builder
```

Note that you might want to add more flags [[1]](https://github.com/rust-fuzz/cargo-fuzz/blob/2243de096b15b79b719ce7489f014d7d8ce197ee/src/project.rs#L153)[[2]](https://github.com/rust-fuzz/cargo-fuzz/blob/2243de096b15b79b719ce7489f014d7d8ce197ee/src/project.rs#L174).

For example for MacOS:

```sh
cd fuzz
ASAN_OPTIONS=detect_odr_violation=0 RUSTC_BOOTSTRAP=1 RUSTFLAGS="--cfg fuzzing -Cpasses=sancov -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-trace-compares -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-trace-geps -Cllvm-args=-sanitizer-coverage-prune-blocks=0 -Cllvm-args=-sanitizer-coverage-pc-table -Clink-dead-code -Zsanitizer=address -Cdebug-assertions" FUZZ_TARGET="vm_value" cargo build --verbose --target x86_64-apple-darwin --bin fuzzer_builder
```

### Troubleshooting

#### linking with CC failed

Are you on MacOS? Have you checked that Xcode is up-to-date?
