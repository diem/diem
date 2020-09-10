# Tests for Move Prover

This directory contains the tests for Move Prover (or Prover for short). More specifically, `sources/stdlib/modules/`
contains the clones of the current Move standard modules. `sources/` contains various Move functions
(with specifications) to test Prover. The functions are grouped into different `.move` files in the directory,
and further grouped into sections within a file divided e.g. by a dashed comment line (e.g., `// ----`).
Please see `source/arithm.move` for example.

There is a convention for test cases under this directory. First of all, all functions defined as test cases should
be valid, passing the syntax & type check. In addition, there are two kinds of test cases, which can be
mixed in a file. The first type of test cases are "correct" Move functions which are expected to be proved by Prover.
Another type of test cases are incorrect Move functions which are expected to be disproved by Prover.
The incorrect functions have suffix `_incorrect` in their names for indication (this indication is currently for
humans, not for the automated test infrastructure).

`cargo test` will automatically detect all `.move` files under this directory and its sub-directories and let the Prover
attempt to prove each function in the file. Unlike `cargo run`, `cargo test` can detect various directives
in comments in the Move source:

- The line `// flag: <flag>` provides a flag to the Prover (see `cargo run -- --help` for  available flags). For
  example, use  `// flag: --verify=public` to restrict verification to public functions (by default, tests use
  `--verify=all`, or `// flag: --boogie=-noVerify` to turn off Boogie verification.
- You can also pass flags to test using the env variable `MVP_TEST_FLAGS`. This is a string like provided on
  the command line to the Move prover which can contain multiple flgs.
- The line `// no-boogie-test` instructs the test driver to not attempt to run boogie at all. This is to support
  negative tests where translation to boogie actually fails.

For each `.move` file, there is a corresponding baseline file (`.exp`) which stores the expected Prover's output
for the `.move` file. `cargo test` invokes Prover against each `.move` file, and examine the output of Prover.
The test is considered to "fail" if the output of Prover is different from the expected output stored in the
corresponding `.exp` file. To update the baseline file, run the test with setting the env variable `UPBL=1`
(i.e., run `UPBL=1 cargo test`). Also, one can run the individual test by giving a specific filename
(for example, `cargo test arithm.move`, and `UPBL=1 cargo test arithm.move`). Note that Prover currently skips
proving some functions such as the native functions and the functions in stanard vector module
(`sources/stdlib/modules/vector.move`) even though these functions come with specifications.

Lastly, if the environment variables such as `BOOGIE_EXE` and `Z3_EXE` are not defined, `cargo test` will only
partially test Prover without invoking Boogie (e.g., only testing the translation to Boogie). The
instruction on how to set the environment variables can be found in `../scripts/README.md`.

## Debugging Long Running Tests

By default, the prover uses a timeout of 40 seconds, which can be changed by the `-T=<seconds>` flag. Healthy tests
should never take that long to finish. To avoid flakes in continuous integration, you should test your tests to
be able to pass at least with `-T=20`. To do so use

```shell script
MVP_TEST_FLAGS="-T=20" cargo test -p move-prover
```

## Code coverage

Analyzing the test coverage of the libra repo is regularly done in CI, and the result updates the online report at
* https://codecov.io/gh/libra/libra

Note that this report is based on the the coverage test when the environment variable `BOOGIE_EXE` is not set.
So, the coverage result may not be as accurate as expected because all verifications with Boogie/Z3 are skipped
during the test.

To run the coverage test locally, one can use `scripts/coverage_report.sh`. However, note that there seems to be
a small accuracy issue here too because some coverage lines are spilled in `lcov` conversion from `grcov`.

For any questions regarding code coverage, please use the Calibra slack channel "#code_coverage".
