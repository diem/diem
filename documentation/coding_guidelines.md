---
id: coding-guidelines
title: Coding Guidelines
---

This document describes the coding guidelines for the Diem Core Rust codebase.

## Code formatting

All code formatting is enforced with [rustfmt](https://github.com/rust-lang/rustfmt) with a project-specific configuration.  Below is an example command to adhere to the Diem Core project conventions.

```
diem$ cargo xfmt
```

## Code analysis

[Clippy](https://github.com/rust-lang/rust-clippy) is used to catch common mistakes and is run as a part of continuous integration.  Before submitting your code for review, you can run clippy with our configuration:

```
diem$ cargo xclippy --all-targets
```

In general, we follow the recommendations from [rust-lang-nursery](https://rust-lang-nursery.github.io/api-guidelines/about.html) and [The Rust Programming Language](https://doc.rust-lang.org/book/).  The remainder of this guide provides detailed guidelines on specific topics in order to achieve uniformity of the codebase.

## Code documentation

Any public fields, functions, and methods should be documented with [Rustdoc](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments).

 Please follow the conventions as detailed below for modules, structs, enums, and functions.  The *single line* is used as a preview when navigating Rustdoc.  As an example, see the 'Structs' and 'Enums' sections in the [collections](https://doc.rust-lang.org/std/collections/index.html) Rustdoc.

 ```
 /// [Single line] One line summary description
 ///
 /// [Longer description] Multiple lines, inline code
 /// examples, invariants, purpose, usage, etc.
 [Attributes] If attributes exist, add after Rustdoc
 ```

Example below:

```rust
/// Represents (x, y) of a 2-dimensional grid
///
/// A line is defined by 2 instances.
/// A plane is defined by 3 instances.
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}
```

### Terminology

The Diem codebase uses inclusive terminology (similar to other projects such as [the Linux kernel](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=49decddd39e5f6132ccd7d9fdc3d7c470b0061bb)).  The terms below are recommended when appropriate.
* allowlist - a set of entities allowed access
* blocklist - a set of entities that are blocked from access
* primary/leader/main - a primary entity
* secondary/replica/follower - a secondary entity

### Constants and fields

Describe the purpose and definition of this data. If the unit is a measurement of time, include it, e.g., `TIMEOUT_MS` for timeout in milliseconds.

### Functions and methods

Document the following for each function:

* The action the method performs - “This method *adds* a new transaction to the mempool.” Use *active voice* and *present tense* (i.e. adds/creates/checks/updates/deletes).
* Describe how and why to use this method.
* Any condition that must be met _before_ calling the method.
* State conditions under which the function will `panic!()` or returns an `Error`
* Brief description of return values.
* Any special behavior that is not obvious

### README.md for top-level directories and other major components

Each major component of Diem Core needs to have a `README.md` file. Major components are:
* top-level directories (e.g. `diem/network`, `diem/language`)
* the most important crates in the system (e.g. `vm-runtime`)

This file should contain:

 * The *conceptual* *documentation* of the component.
 * A link to the external API documentation for the component.
 * A link to the main license of the project.
 * A link to the main contributing guide for the project.

A template for readmes:

```markdown
# Component Name

[Summary line: Start with one sentence about this component.]

## Overview

* Describe the purpose of this component and how the code in
this directory works.
* Describe the interaction of the code in this directory with
the other components.
* Describe the security model and assumptions about the crates
in this directory. Examples of how to describe the security
assumptions will be added in the future.

## Implementation Details

* Describe how the component is modeled. For example, why is the
  code organized the way it is?
* Other relevant implementation details.

## API Documentation

For the external API of this crate refer to [Link to rustdoc API].

[For a top-level directory, link to the most important APIs within.]

## Contributing

Refer to the Diem Project contributing guide [LINK].

## License

Refer to the Diem Project License [LINK].
```

A good example of README.md is `diem/network/README.md` that describes the networking crate.

## Binary, Argument, and Crate Naming

Most tools that we use everyday (rustc, cargo, git, rg, etc.) use dashes `-` as
a separator for binary names and arguments and the [GNU software
manual](https://www.gnu.org/software/libc/manual/html_node/Argument-Syntax.html)
dictates that long options should "consist of `--` followed by a name made of
alphanumeric characters and dashes". As such dashes `-` should be used as
separators in both binary names and command line arguments.

In addition, it is generally accepted by many in the Rust community that dashes
`-` should be used as separators in crate names, i.e. `x25519-dalek`.

## Code suggestions

In the following sections, we have suggested some best practices for a uniform codebase. We will investigate and identify the practices that can be enforced using Clippy. This information will evolve and improve over time.

### Attributes

Make sure to use the appropriate attributes for handling dead code:

```
// For code that is intended for production usage in the future
#[allow(dead_code)]
// For code that is only intended for testing and
// has no intended production use
#[cfg(test)]
```

### Avoid Deref polymorphism

Don't abuse the Deref trait to emulate inheritance between structs, and thus reuse methods.  For more information, read [here](https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deref.md).

### Comments

We recommend that you use `//` and `///` comments rather than block comments `/* ... */` for uniformity and simpler grepping.

### Concurrent types

Concurrent types such as [`CHashMap`](https://docs.rs/crate/chashmap), [`AtomicUsize`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html), etc. have an immutable borrow on self i.e. `fn foo_mut(&self,...)` in order to support concurrent access on interior mutating methods. Good practices (such as those in the examples mentioned) avoid exposing synchronization primitives externally (e.g. `Mutex`, `RwLock`) and document the method semantics and invariants clearly.

*When to use channels vs concurrent types?*

Listed below are high-level suggestions based on experience:

* Channels are for ownership transfer, decoupling of types, and coarse-grained messages.  They fit well for transferring ownership of data, distributing units of work, and communicating async results.  Furthermore, they help break circular dependencies (e.g. `struct Foo` contains an `Arc<Bar>` and `struct Bar` contains an `Arc<Foo>` that leads to complex initialization).

* Concurrent types (e.g. such as [`CHashMap`](https://docs.rs/crate/chashmap) or structs that have interior mutability building on [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html), [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html), etc.) are better suited for caches and states.

### Error handling

Error handling suggestions follow the [Rust book guidance](https://doc.rust-lang.org/book/ch09-00-error-handling.html).  Rust groups errors into two major categories: recoverable and unrecoverable errors.  Recoverable errors should be handled with [Result](https://doc.rust-lang.org/std/result/).  Our suggestions on unrecoverable errors are listed below:

*Fallible functions*

* `duration_since_epoch()` - to obtain the unix time, call the function provided by `diem-infallible`.
* `RwLock` and `Mutex` - Instead of calling `unwrap()` on the standard library implementations of these functions, use the infallible equivalent types that we provide in `diem-infallible`.

*Panic*

* `unwrap()` - Unwrap should only be used for test code.  For all other use cases, prefer `expect()`. The only exception is if the error message is custom-generated, in which case use `.unwrap_or_else(|| panic!("error: {}", foo))`.
* `expect()` - Expect should be invoked when a system invariant is expected to be preserved.  `expect()` is preferred over `unwrap()` and should contain a detailed error message on failure in most cases.
* `assert!()` - This macro is kept in both debug/release and should be used to protect invariants of the system as necessary.
* `unreachable!()` - This macro will panic on code that should not be reached (violating an invariant) and can be used where appropriate.

In production (non-test) code, outside of lock management,  all unrecoverable errors should be cleanly documented describing why said event is unrecoverable. For example, if the system is now in a bad state, state what that state is and the motivation for why a crash / restart is more effective than resolving it within a running system, and what if any steps an operator would need to take to resolve the issue.

### Generics

Generics allow dynamic behavior (similar to [`trait`](https://doc.rust-lang.org/book/ch10-02-traits.html) methods) with static dispatch.  As the number of generic type parameters increases, the difficulty of using the type/method also increases (e.g. consider the combination of trait bounds required for this type, duplicate trait bounds on related types, etc.).  In order to avoid this complexity, we generally try to avoid using a large number of generic type parameters.  We have found that converting code with a large number of generic objects to trait objects with dynamic dispatch often simplifies our code.

### Getters/setters

In general, we follow naming recommendations for getters as specified [here](https://rust-lang.github.io/api-guidelines/naming.html#getter-names-follow-rust-convention-c-getter) and for setters as defined [here](https://github.com/rust-lang/rfcs/blob/master/text/0344-conventions-galore.md#gettersetter-apis).

Getters/setters should be avoided for [`struct`](https://doc.rust-lang.org/book/ch05-00-structs.html) types in the C spirit: compound, passive data structures without internal invariants. Adding them only increases the complexity and number of lines of code without improving the developer experience.

```rust
struct Foo {
    size: usize,
    key_to_value: HashMap<u32, u32>
}

impl Foo {
    /// Simple getter follows xxx pattern
    fn size(&self) -> usize {
        self.size
    }

    /// Setter follows set_xxx pattern
    fn set_foo(&mut self, size: usize){
        self.size = size;
    }

    /// Complex getter follows get_xxx pattern
    fn get_value(&self, key: u32) -> Option<&u32> {
        self.key_to_value.get(&key)
    }
}
```

### Integer Arithmetic

As every integer operation (`+`, `-`, `/`, `*`, etc.) implies edge-cases (e.g. overflows `u64::MAX + 1`, underflows `0u64 -1`, division by zero, etc.),
we use checked arithmetic instead of directly using math symbols.
It forces us to think of edge-cases, and handle them explicitely.
This is a brief and simplified mini guide of the different functions that exist to handle integer arithmetic:

* [checked_](https://doc.rust-lang.org/std/primitive.u32.html#method.checked_add): use this function if you want to handle overflows and underflows as a special edge-case. It returns `None` if an underflow or overflow has happened, and `Some(operation_result)` otherwise.
* [overflowing_](https://doc.rust-lang.org/std/primitive.u32.html#method.overflowing_add): use this function if you want the result of an overflow to potentially wrap around (e.g. `u64::MAX.overflow_add(10) == (9, true)`). It returns the underflowed or overflowed result as well as a flag indicating if an overflow has occured or not.
* [wrapping_](https://doc.rust-lang.org/std/primitive.u32.html#method.wrapping_add): this is similar to overflowing operations, except that it returns the result directly. Use this function if you are sure that you want to handle underflows and overflows by wrapping around.
* [saturating_](https://doc.rust-lang.org/std/primitive.u32.html#method.saturating_add): if an overflow occurs, the result is kept within the boundary of the type (e.g. `u64::MAX.saturating_add(1) == u64::MAX`).

### Logging

We currently use [log](https://docs.rs/log/) for logging.

* [error!](https://docs.rs/log/0.4.10/log/macro.error.html) - Error-level messages have the highest urgency in [log](https://docs.rs/log/).  An unexpected error has occurred (e.g. exceeded the maximum number of retries to complete an RPC or inability to store data to local storage).
* [warn!](https://docs.rs/log/0.4.4.10/log/macro.warn.html) - Warn-level messages help notify admins about automatically handled issues (e.g. retrying a failed network connection or receiving the same message multiple times, etc.).
* [info!](https://docs.rs/log/0.4.4.10/log/macro.info.html) - Info-level messages are well suited for "one-time" events (such as logging state on one-time startup and shutdown) or periodic events that are not frequently occurring - e.g. changing the validator set every day.
* [debug!](https://docs.rs/log/0.4.4.10/log/macro.debug.html) - Debug-level messages can occur frequently (i.e. potentially > 1 message per second) and are not typically expected to be enabled in production.
* [trace!](https://docs.rs/log/0.4.4.10/log/macro.trace.html) - Trace-level logging is typically only used for function entry/exit.

### Testing

*Unit tests*

We follow the general guidance provided [here](https://doc.rust-lang.org/book/ch11-03-test-organization.html). Ideally, all code should be unit tested.  Unit tests should be in the same file as the code it is testing though in a distinct module, using the following syntax:

```rust
struct Foo {
}

impl Foo {
  pub fn magic_number() -> u8 {
    42
  }
}

#[cfg(test)]
mod tests {
  #test
  fn verify_magic_number() {
    assert_eq!(Foo::magic_number(), 42);
  }
}
```

*Property-based tests*

Diem contains [property-based tests](https://blog.jessitron.com/2013/04/25/property-based-testing-what-is-it/) written in Rust using the [`proptest` framework](https://github.com/AltSysrq/proptest). Property-based tests generate random test cases and assert that invariants, also called *properties*, hold for the code under test.

Some examples of properties tested in Diem:

* Every serializer and deserializer pair is tested for correctness with random inputs to the serializer. Any pair of functions that are inverses of each other can be tested this way.
* The results of executing common transactions through the VM are tested using randomly generated scenarios and verified with an *Oracle*.

A tutorial for `proptest` can be found in the [`proptest` book](https://altsysrq.github.io/proptest-book/proptest/getting-started.html).

References:

* [What is Property Based Testing?](https://hypothesis.works/articles/what-is-property-based-testing/) (includes a comparison with fuzzing)
* [An introduction to property-based testing](https://fsharpforfunandprofit.com/posts/property-based-testing/)
* [Choosing properties for property-based testing](https://fsharpforfunandprofit.com/posts/property-based-testing-2/)

*Fuzzing*

Diem contains harnesses for fuzzing crash-prone code like deserializers, using [`libFuzzer`](https://llvm.org/docs/LibFuzzer.html) through [`cargo fuzz`](https://rust-fuzz.github.io/book/cargo-fuzz.html). For more examples, see the `testsuite/diem_fuzzer` directory.

### Conditional compilation of tests

Diem [conditionally
compiles](https://doc.rust-lang.org/stable/reference/conditional-compilation.html)
code that is *only relevant for tests, but does not consist of tests* (unitary
or otherwise). Examples of this include proptest strategies, implementations
and derivations of specific traits (e.g. the occasional `Clone`), helper
functions, etc. Since Cargo is [currently not equipped for automatically activating features
in tests/benchmarks](https://github.com/rust-lang/cargo/issues/2911), we rely on two
conditions to perform this conditional compilation:
- the test flag, which is activated by dependent test code in the same crate
  as the conditional test-only code.
- the `fuzzing` custom feature, which is used to enable fuzzing and testing
related code in downstream crates. Note that this must be passed explicitly to
`cargo xtest` and `cargo x bench`. Never use this in `[dependencies]` unless
the crate is only for testing.

As a consequence, it is recommended that you set up your test-only code in the following fashion.

**For production crates:**

Production crates are defined as the set of crates that create externally published artifacts, e.g. the Diem validator,
the Move compiler, and so on.

For the sake of example, we'll consider you are defining a test-only helper function `foo` in `foo_crate`:

1. Define the `fuzzing` flag in `foo_crate/Cargo.toml` and make it non-default:
    ```toml
    [features]
    default = []
    fuzzing = []
    ```
2. Annotate your test-only helper `foo` with both the `test` flag (for in-crate callers) and the `"fuzzing"` custom feature (for out-of-crate callers):
    ```rust
    #[cfg(any(test, feature = "fuzzing"))]
    fn foo() { ... }
    ```
3. (optional) Use `cfg_attr` to make test-only trait derivations conditional:
    ```rust
    #[cfg_attr(any(test, feature = "testing"), derive(FooTrait))]
    #[derive(Debug, Display, ...)] // inconditional derivations
    struct Foo { ... }
    ```
4. (optional) Set up feature transitivity for crates that call crates that have test-only members. Let's say it's the case of `bar_crate`, which, through its test helpers, calls into `foo_crate` to use your test-only `foo`. Here's how you would set up `bar_crate/Cargo.toml`:
    ```toml
    [features]
    default = []
    fuzzing = ["foo_crate/fuzzing"]
    ```

**For test-only crates:**

Test-only crates do not create published artifacts. They consist of tests, benchmarks or other code that verifies
the correctness or performance of published artifacts. Test-only crates are
explicitly listed in `x.toml` under `[workspace.test-only]`.

These crates do not need to use the above setup. Instead, they can enable the `fuzzing` feature in production crates
directly.

```toml
[dependencies]
foo_crate = { path = "...", features = ["fuzzing"] }
```

*A final note on integration tests*: All tests that use conditional test-only
elements in another crate need to activate the "fuzzing" feature through the
`[features]` section in their `Cargo.toml`. [Integration
tests](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html)
can neither rely on the `test` flag nor do they have a proper `Cargo.toml` for
feature activation. In the Diem codebase, we therefore recommend that
*integration tests which depend on test-only code in their tested crate* be
extracted to their own test-only crate. See `language/move-binary-format/serializer_tests`
for an example of such an extracted integration test.

*Note for developers*: The reason we use a feature re-export (in the `[features]` section of the `Cargo.toml` is that a profile is not enough to activate the `"fuzzing"` feature flag. See [cargo-issue #291](https://github.com/rust-lang/cargo/issues/2911) for details).
