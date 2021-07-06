module 0x1::SimplePackUnpack {
    struct S has drop { a1: address }

    fun unpack(s: S): address {
        s.a1
    } // ret |-> { Formal(0)/a1 }

    fun pack_unpack(a: address): address {
        let s = S { a1: a };
        unpack(s)
    }  // ret |-> { Formal(0) }
}
