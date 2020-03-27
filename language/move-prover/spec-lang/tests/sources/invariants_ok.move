module M {

  struct S {
    x: u64,
    y: bool
  }

  struct R {
    s: S,
  }

  spec struct S {
    invariant x > 0 == y;
    invariant update old(x) < x;
  }

  spec struct R {
    invariant s.x < 10;
  }
}
