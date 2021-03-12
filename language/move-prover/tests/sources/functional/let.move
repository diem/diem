module 0x42::TestLet {

    spec module {
        pragma verify = true;
    }

    struct R has key {
        x: u64
    }

    // Spec Block Let
    // ==============

    fun spec_let(a: u64, b: u64): (u64, u64) {
        (a + 1 + b, a + b)
    }
    spec fun spec_let {
        let x = a + 1;
        let y = x + b;
        ensures result_1 == y;
        ensures result_2 == result_1 - 1;
    }

    fun spec_let_with_old(a: &mut u64, b: &mut u64) {
        let saved_a = *a;
        *a = *a + 1 + *b;
        *b = saved_a + *b;
    }
    spec fun spec_let_with_old {
       let x = a + 1;
       let y = x + b;
       let r2 = a - 1;
       ensures a == old(y); // y in the context of old accesses old a and b.
       ensures b == r2;
    }

    fun spec_let_with_lambda(a: u64, b: u64): (u64, u64) {
        (a + 1 + b, a + b)
    }
    spec fun spec_let_with_lambda {
        let add_to_a = |n| a + n;
        let add_to_b = |n| b + n;
        ensures result_1 == add_to_b(add_to_a(1));
        ensures result_2 == result_1 - 1;
    }

    fun spec_let_with_generic<T:copy + drop>(x: T, y: T): bool {
        x == y
    }
    spec fun spec_let_with_generic {
        let equals_to_x = |z| x == z;
        ensures result == equals_to_x(y);
    }


    // Local Let
    // =========

    fun local_let_with_memory_access(a1: address, a2: address): bool {
        exists<R>(a1) || exists<R>(a2)
    }
    spec fun local_let_with_memory_access {
        ensures result == exists_R(a1, a2);
    }
    spec define exists_R(a1: address, a2: address): bool {
        let e = exists<R>(a1) || exists<R>(a2);
        e && e || e
    }
}
