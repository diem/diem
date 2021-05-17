// separate_baseline: cvc4
// TODO(cvc4): cvc4 generates a false positive.
module 0x42::TestInvariants {
    spec module {
        pragma verify = true;
    }
    struct R<T> has key {
        x: u64,
        t: T
    }

    spec R {
        invariant greater_one(x);
    }

    spec fun greater_one(x: num): bool { x > 1 }

    // Tests whether the invariant of resources in memory holds.
    public fun get<T: store>(a: address): u64 acquires R {
        borrow_global<R<T>>(a).x
    }
    spec get {
        ensures result > 0;
    }

    // Negative test of the above.
    public fun get_invalid<T: store>(a: address): u64 acquires R {
        borrow_global<R<T>>(a).x
    }
    spec get_invalid {
        ensures result < 1;
    }
}
