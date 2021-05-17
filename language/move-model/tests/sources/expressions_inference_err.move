// Inference errors may only be reported if all else succeeds, so we put them in a different file.

module 0x42::M {
  spec module {
    // Incomplete types.
    fun incomplete_types(): u64 {
      let f = |x|x;
      0
    }
  }
}
