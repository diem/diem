module 0x42::M {

  fun foo(x: &mut u64): u64 { *x = *x + 1; *x }

  spec foo {
    let zero = 0;
    let one = zero + 1;
    ensures result == old(x) + one;
  }
}
