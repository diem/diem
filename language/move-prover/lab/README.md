This crate contains tools and data for analyzing the performance of the Move Prover.

The `data` directory contains a number of lab sessions with scripts and persisted data. Each of those directories
has its own README. The `src` directory contains supporting Rust code.

Most labs are currently based on Jupyter notebooks with a Rust based kernel. The installation of Jupyter
can be done via the python installation tool:

```
pip install jupyterlab
pip install notebook
```

For the Rust kernel, follow the installation instructions at [google/evcxr](https://github.
com/google/evcxr/blob/master/evcxr_jupyter/README.md). You need version 0.9.0 or later.


# Caveats

EVCXR has some flaws at this point (or we are not using it correctly):

- Evaluating the first `:dep` after start of a notebook can take a really long time (minutes) since EVCXR compiles
  all of Rust. Running `cargo install sccache` may help.
- After the first evaluation, the raw output of EVCXR is often displayed. Evaluating the cell a second time fixes
  this. Unfortunately, this renders `jupyter nbconvert --execute` unusable to create reports. The only way to do
  so is to manually ensure everything is evaluated correctly, on then use the Web UIs "Print" function ("Export"
  wont work because it seems to be based on `nbconvert`).
