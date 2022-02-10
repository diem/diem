
<a name="@Diem_Framework_Specification_0"></a>

# Diem Framework Specification


[FORM_VER]: https://en.wikipedia.org/wiki/Formal_verification
[SMT]: https://en.wikipedia.org/wiki/Satisfiability_modulo_theories
[DESIGN_BY_CONTRACT]: https://en.wikipedia.org/wiki/Design_by_contract
[MSL]: https://github.com/diem/diem/blob/main/language/move-prover/doc/user/move-model.md
[PROVER]: https://github.com/diem/diem/blob/main/language/move-prover/doc/user/prover-guide.md
[BOOGIE]: https://github.com/boogie-org/boogie
[Z3]: https://github.com/Z3Prover/z3
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md

The Diem framework comes with an exhaustive formal specification of modules and transaction scripts.
The specifications are formally verified against the Move implementation, with the verification enabled in
continuous integration, and successful verification a land blocker for each PR merged into the Diem project.
Here we given an overview of the specification approach and what it provides for the framework.


<a name="@Brief_Introduction_to_Formal_Verification_1"></a>

## Brief Introduction to Formal Verification


[Formal verification][FORM_VER] is an approach to software quality assurance
which has been around since many decades. The basic idea is that properties of the software are formulated in a
*specification language*, and are verified against the software using techniques like symbolic reasoning and theorem
proving. In contrast to traditional testing, verification is exhaustive and holds for *all* possible inputs and
states of a program. While testing can only prove the presence of errors, but not their absence, verification
has the capability to give complete insights about absence of errors and correctness of software w.r.t. the
specification.

Formal verification is challenging for general software because of the complexity of system programming languages and,
most importantly, the large corpus of existing and unspecified software and abstractions on top of which typical systems
are build. In addition, some approaches to verification are not fully automatic and require human interaction from
highly skilled experts.

Smart contracts as they are written in Move avoid some of these problems. First, the Move language has a small
and well-defined semantics which is suitable for verification. Second, the Move runtime environment is fully
isolated and sand-boxed, by construction preventing Move programs to call into other, unspecified software.
Last not least, techniques for formal verification, like [SMT solving][SMT]
made continuous advances over the last decade, and provide fully automated solutions for verification.


<a name="@How_the_Diem_Framework_is_Specified_2"></a>

## How the Diem Framework is Specified


The Diem framework uses the [Move Specification Language][MSL] for specification of properties. The language
is designed in the tradition of [Design by Contract][DESIGN_BY_CONTRACT]. It uses pre- and post-conditions
to define behavior of functions, and invariants over data structures and global resource state. Conditions
are described by predicates which involve access to function parameters, structured data, and global resource state.
The specification language is fully embedded into the Move language, and re-uses syntax and semantics of Move where
appropriate. It is expressive enough to capture the full semantics of Move (with minor exceptions).

Specification of complex functions using pre/post conditions is not trivial, and some of the specifications may
easily exceed in size the actual implementation in Move. This should not surprise, as specifications require to
make many of the "implicit" behavior of code explicit. For example, if a Move function calls another function which
aborts, propagation of this abort happens implicitly. Not so in the specification, where each abort condition needs
to be explicitly accounted for at each function. The Move specification language gives means to abstract some of this
to avoid repetition (namely, reusable specification schemas), yet still specifications can be verbose. However, writing
tests which provide 100% coverage for each relevant input and state combination is arguably significantly more verbose.


<a name="@How_the_Diem_Framework_is_Verified_3"></a>

## How the Diem Framework is Verified


Move specifications are verified by the [Move Prover][PROVER]. This is a tool which works from the generated
Move byte code and combines it with the specifications to create a verification condition which is then passed
on to off-the-shelf standard verification tools (currently [Boogie][Boogie] and [Z3][Z3]). The diagnosis created
by those tools is then translated back to provide feedback on the level of Move, creating error messages
very much similar as a type checker or linter tool. No human interaction is required for this.

Verification of the Diem framework is fully embedded into the developer workflow. A Rust integration test
calls the Move prover on each Move source in the framework, failing the test if Move verification fails. Failures
in this process are land blockers for submitting Move code.


<a name="@Completeness_of_Specification_and_Verification_4"></a>

## Completeness of Specification and Verification


At this point, the Diem framework is specified to the following extent:

- Each transaction script is specified.
- Most Module functions called directly or indirectly via a transaction script are specified. Note that
some Module code which is not called this way may not yet be fully specified. Also some functions might
not be individually specified, but still verified in the context they are used from other functions.
- A crosscut regards *access control* as defined by [DIP-2][ACCESS_CONTROL] has been systematically specified.
- Some aspects of the framework have been abstracted out in the current release and are not verified; most
notably, Event generation has not been specified and verified.


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
