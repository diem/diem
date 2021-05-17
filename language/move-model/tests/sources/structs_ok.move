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

  public fun f(r: R): T {
    T { x: r.s.x }
  }

  spec module {

    fun struct_access(s: S): u64 {
      s.x
    }

    fun nested_struct_access(r: R): bool {
      r.s.y
    }

    fun struct_pack(x: u64, y: bool, z: vector<u64>): S {
      S{x: x, y: y, z: z}
    }

    fun struct_pack_other_order(x: u64, y: bool, z: vector<u64>): S {
      S{z: z, y: y, x: x}
    }

    fun generic_struct_pack(x: u64, y: bool): G<u64> {
      G{x: x, y: y}
    }

    fun generic_struct_pack_instantiated(x: u64, y: bool): G<u64> {
      G<u64>{x: x, y: y}
    }

    fun resource_global(addr: address): T {
      global<T>(addr)
    }

    fun resource_global_exists(addr: address): bool {
      exists<T>(addr)
    }

  }
}
