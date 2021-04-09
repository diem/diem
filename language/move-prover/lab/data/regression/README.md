This lab is used to evaluate performance regression based on a previously recorded benchmark.

Use as in:

```
cd lab/data/regression
./run.sh
./notebook.sh
```

In the notebook, evaluate all cells sequentially to see a comparison of the previous performance with the current one.

In order to make the last result of `run.sh` baseline for future comparison, rename the data file `default.mod_data`
to `default.last.mod_data`. Do *not* commit `default.mod_data`.
