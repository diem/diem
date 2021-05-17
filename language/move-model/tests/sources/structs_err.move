module 0x42::M {

  struct S has drop {
    x: u64,
    y: bool,
    z: vector<u64>,
  }

  struct R has drop {
    s: S
  }

  struct G<T> {
    x: T,
    y: bool,
  }

  struct T has key {
    x: u64
  }

  spec module {

    // Field undeclared
    fun field_undeclared(s: S): u64 {
      s.xx
    }

    // Field wrongly typed
    fun field_wrongly_typed(s: S): bool {
      s.x
    }

    // Pack wrongly typed
    fun pack_wrongly_typed(x: u64, y: u64, z: vector<u64>): S {
      S{x: x, y: y, z: z}
    }

    fun pack_misses_fields(x: u64): S {
      S{x: x}
    }

    // Generic pack wrongly typed
    fun generic_pack_wrongly_typed(x: u64, y: bool): G<bool> {
      G{x: x, y: y}
    }

    // Wrong number of generics
    fun wrong_number_generics(x: u64, y: bool): G<u64> {
      G<u64, bool>{x: x, y: y}
    }
  }
}
