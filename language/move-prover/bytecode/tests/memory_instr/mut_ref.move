address 0x1 {
module TestMutRefs {
    struct T has copy, drop { value: u64 }

    // Resource to track the sum of values in T
    struct TSum has key {
        sum: u64
    }

    public fun new(x: u64): T acquires TSum {
        let r = borrow_global_mut<TSum>(@0x0);
        r.sum = r.sum + x;
        T{value: x}
    }

    public fun delete(x: T) acquires TSum {
        let r = borrow_global_mut<TSum>(@0x0);
        let T{value: v} = x;
        r.sum = r.sum - v;
    }

    public fun increment(x: &mut T) acquires TSum {
        x.value = x.value + 1;
        let r = borrow_global_mut<TSum>(@0x0);
        r.sum = r.sum + 1;
    }

    public fun increment_invalid(x: &mut T) {
        x.value = x.value + 1;
    }

    public fun decrement_invalid(x: &mut T) acquires TSum {
        x.value = x.value - 1;
        let r = borrow_global_mut<TSum>(@0x0);
        r.sum = r.sum - 1;
    }

    fun private_decrement(x: &mut T) acquires TSum {
        x.value = x.value - 1;
        let r = borrow_global_mut<TSum>(@0x0);
        r.sum = r.sum - 1;
    }

    public fun data_invariant(_x: &mut T) {
    }

    fun private_data_invariant_invalid(_x: &mut T) {
    }

    fun private_to_public_caller(r: &mut T) acquires TSum {
        increment(r);
    }

    fun private_to_public_caller_invalid_data_invariant() acquires TSum {
        let x = new(1);
        let r = &mut x;
        private_decrement(r);
        increment(r);
    }
}

module TestMutRefsUser {
    use 0x1::TestMutRefs;

    public fun valid() {
        let x = TestMutRefs::new(4);
        TestMutRefs::increment(&mut x);
        TestMutRefs::delete(x);
    }
}
}
