This module handles building a global environment for a set of Move modules merging multiple sources:
the bytecode as produced by the Move compiler, source mapping information to map the bytecode back
to the Move source, and the AST for specification constructs (derived from the internal AST
of the Move compiler after its expansion phase). In order to create the AST for specs, type checking
of the specification fragments is handled here as well. This might be refactored in the future when
we move more of the Move specification language fragment into the Move compiler.
