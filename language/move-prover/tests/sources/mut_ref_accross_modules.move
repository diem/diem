address 0x1:
module TestMutRefs {

    struct T { value: u64 }

    resource struct TSum {
        sum: u64
    }

    spec struct T {
        global spec_sum: u64;

        invariant update value > old(value);
        invariant pack spec_sum = spec_sum + value;
        invariant unpack spec_sum = spec_sum - value;
    }

    spec module {
        invariant global<TSum>(0x0).sum == spec_sum;
    }

    public fun new(x: u64): T acquires TSum {
        let r = borrow_global_mut<TSum>(0x0);
        r.sum = r.sum + x;
        T{value: x}
    }

    public fun delete(x: T) acquires TSum {
        let r = borrow_global_mut<TSum>(0x0);
        let T{value: v} = x;
        r.sum = r.sum - v;
    }

    public fun increment(x: &mut T) acquires TSum {
        x.value = x.value + 1;
        let r = borrow_global_mut<TSum>(0x0);
        r.sum = r.sum + 1;
    }

    // This should fail because the update invariant requires strict monotonic increase of the value.
    // TODO: we see this error only when we comment out the next function. Need to determine where this
    //   comes from, as we usually see Boogie reporting all errors, and not aborting after the first one.
    public fun decrement_invalid(x: &mut T) acquires TSum {
        x.value = x.value - 1;
        let r = borrow_global_mut<TSum>(0x0);
        r.sum = r.sum - 1;
    }

    // This should fail because we do not update the TSum resource.
    public fun increment_invalid(x: &mut T) {
        x.value = x.value + 1;
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
