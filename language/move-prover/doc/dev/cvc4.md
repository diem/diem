# CVC4 Integration

The [CVC4] integration is an experimental feature which is still evolving. This page documents,
for tool developers, how to use CVC4 as a backend and extend the Move prover to use it right.

## Selecting CVC4

In the following we assume a command line for the move prover as in `mvp <arguments>`. See
the [user documentation](../user/prover-guide.md) how to set this up. For development, it
is recommended to use a `~/.mvprc` file for configuring the prover; the user documentation shows
how. Specifically, this file can be used to point to the Move standard library and
Diem framework as found in the local branch of the Diem repo.

To choose CVC4 as a backend, call the prover as in:

```shell
# mvp --use-cvc4 source.move

```

## Testing with CVC4

The provers testsuite supports multiple *feature groups* which constitute a particular
set of tests which are run with a specific configuration. By default, when `cargo test`
is executed in the prover crate, all groups are executed.

There is one group called `cvc4` which contains the enabled tests for CVC4. This group
is configured as follows at this point:

- Only unit tests contained in `move-prover/tests` are included. Diem framework tests
  are skipped.
- Those tests are only run locally and nightly, but not in CI.
- By default, cvc4 tests share the same baseline (`.exp` file) with the `default` test group
  which uses [Z3].
- Some cvc4 tests have their separate baseline, which is indicated by the
  tag `// separate_baseline: cvc4` in the source. Those tests represent cases where
  there is an issue (most of the time false positives); in some cases those are also legit
  differences resulting from different choices in the model.

For the configuration logic, see the [testsuite.rs](../../tests/testsuite.rs) code.

During development, it is often useful to focus on only running a specific test group.
This is done using `MVP_TEST_FEATURE=cvc4 cargo test`. For documentation of all test parameters, see
[here](../../tests/README.md).

## Obtaining the smtlib file

In order to analyze a problem based on the smtlib file Boogie generates for it, use the following
command line:

```shell
# mvp --generate-smt [ --verify-only <function-name> ] source.move

```

This will generate a file ending with `.smt` for each function in the Move source. The content
of those files is hermetic and contains all the settings which have been passed from upstream
to the solver.

## Specializing the Encoding for CVC4

There are multiple extension points which can be used to customize the prover. They are centered
around the [Tera] template system which provides conditionals and macro expansion. Tera is
quite similar to Django2 and is easy to understand while rather expressive.

The root template the prover includes in every verification problem is found in
[boogie-backend/src/prelude/prelude.bpl](../../boogie-backend/src/prelude/prelude.bpl). This
is templatized [Boogie] source. This includes other templates, like theories for vectors and
multisets, as well as implementations of native Move types found in
[prelude/native.bpl](../../boogie-backend/src/prelude/native.bpl).

### Accessing Backend Options

Inside of templates one has access to the backend options of the prover (`[backend]` section
in the toml config). To test in a template whether CVC4 is selected as a backend, use `{{options.
use_cvc4}}`. New options can be also added to `BoogieOptions` in Rust and will be accessible
this way.

### Replacing Vector Theories

The prover, in combination with Rust code and the templates, supports the concept of different
vector theories. There are multiple theories selectable via the option `--vector-theory`.
For a benchmark and description of the theories, see
[here](../../lab/data/vector-theories/notebook.pdf).

In order to add a new vector theory specific for CVC4, the following steps are needed:

- Write the theory, starting e.g. from the
  [vector_array_theory](../../boogie-backend/src/prelude/vector-array-theory.bpl). A new theory
  need to implement the same set of functions as seen there.
- Integrate the theory into the prover as follows:
  1. Extend the enumeration [VectorTheory](../../boogie-backend/src/options.rs) by an item to
     represent the new theory. Don't forget to implement the close-by `is_extensional` function
     appropriately for the new theory. This tells the prover whether the theory supports extensional
     equality provided the element values do.
  2. Now go to [boogie-backend/src/lib.rs](../../boogie-backend/src/lib.rs) and follow the patterns
     you find for other theories in order to connect the outcome of (1) to the prover. Once done,
     all is set and the new theory should be ready to use via the option
     `--vector-theory=MyEnumItemName`.

Notice that the theories are using generic Boogie in *monomorphization* mode. For the type
`Vec T`, the actual type reaching the smt backend will be `Vec_2923`, which is some instantiation
of this type. In general, the VCs reaching the SMT backend are fully monomorphized, either
via Boogies mechanism, or via explicit logic in the Move provers templates and codegen.

## Analyzing Results

The Move prover comes with tool support and conventions to systematically perform benchmarks
and other data-driven experiments. The crate which contains this support is found
at [move-prover/lab](../../lab).

> TODO: we should create a `lab/data/z3-cvc4` for a comparison test once the cvc4 integration
> passes unit tests. The existing `lab/data/new-boogie` can be used as a starting point.

For testing new vector theories, the existing [vector theory lab](../../lab/data/vector-theories)
can be extended, or a new lab created from it.


[Boogie]: https://github.com/boogie-org/boogie
[CVC4]: https://cvc4.github.io/
[Z3]: https://github.com/Z3Prover/z3
[Tera]: https://tera.netlify.app/docs
