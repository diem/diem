module TestAbortsIf {

    // -------------------------
    // No `aborts_if` statements
    // -------------------------

    // succeeds, because the specification claims no 'aborts_if' condition.
    fun no_aborts_if(_x: u64, _y: u64) {
        abort 1
    }
    spec fun no_aborts_if {
    }


    // ----------------------------
    // Single `aborts_if` statement
    // ----------------------------

    // succeeds. Very basic test.
    fun abort1(x: u64, y: u64) {
        if (!(x > y)) abort 1
    }
    spec fun abort1 {
        aborts_if x <= y;
    }

    // fails, because it does not abort when x <= y.
    fun abort2_incorrect(_x: u64, _y: u64) {
    }
    spec fun abort2_incorrect {
        aborts_if _x <= _y;
    }

    // succeeds.
    fun abort3(_x: u64, _y: u64) {
        abort 1
    }
    spec fun abort3 {
        aborts_if true;
    }

    // fails, because it does not abort when x <= y.
    fun abort4_incorrect(x: u64, y: u64) {
        if (x > y) abort 1
    }
    spec fun abort4_incorrect {
        aborts_if x <= y;
    }

    // fails, because it also aborts when x == y which condition has not been specified.
    fun abort5_incorrect(x: u64, y: u64) {
        if (x <= y) abort 1
    }
    spec fun abort5_incorrect {
        aborts_if x < y;
    }

    // fails, because it does not abort when x == y.
    fun abort6_incorrect(x: u64, y: u64) {
        if (x < y) abort 1
    }
    spec fun abort6_incorrect {
        aborts_if x <= y;
    }


    // -------------------------------
    // Multiple `aborts_if` statements
    // -------------------------------

    // succeeds. Multiple 'aborts_if' conditions are 'or'ed.
    fun multi_abort1(x: u64, y: u64) {
        if (x <= y) abort 1
    }
    spec fun multi_abort1 {
        aborts_if x < y;
        aborts_if x == y;
    }

    // fails, because it does not abort when x == y.
    fun multi_abort2_incorrect(x: u64, y: u64) {
        if (x < y) abort 1
    }
    spec fun multi_abort2_incorrect {
        aborts_if x < y;
        aborts_if x == y;
    }

    // fails, because it also abort when x > y which condition has not been specified.
    fun multi_abort3_incorrect(_x: u64, _y: u64) {
        abort 1
    }
    spec fun multi_abort3_incorrect {
        aborts_if _x < _y;
        aborts_if _x == _y;
    }

    // succeeds. Aborts all the time.
    fun multi_abort4(_x: u64, _y: u64) {
        abort 1
    }
    spec fun multi_abort4 {
        aborts_if _x < _y;
        aborts_if _x == _y;
        aborts_if _x > _y;
    }

    fun multi_abort5_incorrect(x: u64) {
        if (x == 0) {
            abort 1
        };
    }
    spec fun multi_abort5_incorrect {
        aborts_if true;
        aborts_if x > 0;
    }
}
