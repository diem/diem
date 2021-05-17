module 0x42::Test {
    struct R {
        x: u64,
        s: S,
    }

    struct S {
        y: u64
    }

    spec R {
        invariant x > s.y;
    }

    spec S {
        invariant y > 0;
    }

    public fun test_pack() : R {
        let s = S {y: 1};
        let r = R {x: 3, s: s};
        r
    }
}
