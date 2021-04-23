This lab is used to compare the current version of Boogie with an alternative, experimental one.
To run this lab, one must have set the variable `EXP_BOOGIE_EXE` to the alternative boogie
binary.

To view the results of the current benchmark data, use:

```
./notebook.sh
```

To regenerate the benchmark data, use:

```
./run.sh
```

You can keep the notebook open in the browser when you regenerate for experimentation.
Simply re-evaluate all cells from the point where the data is read. This will be much faster as
starting the notebook again after regeneration.
