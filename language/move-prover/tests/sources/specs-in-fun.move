module TestAssertAndAssume {
    // succeeds
    fun simple1(x: u64, y: u64) {
        if (!(x > y)) abort 1;
        spec {
            assert x > y;
        }
    }

    // fails
    fun simple2(x: u64, y: u64) {
        if (!(x > y)) abort 1;
        spec {
            assert x == y;
        }
    }

    // succeeds
    fun simple3(x: u64, y: u64) {
        spec {
            assume x > y;
            assert x >= y;
        }
    }

    // fails
    fun simple4(x: u64, y: u64) {
        spec {
            assume x >= y;
            assert x > y;
        }
    }
}
