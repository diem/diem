module 0x42::TestDefines {

    spec module {
        pragma verify = true;
    }

    spec module {
        fun in_range(x: num, min: num, max: num): bool {
            x >= min && x <= max
        }

        fun eq<T>(x: T, y: T): bool {
            x == y
        }
    }

    fun add(x: u64, y: u64): u64 { x + y }

    spec add {
        aborts_if !in_range(x + y, 0, 18446744073709551615);
        ensures eq(result, x + y);
    }

    // --------------------------------------
    // Spec functions accessing global state.
    // --------------------------------------

    struct R has key { x: u64 }

    spec module {
        fun exists_both(addr1: address, addr2: address): bool {
            exists<R>(addr1) && exists<R>(addr2)
        }
        fun get(addr: address): u64 {
            global<R>(addr).x
        }
    }

    fun equal_R(addr1: address, addr2: address): bool acquires R {
        let r1 = borrow_global<R>(addr1);
        let r2 = borrow_global<R>(addr2);
        r1.x == r2.x
    }
    spec equal_R {
        aborts_if !exists_both(addr1, addr2);
        ensures result == (get(addr1) == get(addr2));
    }

    // ------------------------------------------
    // Spec functions derived from Move functions
    // ------------------------------------------

    fun add_as_spec_fun(x: u64, y: u64): u64 { x + y }
    fun add_fun(x: u64, y: u64): u64 { x + y }
    spec add_fun {
        include AddOk;
    }
    spec schema AddOk {
        x: num; y: num; result: num;
        ensures result == add_as_spec_fun(x, y);
    }
}
