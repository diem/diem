// flag: --trace
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
    spec spec_let {
        let x = a + 1;
        let y = x + b;
        ensures result_1 == y;
        ensures result_2 == result_1 - 1;
    }

    fun spec_let_with_old(a: &mut u64, b: &mut u64) {
        *a = *a + 1 + *b;
        *b = *a + *b;
    }
    spec spec_let_with_old {
       let post new_a = old(a) + 1 + old(b);
       let post new_b = a + old(b);
       ensures a == new_a;
       ensures b == new_b;
    }

    fun spec_let_with_abort(a: &mut u64, b: &mut u64) {
        let saved_a = *a;
        *a = *a / (*a + *b);
        *b = saved_a * *b;
    }
    spec spec_let_with_abort {
        pragma opaque;
        let sum = a + b;
        let product = a * b;
        aborts_if sum == 0;
        aborts_if sum > MAX_U64;
        aborts_if product > MAX_U64;
        let post new_a = old(a) / sum;
        ensures a == new_a + sum - sum;
        ensures b == product;
    }

    fun spec_let_with_abort_opaque_caller(a: &mut u64, b: &mut u64) {
        spec_let_with_abort(a, b)
    }
    spec spec_let_with_abort_opaque_caller {
        // Same as the callee
        let sum = a + b;
        let product = a * b;
        aborts_if sum == 0;
        aborts_if sum > MAX_U64;
        aborts_if product > MAX_U64;
        let post new_a = old(a) / sum;
        ensures a == new_a + sum - sum;
        ensures b == product;
    }

    fun spec_let_with_abort_incorrect(a: &mut u64, b: &mut u64) {
        let saved_a = *a;
        *a = *a / (*a + *b);
        *b = saved_a * *b;
    }
    spec spec_let_with_abort_incorrect {
        let sum = a + b;
        let product = a * b;
        aborts_if sum != 0;
        aborts_if sum >= MAX_U64;
        aborts_if product >= MAX_U64;
        let post new_a = old(a) / sum;
        ensures a == new_a;
        ensures b == product;
    }

    fun spec_let_with_schema(a: &mut u64, b: &mut u64) {
        let saved_a = *a;
        *a = *a / (*a + *b);
        *b = saved_a * *b;
    }
    spec spec_let_with_schema {
        let sum = a + b;
        let product = a * b;
        aborts_if sum == 0;
        aborts_if sum > MAX_U64;
        aborts_if product > MAX_U64;
        let post new_a = old(a) / sum;
        include Ensures{actual: a, expected: new_a + sum - sum};
        include Ensures{actual: b, expected: product};
    }
    spec schema Ensures {
        actual: u64;
        expected: u64;
        let post a = expected;
        let post b = actual;
        include Ensures2{a: a, b: b};
    }
    spec schema Ensures2 {
        a: u64;
        b: u64;
        ensures a == b;
    }

    // Local Let
    // =========

    fun local_let_with_memory_access(a1: address, a2: address): bool {
        exists<R>(a1) || exists<R>(a2)
    }
    spec local_let_with_memory_access {
        ensures result == exists_R(a1, a2);
    }
    spec fun exists_R(a1: address, a2: address): bool {
        let e = exists<R>(a1) || exists<R>(a2);
        e && e || e
    }

    // Let Shadowing
    // =============

    fun shadowed(v: u64): u64 {
        v
    }
    spec shadowed {
        include Shadowed1;
    }

    spec schema Shadowed1 {
        v: u64;
        include Shadowed2;
    }

    spec schema Shadowed2 {
        let v = true;
        ensures v == true;
    }
}
