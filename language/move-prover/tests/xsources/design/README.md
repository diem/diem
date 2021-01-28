A set of examples for illustrating how the bytecode transformation pipeline works.

Run as in:

```
mvp -k --v2 --dump-bytecode <example>.move
```

The current directory will contain bytecode dumps for each phase in `<example>_*.bytecode`,
as well as `output.bpl` with the Boogie code.
