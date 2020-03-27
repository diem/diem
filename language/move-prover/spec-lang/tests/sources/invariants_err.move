module M {

  struct S {
    x: u64,
  }

  spec struct S {
    // Expression not a bool
    invariant x + 1;
    // Old expression in data invariant
    invariant old(x) > 0;
    // Nested old expression.
    invariant update old(old(x)) > 0;
  }
}
