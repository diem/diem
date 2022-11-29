# Benchmarking multiple vector theories

This lab compares the following vector theories (sources in ./boogie-backend/prelude):

- BoogieArray: this is currently the default vector theory used in the Move Prover. It is based on Boogie Arrays (in contrast to native SMT arrays) and does not support extensional equality.
- BoogieArrayIntern: this is a boogie array theory which uses an internalization of representation to achieve extensionality.
- SmtArray: this is a vector theory using SMT native arrays, without support for extensional equality.
- SmtArrayExt: this is a vector theory using SMT native arrays, with added axioms to ensure extensional equality.
- SmtSeq: this is a vector theory based on SMT sequences.

## Module Verification Time

![Module-By-Module](mod_by_mod.svg)

## Function Verification Time

![Function-By-Function](fun_by_fun.svg)
