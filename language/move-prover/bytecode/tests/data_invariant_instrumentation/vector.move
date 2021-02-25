module Test {

    struct R has drop {
        x: u64,
        s: vector<vector<S>>,
    }

    struct S has drop {
        y: u64
    }

    spec struct S {
        invariant y > 0;
    }

    public fun test(_r: R) {
    }
}
