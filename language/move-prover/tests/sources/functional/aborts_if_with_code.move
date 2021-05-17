module 0x42::TestAbortsIfWithCode {

    spec module {
        pragma verify = true;
    }

    // AbortsIf With Code
    // ==================

    // Basic tests for error codes.
    fun conditional_abort(x: u64, y: u64): u64 {
        if (x == 1) {
            abort 2
        };
        if (y == 2) {
            abort 3
        };
        // This one can also abort on overflow, with execution failure (code = -1).
        x + y
    }
    spec conditional_abort {
        aborts_if x == 1 with 2;
        aborts_if y == 2 with 3;
        aborts_if x + y > MAX_U64 with EXECUTION_FAILURE;
        ensures result == x + y;
    }

    // Basic test of wrong codes, failing verification.
    fun conditional_abort_invalid(x: u64, y: u64): u64 {
        if (x == 1) {
            abort 2
        };
        if (y == 2) {
            abort 3
        };
        x
    }
    spec conditional_abort_invalid {
        aborts_if x == 1 with 1; // wrong code
        aborts_if y == 2 with 3;
        ensures result == x;
    }

    // Test for wrong code w/ EXECUTION_FAILURE
    fun exec_failure_invalid(x: u64): u64 {
        10 / x
    }
    spec exec_failure_invalid {
        aborts_if x == 0 with 1; // wrong code
        ensures result == 10 / x;
    }

    // AbortsIf With Code Mixed with Without Code
    // ==========================================

    fun aborts_if_with_code_mixed(x: u64) {
        if (x == 1) {
            abort(1)
        };
        if (x == 2) {
            abort(2)
        };
    }
    spec aborts_if_with_code_mixed {
        aborts_if x == 1;
        aborts_if x == 2 with 2;
    }

    fun aborts_if_with_code_mixed_invalid(x: u64) {
        if (x == 1) {
            abort(1)
        };
        if (x == 2) {
            abort(2)
        };
    }
    spec aborts_if_with_code_mixed_invalid {
        aborts_if x == 1;
        aborts_if x == 2 with 1;
    }

    // AbortsWith
    // ==========

    fun aborts_with(x: u64) {
        if (x == 1) {
            abort(1)
        };
        if (x == 2) {
            abort(2)
        };
    }
    spec aborts_with {
        aborts_with 1,2;
    }

    fun aborts_with_invalid(x: u64) {
        if (x == 1) {
            abort(1)
        };
        if (x == 2) {
            abort(2)
        };
    }
    spec aborts_with_invalid {
        aborts_with 1,3;
    }

    fun aborts_with_mixed(x: u64) {
        if (x == 1) {
            abort(1)
        };
        if (x == 2) {
            abort(2)
        };
    }
    spec aborts_with_mixed {
        pragma aborts_if_is_partial = true;
        aborts_if x == 1 with 1;
        aborts_with 2;
    }

    fun aborts_with_mixed_invalid(x: u64) {
        if (x == 1) {
            abort(1)
        };
        if (x == 2) {
            abort(1)
        };
    }
    spec aborts_with_mixed_invalid {
        pragma aborts_if_is_partial = true;
        aborts_if x == 1 with 1;
        aborts_with 2;
    }


}
