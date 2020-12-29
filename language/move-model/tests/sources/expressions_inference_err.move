// Inference errors may only be reported if all else succeeds, so we put them in a different file.

module M {
  spec module {
    // Incomplete types.
    define incomplete_types(): u64 {
      let f = |x|x;
      0
    }
  }
}
