module M {

  struct S {
    x: u64,
  }

  spec module {
    define exists_in_vector(v: vector<S>): bool {
      exists s in v: s.x > 0
    }
  }
}
