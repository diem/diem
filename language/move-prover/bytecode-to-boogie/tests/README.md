# How to add a new test

We need lots and lots of examples of verification problems in order to stabilize the Move Prover. If you find a
problem with the Move Prover, please consider to add a test which captures it, and send a PR.

Adding a test is simple:

1. Create an MVIR source in the [`../test_mvir`](../test_mvir) directory representing the problem.
   See [`../test_mvir/test-specs-verify.rs`](../test_mvir/verify-create-resource.mvir) for an example.
2. Add a `#[test] fn` to the Rust source [`./prover_tests.rs`](./prover_tests.rs):

   ```rust
   #[test]
   fn test_my_test() {
       test(VERIFY, &["test_mvir/my-test.mvir"]);
   }
   ```
3. To run the test first ensure that you have set the environment variables `BOOGIE_EXE` and `Z3_EXE`. (The command
   line interface of MVP also uses those variables, so its a good idea to have them in your `.bashrc`.) Then
   run ```cargo test test_my_test```, where `test_my_test` is the name of the test function you have introduced in
   step 2.
4. Annotate a line which produces an error with the ```//! <error text>``` comment, where `<error text>` is any
   substring of the error text.

   > *Note*: the actual position or even order of the comment does not matter yet, the current algorithm simply
   > eliminates each error message which matches a comment string. A comment string can be only used once, and the
   > test fails if either not all errors are eliminated, or not all comments have been used.

   If the produced error is actually unexpected, please add a `TODO` and explain what you expected, so the issue
   can be investigated.
5. At the latest before you send a PR, also re-generate the golden files:
   ```shell script
   touch tests/translator_tests.rs
   VERIFY_BPL_GOLDEN=1 REGENERATE_GOLDENFILES=1 cargo test
   ```
