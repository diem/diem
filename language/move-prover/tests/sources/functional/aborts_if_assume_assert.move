module TestAbortsIfAssumeAssert {

    spec module {
        pragma verify = true;

        // Implicitly adds aborts_if false to all functions without aborts spec
        pragma aborts_if_is_strict = true;
    }

    // aborts_if [assert]
    // ==================

    // Function with asserted aborts_if
    public fun asserted(x: u64): u64 {
        x - 1
    }
    spec fun asserted {
        aborts_if [assert] x == 0;
    }

    // Call to asserted -- precondition of aborts_if holds
    public fun asserted_caller(): u64 {
        asserted(1)
    }
    spec fun asserted_spec_invalid {
        requires x > 0;
    }

    // Call to asserted -- precondition of aborts_if does not hold
    public fun asserted_caller_invalid(): u64 {
        asserted(0)
    }

    // Call to asserted where we catch the error and produce a logical one instead - precondition of aborts_if does hold
    public fun assert_caller_aborts_with_logical_error(x: u64): u64 {
        if (x == 0) abort(43);
        asserted(x)
    }
    spec fun assert_caller_aborts_with_logical_error {
        aborts_if [assert] x == 0 with 43;
    }

    // Function with asserted aborts_if which does not actually hold
    public fun asserted_spec_invalid(x: u64): u64 {
        x
    }
    spec fun asserted_spec_invalid {
        aborts_if [assert] x == 1;
    }

    // aborts_if [assume]
    // ==================

    // Function with assumed aborts_if
    public fun assumed(x: u64): u64 {
        x - 1
    }
    spec fun assumed {
        aborts_if [assume] x == 0;
    }

    // Call to assumed -- this will exclude `x == 0` from verification.
    public fun assumed_caller(x: u64): u64 {
        assumed(x)
    }

    // Call to assumed -- unsound!
    public fun assumed_caller_unsound(): u64 {
        assumed(0)
    }
    spec fun assumed_caller_unsound {
        ensures true == false; // oops!
    }

    // Function with assumed aborts_if which does not actually hold
    public fun assumed_spec_invalid(x: u64): u64 {
        x
    }
    spec fun assumed_spec_invalid {
        aborts_if [assume] x == 1;
    }
}
