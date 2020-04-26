module TestAssertAndAssume {
    // Tests that should verify

    fun simple1(x: u64, y: u64) {
        if (!(x > y)) abort 1;
        spec {
            assert x > y;
        }
    }

    fun simple2(x: u64) {
        let y: u64;
        y = x + 1;
        spec {
            assert x == y - 1;
        }
    }

    fun simple3(x: u64, y: u64) {
        spec {
            assume x > y;
            assert x >= y;
        }
    }

    fun simple4(x: u64, y: u64) {
        let z: u64;
        z = x + y;
        spec {
            assume x > y;
            assert z > 2*y;
        }
    }

    // Tests that should not verify

    fun simple1_incorrect(x: u64, y: u64) {
        if (!(x > y)) abort 1;
        spec {
            assert x == y;
        }
    }

    fun simple2_incorrect(x: u64) {
        let y: u64;
        y = x + 1;
        spec {
            assert x == y;
        }
    }

    fun simple3_incorrect(x: u64, y: u64) {
        spec {
            assume x >= y;
            assert x > y;
        }
    }

    fun simple4_incorrect(x: u64, y: u64) {
        let z: u64;
        z = x + y;
        spec {
            assume x > y;
            assert z > 2*x;
        }
    }
}
