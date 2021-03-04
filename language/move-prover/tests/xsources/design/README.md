A set of examples for illustrating how the bytecode transformation pipeline works.

This directory contains the bytecode dump of each of the Move
examples. You can run an individual example as in:

```
mvp -k --v2 --dump-bytecode <example>.move
```

The `-k` option will also let the Move prover leave the generated Boogie output
in `output.bpl`.
