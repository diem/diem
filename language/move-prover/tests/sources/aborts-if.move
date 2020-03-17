module TestAbortsIf {

    // -------------------------
    // No `aborts_if` statements
    // -------------------------

    // succeeds, because the specification claims no 'aborts_if' condition.
    fun aborts_if_ok(x: u64, y: u64) {
        abort 1
    }
    spec fun aborts_if_ok {
    }


    // ----------------------------
    // Single `aborts_if` statement
    // ----------------------------

    // succeeds. Very basic test.
    fun abort1_ok(x: u64, y: u64) {
        if (!(x > y)) abort 1
    }
    spec fun abort1_ok {
        aborts_if x <= y;
    }

    // fails, because it does not abort when x <= y.
    fun abort2_bad(x: u64, y: u64) {
    }
    spec fun abort2_bad {
        aborts_if x <= y;
    }

    // succeeds.
    fun abort3_ok(x: u64, y: u64) {
        abort 1
    }
    spec fun abort3_ok {
        aborts_if true;
    }

    // fails, because it does not abort when x <= y.
    fun abort4_bad(x: u64, y: u64) {
        if (x > y) abort 1
    }
    spec fun abort4_bad {
        aborts_if x <= y;
    }

    // fails, because it also aborts when x == y which condition has not been specified.
    fun abort5_bad(x: u64, y: u64) {
        if (x <= y) abort 1
    }
    spec fun abort5_bad {
        aborts_if x < y;
    }

    // fails, because it does not abort when x == y.
    fun abort6_bad(x: u64, y: u64) {
        if (x < y) abort 1
    }
    spec fun abort6_bad {
        aborts_if x <= y;
    }


    // -------------------------------
    // Multiple `aborts_if` statements
    // -------------------------------

    // succeeds. Multiple 'aborts_if' conditions are 'or'ed.
    fun multi_abort1_ok(x: u64, y: u64) {
        if (x <= y) abort 1
    }
    spec fun multi_abort1_ok {
        aborts_if x < y;
        aborts_if x == y;
    }

    // fails, because it does not abort when x == y.
    fun multi_abort2_bad(x: u64, y: u64) {
        if (x < y) abort 1
    }
    spec fun multi_abort2_bad {
        aborts_if x < y;
        aborts_if x == y;
    }

    // fails, because it also abort when x > y which condition has not been specified.
    fun multi_abort3_bad(x: u64, y: u64) {
        abort 1
    }
    spec fun multi_abort3_bad {
        aborts_if x < y;
        aborts_if x == y;
    }

    // succeeds. Aborts all the time.
    fun multi_abort4_ok(x: u64, y: u64) {
        abort 1
    }
    spec fun multi_abort4_ok {
        aborts_if x < y;
        aborts_if x == y;
        aborts_if x > y;
    }
}
