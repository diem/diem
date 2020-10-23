## Fuzzing support for Libra

This crate contains support for fuzzing Libra targets. This support sincludes:

* corpus generation with `proptest`
* automatically running failing examples with `cargo test`

### Prerequisites

Install [`cargo-fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html) if not already available: `cargo install cargo-fuzz`.

### Fuzzing a target

First, switch to the directory this README is in: `cd testsuite/libra-fuzzer`.

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

There are two ways to reproduce an issue to investigate a finding:

1. run the harness test (the code the fuzzer runs) directly
2. run the fuzzer code

The following command (with your own artifact contained in a similar path)
will run the harness test with your input:

```
cargo run --bin investigate -- -i artifacts/compiled_module/crash-5d7f403f
```

The following command will run libfuzzer on the relevant target with your input:

```
# build single fuzzer for target using instruction in the 'google oss-fuzz integration' section
./fuzzer input
```

Note that this should work out of the box for crashes,
but timeouts might need a `-timeout 25` argument to libfuzzer,
and out of memory might need a `-rss_limit_mb=2560` argumnent to libfuzzer.

See [Google OSS-Fuzz's documentation on reproducing bugs](https://google.github.io/oss-fuzz/advanced-topics/reproducing/) as well.

### Flamegraph

To obtain a flamegraph of a harness test, run the following command:

```sh
FUZZ_TARGET=compiled_module cargo flamegraph -p libra-fuzzer --bin flamegraph
```

It is good to first generate some corpus and run the fuzzer over it for a bit (to find new corpus). The larger corpus, the better flamegraph you will obtain.

### Fuzzing Coverage

To test coverage of our fuzzers you can run the following command with [grcov](https://github.com/mozilla/grcov):

```sh
RUSTFLAGS='--cfg feature="fuzzing"' CORPUS_PATH=fuzz/corpus cargo xtest --html-cov-dir <some path for html output> -p libra-fuzzer -- coverage
```

### Building a single fuzzer

To integrate our fuzzers with [Google OSS-Fuzz](https://github.com/google/oss-fuzz) project,
we need to have one binary per fuzzer.
This can also be handy when you want to analyze a fuzzer with tools like Instruments.
For this, build.rs can create a fuzzer binary based on an environement variable.
Use it as such:

```sh
cd libra/testsuite/libra-fuzzer
fuzz/google-oss-fuzz/build_fuzzer.sh ConsensusProposal .
./ConsensusProposal
```

### Troubleshooting

#### linking with CC failed

Are you on MacOS? Have you checked that Xcode is up-to-date?

#### My backtrace does not contain file names and line numbers

You need to use llvm-symbolizer, see https://github.com/rust-fuzz/cargo-fuzz/issues/160
