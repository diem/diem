module M {

  struct S {
    x: u64,
    y: bool,
    z: vector<u64>,
  }

  struct R {
    s: S
  }

  struct G<T> {
    x: T,
    y: bool,
  }

  resource struct T {
    x: u64
  }

  spec module {

    // Field undeclared
    define field_undeclared(s: S): u64 {
      s.xx
    }

    // Field wrongly typed
    define field_wrongly_typed(s: S): bool {
      s.x
    }

    // Pack wrongly typed
    define pack_wrongly_typed(x: u64, y: u64, z: vector<u64>): S {
      S{x: x, y: y, z: z}
    }

    define pack_misses_fields(x: u64): S {
      S{x: x}
    }

    // Generic pack wrongly typed
    define generic_pack_wrongly_typed(x: u64, y: bool): G<bool> {
      G{x: x, y: y}
    }

    // Wrong number of generics
    define wrong_number_generics(x: u64, y: bool): G<u64> {
      G<u64, bool>{x: x, y: y}
    }
  }
}
