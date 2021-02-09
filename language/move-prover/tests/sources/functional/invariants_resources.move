module TestInvariants {

    spec module {
        pragma verify = true;
    }

    resource struct R<T> {
        x: u64,
        t: T
    }

    spec struct R {
        invariant greater_one(x);
    }

    spec define greater_one(x: num): bool { x > 1 }

    // Tests whether the invariant of resources in memory holds.
    public fun get<T>(a: address): u64 acquires R {
        borrow_global<R<T>>(a).x
    }
    spec fun get {
        ensures result > 0;
    }

    // Negative test of the above.
    public fun get_invalid<T>(a: address): u64 acquires R {
        borrow_global<R<T>>(a).x
    }
    spec fun get_invalid {
        // TODO(refactoring): This test seems to sometime succeed not producing an error when run
        //   via `cargo test`. However, we cannot reproduce this on the command line, or with
        //   `check_stability.sh <source> 100`. It might be an issue with the test driver or a crash
        //   downstream in z3 which is not reported back in tests
        pragma verify = false;
        ensures result < 1;
    }
}
