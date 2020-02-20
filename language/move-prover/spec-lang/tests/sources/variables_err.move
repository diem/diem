module M {

  struct S {
    x: u64,
  }

  spec struct S {
    global tick: num;

    // Undeclared
    invariant update ticks = tick + 1;

    // Wrong type
    invariant update tick = false;

    // Cannot have assignment
    invariant tick = x;

    // Must have assignment
    invariant pack x == 0;
    invariant unpack x == 0;
  }
}
