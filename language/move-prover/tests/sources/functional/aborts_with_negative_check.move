module 0x42::TestAbortsWithCheck {
    /*
    TODO(refactoring): this test is deactivated until we have ported this (or a similar) feature, or decided to
      drop it in which case the test should be removed.

    fun aborts_with_check_too_many_incorrect(x: u64, y: u64): u64 {
        if (x == 1) {
            abort 2
        };
        if (y == 2) {
            abort 3
        };
        x
    }
    spec aborts_with_check_too_many_incorrect {
        aborts_if x == 1 with 2;
        aborts_if y == 2 with 3;
        aborts_with [check] 2, 3, 4;
        ensures result == x;
    }
    */
}
