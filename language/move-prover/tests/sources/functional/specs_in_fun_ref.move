module 0x42::TestAssertWithReferences {

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

    // This function verifies.
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
                invariant x <= n;
            };
            if (!(x < n)) break;
            x = x + 1;
        };
        spec {
            assert x == n;
        };
        x
    }
    spec simple5 {
        ensures result == n;
    }

    // This function verifies.
    fun simple6(n: u64): u64 {
        let x = 0;

        while ({
            spec {
                invariant x <= n;
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
    spec simple6 {
        ensures result == n;
    }

    // This function verifies.
    fun simple7(x: &mut u64, y: u64): u64 {
        let a = x;
        let b = y;
        let c = &mut b;
        *c = *a;
        *a = y;
        spec {
            assert a == y;
            assert x == y;
            assert c == old(x);
            // NOTE: cannot verify the following, write-back not propagated yet
            // assert b == old(x);
        };
        let _ = c;
        b
    }
    spec simple7 {
        ensures x == y && result == old(x);
    }
}
