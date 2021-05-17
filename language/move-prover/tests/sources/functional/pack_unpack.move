address 0x1 {
module TestPackUnpack {

    /*
    TODO(refactoring): this test is deactivated until we have ported this (or a similar) feature, or decided to
      drop it in which case the test should be removed

    spec module {
        pragma verify = true;
    }

    resource struct R { nested: S  }
    spec R {
        global r_count: num;
        invariant pack r_count = r_count + 1;
        invariant unpack r_count = r_count - 1;
    }

    resource struct S { value: u64 }
    spec S {
        global s_sum: num;
        invariant value > 0;
        invariant pack s_sum = s_sum + value;
        invariant unpack s_sum = s_sum - value;

    }

    // Working with values
    // ===================

    /// Test that global vars are updated on creation.
    public fun create(): R {
        R{nested: S{value: 3}}
    }
    spec create {
        ensures result == R{nested: S{value: 3}};
        ensures r_count == old(r_count) + 1;
        ensures s_sum == old(s_sum) + 3;
    }

    /// Test that global vars are updated on destroy.
    public fun destroy(r: R): u64 {
        let R{nested} = r;
        let S{value} = nested;
        value
    }
    spec destroy {
        ensures result == old(r.nested.value);
        ensures r_count == old(r_count) - 1;
        ensures s_sum == old(s_sum) - old(r.nested.value);
    }

    /// Test that global vars do change if R is destroyed and S updated.
    public fun extract_and_update(r: R): S {
        let R{nested} = r;
        nested.value = nested.value + 3;
        nested
    }
    spec extract_and_update {
        ensures result == S{value: old(r.nested.value) + 3};
        ensures r_count == old(r_count) - 1;
        ensures s_sum == old(s_sum) + 3;
    }

    // Working with references
    // =======================

    /// Test that global vars do not change for a public function taking a `&mut R`
    public fun read_ref_unchanged(r: &mut R): u64 {
        let R{nested} = r;
        nested.value
    }
    spec read_ref_unchanged {
        ensures result == r.nested.value;
        ensures r_count == old(r_count);
        ensures s_sum == old(s_sum);
    }

    /// Test that global vars do change if nested is updated.
    public fun update_ref_changed(r: &mut R): u64 {
        let R{nested} = r;
        nested.value = nested.value + 2;
        nested.value
    }
    spec update_ref_changed {
        ensures result == r.nested.value;
        ensures r.nested.value == old(r.nested.value) + 2;
        ensures r_count == old(r_count);
        ensures s_sum == old(s_sum) + 2;
    }

    /// Test that moving a value from one to another does not change global balance.
    public fun move_ref_unchanged(r1: &mut R, r2: &mut R) {
        r2.nested.value = r2.nested.value + r1.nested.value - 1;
        r1.nested.value = 1; // We can't reset this to 0 because then invariant fails
    }
    spec move_ref_unchanged {
        ensures r2.nested.value == old(r2.nested.value) + old(r1.nested.value) - 1;
        ensures r1.nested.value == 1;
        ensures r_count == old(r_count);
        ensures s_sum == old(s_sum);
    }

    /// Test that moving a value from one to another does not change global balance,
    /// but fails invariant if value is set to 0.
    public fun move_ref_unchanged_invariant_incorrect(r1: &mut R, r2: &mut R) {
        r2.nested.value = r2.nested.value + r1.nested.value;
        r1.nested.value = 0;
    }
    spec move_ref_unchanged_invariant_incorrect {
        ensures r2.nested.value == old(r2.nested.value) + old(r1.nested.value);
        ensures r1.nested.value == 0;
        ensures r_count == old(r_count);
        ensures s_sum == old(s_sum);
    }

    /// Test that calling a private function with a reference violating the invariant
    /// temporarily succeeds.
    public fun call_private_violating_invariant(r: &mut R) {
        let s = &mut r.nested;
        private_update_value(s, 0);
        s.value = 1; // restore invariant.
    }
    fun private_update_value(s: &mut S, v: u64) {
        s.value = v;
    }
    spec call_private_violating_invariant {
        ensures r.nested.value == 1;
    }

    /// Test that calling a public function with a reference violating the invariant
    /// fails.
    public fun call_public_violating_invariant_incorrect(r: &mut R) {
        let s = &mut r.nested;
        public_update_value(s, 0);
        s.value = 1; // restore invariant.
    }
    public fun public_update_value(s: &mut S, v: u64) {
        s.value = v;
    }
    spec call_public_violating_invariant_incorrect {
        ensures r.nested.value == 1;
    }
    spec public_update_value {
        pragma verify = false; // turn noise off for this one, we are interested in the caller
    }

    /// Test that selecting a value from a reference currently no satisfying the invariant fails.
    public fun select_value_violating_invariant_incorrect(r: &mut R): u64 {
        let s = &mut r.nested;
        s.value = 0; // invariant violated
        read_S_from_immutable(s)
    }
    fun read_S_from_immutable(s: &S): u64 {
        s.value
    }
    spec select_value_violating_invariant_incorrect {
        ensures result == 0;
    }

    /// Test that selecting a value from a reference passed in as a parameter to a private
    /// function, and therefore, currently no satisfying the invariant, fails.
    fun private_select_value_violating_invariant_incorrect(r: &mut R): u64 {
        // Because this is a private function, invariant for r may not hold.
        read_S_from_immutable(&r.nested)
    }
    spec private_select_value_violating_invariant_incorrect {
        ensures result == r.nested.value;
    }

    /// Test that selecting a value from a reference passed in as a parameter to a private
    /// function, and therefore, currently no satisfying the invariant, fails.
    fun private_pass_value_violating_invariant_incorrect(s: &mut S): u64 {
        // Because this is a private function, invariant for s may not hold.
        read_S_from_immutable(s)
    }
    spec private_pass_value_violating_invariant_incorrect {
        ensures result == s.value;
    }

    // Working with references returned by functions
    // =============================================

    public fun update_via_returned_ref(): R {
        let r = R{nested: S{value: 1}};
        let v_ref = get_value_ref(&mut r);
        *v_ref = 2;
        r
    }
    fun get_value_ref(r: &mut R): &mut u64 {
        &mut r.nested.value
    }
    spec update_via_returned_ref {
        ensures result.nested.value == 2;
        ensures r_count == old(r_count) + 1;
        ensures s_sum == old(s_sum) + 2;
    }

    public fun update_via_returned_ref_invariant_incorrect(): R {
        let r = R{nested: S{value: 1}};
        let v_ref = get_value_ref(&mut r);
        *v_ref = 0;
        r
    }

    public fun update_via_returned_ref_var_incorrect(): R {
        let r = R{nested: S{value: 1}};
        let v_ref = get_value_ref(&mut r);
        *v_ref = 1;
        r
    }
    spec update_via_returned_ref_var_incorrect {
        ensures result.nested.value == 1;
        ensures r_count == old(r_count) + 1;
        ensures s_sum == old(s_sum) + 2;
    }

    */

}
}
