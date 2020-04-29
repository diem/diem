address 0x42 {

module A {
    resource struct Coin { u: u64 }

    public fun new(): Coin {
        Coin { u: 1 }
    }

    public fun join(c1: Coin, c2: Coin): Coin {
        let Coin { u: u1 } = c1;
        let Coin { u: u2 } = c2;
        Coin { u: u1 + u2 }
    }

    public fun split(c1: Coin, amt: u64): (Coin, Coin) {
        let Coin { u } = c1;
        0x0::Transaction::assert(u >= amt, 42);
        (Coin { u: u - amt }, Coin { u: amt })
    }
}

module Tester {
    use 0x42::A;

    resource struct Pair { x: A::Coin, y: A::Coin }

    fun test_eq(addr1: address, addr2: address): bool acquires Pair {
        let p1 = borrow_global<Pair>(addr1);
        // It is lossy in the sense that this could be acceptable if we had 'acquires imm Pair' or
        // something to indicate the "acquires" are immutable
        eq_helper(p1, addr2)
    }

    fun eq_helper(p1: &Pair, addr2: address): bool acquires Pair {
        let p2 = borrow_global<Pair>(addr2);
        p1 == p2
    }

}

// check: GLOBAL_REFERENCE_ERROR

}
