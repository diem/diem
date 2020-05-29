# Move Prover User Guide (DRAFT)

This is the user guide for the Move prover. This document does not describe the Move specification language
or the conceptual design of the Move prover; we refer to elsewhere for this.

## Running the Prover

To run the Move prover while working in the Libra tree, we recommend to use `cargo run`. You can define an alias
as below in your `.bashrc` (or other shell configuration) to simplify this:

```shell script
alias mvp="cargo run --release --quiet --package move-prover --"
```

The `--release` flag can also be omitted if you prefer faster compilation over faster execution. However, once the
prover has been build by cargo, and provided sources have not changed, the cargo command will quickly proceed.
Note that the Rust code of the prover (and the underlying Move compiler) is by about a factor of 20 faster in
release mode than in debug mode.

In the sequel, we assume that `mvp` invokes the binary of the prover, either via an alias as above, or by
other means.

## Command Line Interface and Configuration

The Move prover has a traditional compiler-style command line interface: you pass a set of sources, tell it where to
look for dependencies of those sources, and optionally provide flags to control operation:

```shell script
> mvp --dependency . source.move
> # Short form:
> mvp -d . source.move
```

Above, we process a file `source.move` in the current directory, and tell the prover to look up any dependencies this source
might have in the current directory (`.`). If verification succeeds, the prover will terminate with printing
some statistics dependent on the configured verbosity level. Otherwise, it will print diagnosis, as will be
discussed below.

## Configuration File

All options available via the command line, plus some more, can be also configured via a file. Moreover, you can
set an environment variable which contains a path to the default configuration file the prover should use. This is handy,
for example, to let the prover automatically find dependencies to the Move standard library, as shown below:

```shell script
> echo "move_deps = [\"<path-to-libra>/language/stdlib/modules\"]" > ~/.mvprc
> export MOVE_PROVER_CONFIG=~/.mvprc
```

To see the full format of the configuration file use `mvp --print-config`. This will print out the toml for
all available options. You can use this output as a blueprint for creating your own configuration
file.

## Diagnosis

When the prover finds a verification error it prints out diagnosis in a style similar to a compiler or a debugger. We
explain the different types of diagnoses below, based on the following evolving example:

```move
module M {
  resource struct Counter {
      value: u8,
  }

  public fun increment(a: address) acquires Counter {
      let r = borrow_global_mut<Counter>(a);
      r.value = r.value + 1;
  }

  spec fun increment {
      aborts_if aborts_if !exists<Counter>(a);
      ensures global<Counter>(a).value == old(global<Counter>(a)).value + 1;
  }
}
```

We will modify this example as we demonstrate different types of diagnoses.

### Unexpected Abort

If we run the Move prover on the above example, we get the following error:

```
error: abort not covered by any of the `aborts_if` clauses

   ┌── tutorial.move:6:3 ───
   │
 6 │ ╭   public fun increment(a: address) acquires Counter {
 7 │ │       let r = borrow_global_mut<Counter>(a);
 8 │ │       r.value = r.value + 1;
 9 │ │   }
   │ ╰───^
   ·
 8 │       r.value = r.value + 1;
   │                         - abort happened here
   │
   =     at tutorial.move:6:3: increment (entry)
   =     at tutorial.move:7:15: increment
   =         a = 0x5,
   =         r = &M.Counter{value = 255u8}
   =     at tutorial.move:8:17: increment (ABORTED)
```

The prover has generated a counter example which leads to an overflow when adding 1 the value of 255 for an `u8`.
This happens if the function specification states something abort abort behavior, but the condition under which
the function is aborting is not covered by the specification. And in fact, with `aborts_if !exists<Counter>(a)` we
only cover the abort if the resource does not exists, but not the overflow.

Let's fix the above and add the following condition:

```move
spec fun increment {
    aborts_if global<Counter>(a).value == 255;
}
```

With this, the prover will succeed without any errors.

### Postcondition might not hold

Let us inject an error into the `ensures` condition of the above example:

```move
spec increment {
    ensures global<Counter>(a).value == /*old*/(global<Counter>(a).value) + 1;
}
```

With this, the prover will produce the following diagnosis:

```
error:  A postcondition might not hold on this return path.

    ┌── tutorial.move:14:7 ───
    │
 14 │       ensures global<Counter>(a).value == global<Counter>(a).value + 1;
    │       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    │
    =     at tutorial.move:6:3: increment (entry)
    =     at tutorial.move:7:15: increment
    =         a = 0x5,
    =         r = &M.Counter{value = 50u8}
    =     at tutorial.move:8:17: increment
    =         r = &M.Counter{value = 50u8}
    =     at tutorial.move:6:3: increment
    =     at tutorial.move:6:3: increment (exit)
```

While we know what the error is (we just injected it), looking at the printed information makes it not particular
obvious. This is because we don't directly see on which values the `ensures` condition was actually evaluated. To
inspect those, we have two options. First, we can run the prover with the `--trace` option. The output then
looks as below:

```
error:  A postcondition might not hold on this return path.

    ┌── tutorial.move:14:7 ───
    │
 14 │       ensures global<Counter>(a).value == global<Counter>(a).value + 1;
    │       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    ·
 12 │       aborts_if !exists<Counter>(a);
    │                                  - 0x5
    ·
 12 │       aborts_if !exists<Counter>(a);
    │                  ------------------ true
    ·
 14 │       ensures global<Counter>(a).value == global<Counter>(a).value + 1;
    │               ------------------ M.Counter{value = 244u8}
    ·
 14 │       ensures global<Counter>(a).value == global<Counter>(a).value + 1;
    │                                           ------------------ M.Counter{value = 244u8}
    │
    =     at tutorial.move:6:3: increment (entry)
    =     at tutorial.move:7:15: increment
    =         a = 0x5,
    =         r = &M.Counter{value = 243u8}
    =     at tutorial.move:8:17: increment
    =         r = &M.Counter{value = 243u8}
    =     at tutorial.move:6:3: increment
    =     at tutorial.move:6:3: increment (exit)
```

> Note: the `--trace` option is currently known to sometimes produce false positives.

Instead of the `--trace` option, one can also use the builtin function `TRACE(exp)` in conditions to explicitly
mark expressions whose value should be printed on verification failures.

## Debugging

The Move prover is still an evolving tool with bugs and deficiencies. Sometimes it might be necessary to debug
a problem based on the output it passes to the underlying backends. There are two options to this end:

- By default, the prover will place the generated Boogie code in a file `output.bpl`, and the errors Boogie reported
  in a file `output.bpl.log`.
- With the option `-C backend.generate_smt=true` the prover will generate, for each verification problem, a file in
  the smtlib format. The file is named after the verified function. This file contains the output Boogie
  passes on to Z3 or other connected SMT solvers.
