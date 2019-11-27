This directory contains a set of so-called golden files containing compilation results
of the boogie translator. Those files are used for regression testing of the translator,
as well as for documenting the translation scheme.

The tests are *not* run by default but are guarded behind an environment variable. This is to
avoid integration test failures for unrelated upstream changes in the VM, mvir, or standard
libraries. For changes to the move prover, please run those tests manually and include the
result in your PR.

When running or updating the tests, keep in mind that upstream changes may have made the
golden files out-of-date independent of your changes.

To run those tests, use:

```shell script
VERIFY_BPL_GOLDEN=1 cargo test --test translator_tests
```

To update the golden files, use:

```shell script
VERIFY_BPL_GOLDEN=1 REGENERATE_GOLDENFILES=1 cargo test --test translator_tests
```

By careful with cargo caching of test results. Cargo doesn't know about the semantics of environment
variables, and may not actually rerun tests for the different settings above. You may have to use

```shell script
touch tests/driver/mod.rs
```

... to enforce rerunning.
