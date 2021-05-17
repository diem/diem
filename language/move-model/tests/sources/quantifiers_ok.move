module 0x42::M {

  struct S {
    x: u64,
  }

  spec module {
    fun exists_in_vector(v: vector<S>): bool {
      exists s in v: s.x > 0
    }

    fun some_in_vector(v: vector<S>): S {
        choose s in v where s.x == 0
    }
  }
}
