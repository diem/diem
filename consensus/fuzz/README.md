# Fuzzing Consensus

## Abstract

Fuzzing is good.

## Fuzzing

Everything should be ready for you to fuzz. There are three fuzzers out there, fuzzing:

* a consensus proposal message
* a consensus vote message
* a consensus timeout message

You can run them with the following command:

```
cargo fuzz run fuzz_consensus_vote -- fuzz/corpus/vote
cargo fuzz run fuzz_consensus_proposal -- fuzz/corpus/proposal
cargo fuzz run fuzz_consensus_timeout -- fuzz/corpus/timeout
```

## Generating Corpus

To generate the corpus, run the following test:

```
cargo test -p consensus fuzzing_corpus -- --ignore
```

