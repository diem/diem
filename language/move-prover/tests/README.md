# Tests for Move Prover

This directory contains the tests for Move Prover. The tests are defined by the `.move` files in this tree,
as well as all the `.move` files in the [Diem framework](../../stdlib).

*Note*: in order to run these tests locally, you must have installed tools and setup a few environment variables.
See [`../doc/user/install.md`](../doc/user/install.md) for details. If the environment variables for
configuring the prover are not set as described there, all tests and this directory will trivially pass.

*Note*: these are baseline tests, with expectations of prover output stored in files ending in `.exp`. To update
those files, use `UPBL=1 cargo test`. To update or test a single file, you can also provide a fragment of the move
source path.

There is a convention for test cases under this directory. In general, there are two kinds of test cases, which can be
mixed in a file. The first type of test cases are "correct" Move functions which are expected to be proven so.
Another type of test cases are incorrect Move functions which are expected to be disproven, with the created errors
stored in so-called 'expectation baseline files' (`.exp`). The incorrect functions have suffix `_incorrect` in
their names, by convention. It is expected that only errors for functions with this suffix appear in `.exp` files.

`cargo test` will automatically detect all `.move` files under this directory and its sub-directories and let the Prover
attempt to prove each function in the file. Unlike `cargo run`, `cargo test` can detect various directives
in comments in the Move source:

- The line `// flag: <flag>` provides a flag to the Prover (see `cargo run -- --help` for  available flags). For
  example, use  `// flag: --verify=public` to restrict verification to public functions (by default, tests use
  `--verify=all`).
- You can also pass flags to test using the env variable `MVP_TEST_FLAGS`. This is a string as provided on
  the command line to the Move prover which can contain multiple flgs.
- The line `// no-boogie-test` instructs the test driver to not attempt to run boogie at all. This is to support
  negative tests where translation to boogie actually fails.


## Debugging Long Running Tests

By default, the prover uses a timeout of 40 seconds, which can be changed by the `-T=<seconds>` flag. Healthy tests
should never take that long to finish. To avoid flakes in continuous integration, you should test your tests to
be able to pass at least with `-T=20`. To do so use

```shell script
MVP_TEST_FLAGS="-T=20" cargo test -p move-prover
```

## Code coverage

Analyzing the test coverage of the diem repo is regularly done in CI, and the result updates the online report at
* https://ci-artifacts.diem.com/coverage/unit-coverage/latest/index.html
* https://codecov.io/gh/diem/diem (reports significantly less coverage due to panic unwinding being considered a branch)

Note that this report is based on the the coverage test when the environment variable `BOOGIE_EXE` is not set.
So, the coverage result may not be as accurate as expected because all verifications with Boogie/Z3 are skipped
during the test.

To run the coverage test locally, one can use `cargo xtest html-cov-dir="/some/dir"`.   Keep in mind what is compiled and run when
targeting a single crate is not the same as is run/built with multiple crates due to cargo's feature unification.

For any questions regarding code coverage, please use the Cadiem slack channel "#code_coverage".
