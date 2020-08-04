module TestEnsuresFalseSmokeTest {
    spec module {
        pragma verify = false;
        pragma always_aborts_test = true;
    }

    /// Expect this not to return any errors
    public fun aborts_if_true_succeeds() {
    }
    spec fun aborts_if_true_succeeds {
    }

    /// Expect this to return an error because it passes the ensures false
    public fun aborts_if_true_smoke_test_fails() {
        abort 1
    }
    spec fun aborts_if_true_smoke_test_fails {
    }

    /// More complicated example where smoke test should catch an error
    public fun aborts_if_true_smoke_test_fails_2(): u64 {
        let x: u64;
        x = 1;
        if (x > 0) {
            abort 1
        };
        x
    }
    spec fun aborts_if_true_smoke_test_fails_2 {
    }

    /// Example where an invariant causes the smoke test to always abort
    /// > NOTE(kkmc): It seems like we should require inductive proofs of global invariants
    /// to include both a base case (e.g. automatically annotating move_to, move_from, pack
    /// unpack where base cases should be proved) and induction step?
    /// Currently, if we have an invariant I, we don't require a base case and it is
    /// generally written manually by stating !old(exists<S>) && exists<S> ==> I.
    resource struct S {
        x: u64
    }
    public fun aborts_when_strong_inv_holds_incorrect(): u64
    acquires S {
        let x = borrow_global_mut<S>(0x0).x;
        if (x < 100) {
            abort 1
        };
        x
    }
    spec schema S_x_always_gt_100 {
        invariant module global<S>(0x0).x < 100;
    }
    spec module {
        apply S_x_always_gt_100 to aborts_when_strong_inv_holds_incorrect;
    }
    /// This function should flag that abort is always true
    spec fun aborts_when_strong_inv_holds_incorrect {
        aborts_if !exists<S>(0x0);
        aborts_if global<S>(0x0).x < 100;
        ensures result == global<S>(0x0).x;
    }

    /// Nested function calls
    /// NOTE: Checking that the nested calls don't raise an error
    /// This is a test to check that we don't miss an error as
    /// a result of nested calls, which was happening earlier.
    public fun base_call(): u64 {
        nested_call(10)
    }
    public fun nested_call(x: u64): u64 {
        x + 1
    }
    spec fun nested_call {
        requires x == 10;
        ensures result == 11;
    }
    spec fun base_call {
        ensures result == 11;
    }
}
