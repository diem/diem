module 0x42::Test {

    struct R has drop {
        x: u64,
        s: vector<vector<S>>,
    }

    struct S has drop {
        y: u64
    }

    spec S {
        invariant y > 0;
    }

    public fun test(_r: R) {
    }
}
