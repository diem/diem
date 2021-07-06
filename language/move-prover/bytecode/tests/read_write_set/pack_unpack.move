module 0x1::PackUnpack {

    struct S has drop { a1: address, t: T, i: u64 }
    struct T has drop { a2: address, b: bool }
    struct Glob has key { b: bool }

    fun read_unpacked_addr(s: S): address {
        let S { a1, t: T { a2, b }, i: _ } = s;
        if (b) { a1 } else { a2 }
    } // ret |-> { Formal(0)/a1, Formal(0)/t/a2 }

    fun pack_then_read(a2: address): address {
        let t = T { a2, b: false };
        *&t.a2
    } // ret |-> Formal(0)

    fun read_unpacked_borrow_glob(s: S): bool acquires Glob {
        let S { a1, t: _, i: _ } = s;
        *&borrow_global<Glob>(a1).b
    } // ret |-> Formal(0)/a1/Glob/b

    fun read_packed_borrow_glob(a2: address): bool acquires Glob {
        let t = T { a2, b: false };
        *&borrow_global<Glob>(*&t.a2).b
    } // ret |-> Formal(0)/Glob/b

    fun reassign_packed_addr(a2: address): address {
        let t = T { a2: @0x7, b: false };
        _ = t;
        t = T { a2, b: false };
        *&t.a2
    } // ret |-> Formal(0)

    fun use_results(_: u64, a: address): address {
        let a1 = pack_then_read(a);
        let s = S { a1, t: T { a2: @0x7, b: true }, i: 10 };
        read_unpacked_addr(s)
    }  // ret |-> { Formal(1), @0x7 }
}
