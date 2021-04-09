This notebook is used to analyze quantifier instantiations, for example for resolving
timeout problems. See general info for working with Jupyter in the
[lab/README.md](../../README.md).

To use this notebook, generate a z3 log file by calling the Move prover as follows:

```
cargo run -p move-prover --z3-trace=Module::function source.move
```

This will create a file `function.z3log` in the current directory.
You can move this file to the well known location `/tmp/mvp.z3log` which is
used by default in the notebook. Alternatively, you can override the definition of the
file location in the notebook (but please do not submit).

To start the notebook, run:

```
./notebook.sh
```

In the notebook, evaluate all cells sequentially.

You can regenerate the data at `/tmp/mvp.z3log` at any time while the notebook
is open. To see the changes in the open notebook, re-evaluate cells from the
point where the z3tracer model is loaded.
