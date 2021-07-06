module 0x1::SecondaryIndex {
  struct A has key, drop { a_addr: address }
  struct B has key, drop { b_addr: address }
  struct S { f: address, g: address }
  struct C { b : B }

  fun read_secondary_index_from_formal(a: &A): address acquires B {
      borrow_global<B>(a.a_addr).b_addr
  }

  fun read_secondary_index_from_global(a: address): address acquires A, B {
      let addr = borrow_global<A>(a).a_addr;
      borrow_global<B>(addr).b_addr
  }

  fun read_secondary_index_from_formal_interproc(
      a_addr: address
  ): address acquires B {
      let a = A { a_addr };
      read_secondary_index_from_formal(&a)
  }

  fun read_secondary_index_from_global_interproc(
      a: address
  ): address acquires A,B {
      read_secondary_index_from_formal(borrow_global<A>(a))
  }

  fun two_secondary_indexes(s: &S, b: bool): address acquires A {
    if (b) {
      borrow_global<A>(s.f).a_addr
    } else {
      borrow_global<A>(s.g).a_addr
    }
  }

  fun call_two_secondary_indexes(s: &S): address acquires A {
    two_secondary_indexes(s, true)
  }

  fun write_both_fields(c: &mut C, b: B, addr: address, flag: bool) {
    if (flag) {
      c.b = b;
    };
    if (flag) {
      c.b.b_addr = addr;
    }
  }
}
