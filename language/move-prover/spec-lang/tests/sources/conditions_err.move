module M {

  fun add_some(x: &mut u64): u64 { *x = *x + 1; *x }

  spec fun add_some {
    // Type of condition not bool.
    aborts_if x;
    ensures old(x) + x;
    // Using result which does not exist.
    ensures result_1 == 0;
  }
}
