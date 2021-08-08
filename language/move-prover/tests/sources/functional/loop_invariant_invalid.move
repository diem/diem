module 0x42::InvalidLoopInvariants {
    public fun invariant_declared_in_body() {
        let i = 0;
        while (i <= 10) {
            spec {
                invariant i <= 10;
            };
            if (i > 10) abort 10;
            i = i + 1;
        }
    }

    public fun non_consecutive_invariant_decl() {
        let i = 0;
        let j = 42;
        let k = j;
        while ({
            spec {
                invariant i <= 11;
                assert j == 42;
                invariant k == j;
            };
            (i <= 10)
        }) {
            if (i > 10) abort 10;
            i = i + 1;
        }
    }
}
