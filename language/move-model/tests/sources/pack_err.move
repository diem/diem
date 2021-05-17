module 0x42::M {

  struct S has copy, drop {
    x: u64,
  }

  spec S {
    global sum: num;
    invariant pack sum = sum + x;
    invariant unpack sum = sum - x;
  }
}
