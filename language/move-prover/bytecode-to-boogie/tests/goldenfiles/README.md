This directory contains a set of so-called golden files containing compilation results
of the boogie translator. Those files are used for optional regression testing of the translator,
as well as for documenting the translation scheme.

The tests are not run by default but are behind a feature flag. This is because upstream changes in the VM, mvir, or
standard libraries may result in breaking them.

To run those tests, use:

```shell script
cargo test --test translator_test --features golden
```

To update the golden files, use:

```shell script
REGENERATE_GOLDENFILES=1 cargo test --test translator_test --features golden
```
