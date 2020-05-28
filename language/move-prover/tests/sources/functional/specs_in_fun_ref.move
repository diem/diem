module TestAssertWithReferences {
    spec module {
        pragma verify = true;
    }

    // This function verifies.
    // Feature: An input parameter is mutated
    fun simple1(x: u64, y: u64) {
        let z;
        y = x;
        z = x + y;
        spec {
            assert x == y;
            assert z == 2*x;
            assert z == 2*y;
        }
    }

    // This function verifies.
    // Feature: An input reference parameter is mutated
    fun simple2(x: &mut u64, y: &mut u64) {
        *y = *x;
        spec {
            assert x == y;
        };
    }

    // This function fails.
    // Feature: Updates to y span the spec block.  The partial update before
    // the spec block is written back only when y dies at the end of the function.
    // Hence, the first assert fails but the second verifies.
    fun simple3(x: &mut u64, y: &mut u64) {
        *y = *x;
        spec {
            assert x == y;
        };
        *y = *y + 1;
        spec {
            assert x + 1 == y;
        }
    }

    // This function verifies.
    // Feature: Updates to y span the spec block.  To work around the problem of
    // delayed writeback, either read y or freeze y and use the resulting value
    // in the spec block.
    fun simple4(x: &mut u64, y: &mut u64) {
        let z;
        *y = *x;
        z = *x + *y;
        let vx = *x;
        let vy = *y;
        let fx = freeze(x);
        let fy = freeze(y);
        spec {
            assert fx == fy;
            assert vx == vy;
            assert z == 2*x;
        };
        _ = fy; // release fy
        *y = *y + 1;
        spec {
            assert x + 1 == y;
            assert z + 1 == x + y;
        }
    }

    // This function verifies.
    fun simple5(n: u64): u64 {
        let x = 0;

        loop {
            spec {
                assert x <= n;
            };
            if (!(x < n)) break;
            x = x + 1;
        };
        spec {
            assert x == n;
        };
        x
    }
    spec fun simple5 {
        ensures result == n;
    }

    // This function verifies.
    fun simple6(n: u64): u64 {
        let x = 0;

        while ({
            spec {
                assert x <= n;
            };
            (x < n)
        }) {
            x = x + 1;
        };
        spec {
            assert x == n;
        };
        x
    }
    spec fun simple6 {
        ensures result == n;
    }

    // This function fails.
    fun simple7(n: u64): u64 {
        let x = 0;

        while ({
            spec {
                assert x == 0;
            };
            (x < n)
        }) {
            x = x + 1;
        };
        x
    }
}
