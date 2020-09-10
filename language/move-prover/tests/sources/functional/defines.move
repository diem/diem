module TestDefines {

    spec module {
        pragma verify = true;
    }

    spec module {
        define in_range(x: num, min: num, max: num): bool {
            x >= min && x <= max
        }

        define eq<T>(x: T, y: T): bool {
            x == y
        }
    }

    fun add(x: u64, y: u64): u64 { x + y }

    spec fun add {
        aborts_if !in_range(x + y, 0, 18446744073709551615);
        ensures eq(result, x + y);
    }

    // --------------------------------------
    // Spec functions accessing global state.
    // --------------------------------------

    resource struct R { x: u64 }

    spec module {
        define exists_both(addr1: address, addr2: address): bool {
            exists<R>(addr1) && exists<R>(addr2)
        }
        define get(addr: address): u64 {
            global<R>(addr).x
        }
    }

    fun equal_R(addr1: address, addr2: address): bool acquires R {
        let r1 = borrow_global<R>(addr1);
        let r2 = borrow_global<R>(addr2);
        r1.x == r2.x
    }
    spec fun equal_R {
        aborts_if !exists_both(addr1, addr2);
        ensures result == (get(addr1) == get(addr2));
    }
}
