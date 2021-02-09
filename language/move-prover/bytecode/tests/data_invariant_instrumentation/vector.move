module Test {

    struct R {
        x: u64,
        s: vector<vector<S>>,
    }

    struct S {
        y: u64
    }

    spec struct S {
        invariant y > 0;
    }

    public fun test(_r: R) {
    }
}
