module TestAbortsIfWithCodeError {

    spec module {
        pragma verify = true;
    }

    // Test to trigger the compile error of using codes with partial aborts and without aborts_with.
    fun aborts_with_codes_partial(x: u64, y: u64): u64 {
        if (x == 1) {
            abort 2
        };
        if (y == 2) {
            abort 3
        };
        x + y
    }
    spec fun aborts_with_codes_partial {
        pragma aborts_if_is_partial = true;
        aborts_if x == 1 with 2;
        aborts_if y == 2;
        ensures result == x + y;
    }
}
