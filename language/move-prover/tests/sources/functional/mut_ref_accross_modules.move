address 0x1 {
module TestMutRefs {

    spec module {
        pragma verify = true;
    }


    struct T { value: u64 }

    // Resource to track the sum of values in T
    resource struct TSum {
        sum: u64
    }

    spec struct T {
        // global specification variable tracking sum of values.
        global spec_sum: u64;

        // Data invariant.
        invariant value > 0;

        // Pack/unpack invariants updating spec var
        invariant pack spec_sum = spec_sum + value;
        invariant unpack spec_sum = spec_sum - value;
    }

    spec module {
        // Module invariant that the tracked resource sum equals the spec sum
        invariant global<TSum>(0x0).sum == spec_sum;
    }

    // This succeeds because module invariant can be assumed on entry (public function) and is maintained on exit.
    public fun new(x: u64): T acquires TSum {
        let r = borrow_global_mut<TSum>(0x0);
        r.sum = r.sum + x;
        T{value: x}
    }
    spec fun new {
        requires x > 0;
    }

    // This succeeds because module invariant can be assumed on entry (public function) and is maintained on exit.
    public fun delete(x: T) acquires TSum {
        let r = borrow_global_mut<TSum>(0x0);
        let T{value: v} = x;
        r.sum = r.sum - v;
    }

    // This succeeds because module invariant can be assumed on entry (public function) and is maintained on exit.
    public fun increment(x: &mut T) acquires TSum {
        x.value = x.value + 1;
        let r = borrow_global_mut<TSum>(0x0);
        r.sum = r.sum + 1;
    }

    // This should fail because we do not update the TSum resource.
    public fun increment_invalid(x: &mut T) {
        x.value = x.value + 1;
    }

    // This should fail because we violate the data invariant (value always > 0). The data invariant is enforced
    // on exit even though it works on a mut ref since this is a public method.
    public fun decrement_invalid(x: &mut T) acquires TSum {
        x.value = x.value - 1;
        let r = borrow_global_mut<TSum>(0x0);
        r.sum = r.sum - 1;
    }

    // This one is exactly the same as the previous but succeeds since it is private. The data invariant is not
    // enforced because the mut ref is manipulated privately.
    fun private_decrement(x: &mut T) acquires TSum {
        x.value = x.value - 1;
        let r = borrow_global_mut<TSum>(0x0);
        r.sum = r.sum - 1;
    }

    // TODO: This function should verify because the data invariant for a mut ref is expected to hold. Currently, it verifies but for the wrong reason: the resource invariant is assumed on the inout value modeling x.
    public fun data_invariant(x: &mut T) {
        spec {
            assert x.value > 0;
        }
    }

    // TODO: This function should fail because the data invariant for a mut ref is not expected to hold. Currently, it verifies because the resource invariant is assumed on the inout value modeling x.
    fun private_data_invariant_invalid(x: &mut T) {
        spec {
            assert x.value > 0;
        }
    }

    // The next function should succeed because calling a public function from a private one maintains module invariants.
    // TODO: This function fails because PackRef(r) is called just before increment(r).
    fun private_to_public_caller(r: &mut T) acquires TSum {
        // Before call to public increment, data invariants and module invariant must hold.
        // Here we assume them, and force spec_sum to start with a zero value.
        spec {
            assume spec_sum == 0;
            assume r.value > 0;
            assume global<TSum>(0x0).sum == spec_sum;
        };
        increment(r);
        // After call to increment, we expect spec_sum to be incremented.
        spec {
            assert spec_sum == 1;
        };
    }

     // The next function fails because module invariant doesn't hold before calling public function.
     fun private_to_public_caller_invalid_precondition(r: &mut T) acquires TSum {
         increment(r);
     }

     // TODO: This function should fail because the data invariant does not hold at the call to increment. But, currently it does not fail since there is no data invariant assertion at the call to increment (due to my changes?).
     fun private_to_public_caller_invalid_data_invariant() acquires TSum {
         // Before call to public new(), assume module invariant.
         spec {
            assume global<TSum>(0x0).sum == spec_sum;
         };
         let x = new(1);
         let r = &mut x;
         // The next call will lead to violation of data invariant. However, because it is private, no
         // problem reported yet.
         private_decrement(r);
         // The next call enforces both module and data invariants.
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
