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

    define struct_access(s: S): u64 {
      s.x
    }

    define nested_struct_access(r: R): bool {
      r.s.y
    }

    define struct_pack(x: u64, y: bool, z: vector<u64>): S {
      S{x: x, y: y, z: z}
    }

    define generic_struct_pack(x: u64, y: bool): G<u64> {
      G{x: x, y: y}
    }

    define generic_struct_pack_instantiated(x: u64, y: bool): G<u64> {
      G<u64>{x: x, y: y}
    }

    define resource_global(addr: address): T {
      global<T>(addr)
    }

    define resource_global_exists(addr: address): bool {
      exists<T>(addr)
    }

  }
}
