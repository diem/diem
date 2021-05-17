module 0x42::TestAbortsIf {

    spec module {
        pragma verify = true;
    }

    // -------------------------
    // No `aborts_if` statements
    // -------------------------

    // succeeds, because the specification claims no 'aborts_if' condition.
    fun no_aborts_if(_x: u64, _y: u64) {
        abort 1
    }
    spec no_aborts_if {
    }


    // ----------------------------
    // Single `aborts_if` statement
    // ----------------------------

    // succeeds. Very basic test.
    fun abort1(x: u64, y: u64) {
        if (!(x > y)) abort 1
    }
    spec abort1 {
        aborts_if x <= y;
    }

    // fails, because it does not abort when x <= y.
    fun abort2_incorrect(_x: u64, _y: u64) {
    }
    spec abort2_incorrect {
        aborts_if _x <= _y;
    }

    // succeeds.
    fun abort3(_x: u64, _y: u64) {
        abort 1
    }
    spec abort3 {
        aborts_if true;
    }

    // fails, because it does not abort when x <= y.
    fun abort4_incorrect(x: u64, y: u64) {
        if (x > y) abort 1
    }
    spec abort4_incorrect {
        pragma aborts_if_is_partial = true;
        aborts_if x <= y;
    }

    // fails, because it also aborts when x == y which condition has not been specified.
    fun abort5_incorrect(x: u64, y: u64) {
        if (x <= y) abort 1
    }
    spec abort5_incorrect {
        aborts_if x < y;
    }

    // fails, because it does not abort when x == y.
    fun abort6_incorrect(x: u64, y: u64) {
        if (x < y) abort 1
    }
    spec abort6_incorrect {
        aborts_if x <= y;
    }


    // -------------------------------
    // Multiple `aborts_if` statements
    // -------------------------------

    // succeeds. Multiple 'aborts_if' conditions are 'or'ed.
    fun multi_abort1(x: u64, y: u64) {
        if (x <= y) abort 1
    }
    spec multi_abort1 {
        aborts_if x < y;
        aborts_if x == y;
    }

    // fails, because it does not abort when x == y.
    fun multi_abort2_incorrect(x: u64, y: u64) {
        if (x < y) abort 1
    }
    spec multi_abort2_incorrect {
        aborts_if x < y;
        aborts_if x == y;
    }

    // fails, because it also abort when x > y which condition has not been specified.
    fun multi_abort3_incorrect(_x: u64, _y: u64) {
        abort 1
    }
    spec multi_abort3_incorrect {
        aborts_if _x < _y;
        aborts_if _x == _y;
    }

    // succeeds. Aborts all the time.
    fun multi_abort4(_x: u64, _y: u64) {
        abort 1
    }
    spec multi_abort4 {
        aborts_if _x < _y;
        aborts_if _x == _y;
        aborts_if _x > _y;
    }

    fun multi_abort5_incorrect(x: u64) {
        if (x == 0) {
            abort 1
        };
    }
    spec multi_abort5_incorrect {
        aborts_if true;
        aborts_if x > 0;
    }

    // -------------------------------
    // `aborts_if` pragmas
    // -------------------------------

    fun abort_at_2_or_3(x: u64) {
        if (x == 2 || x == 3) abort 1;
    }
    spec abort_at_2_or_3 {
        // It is ok to specify only one abort condition of we set partial to true.
        pragma aborts_if_is_partial = true;
        aborts_if x == 2;
    }

    fun abort_at_2_or_3_total_incorrect(x: u64) {
        if (x == 2 || x == 3) abort 1;
    }
    spec abort_at_2_or_3_total_incorrect {
        // Counter check that we get an error message without the pragma.
        // pragma aborts_if_is_partial = false;  // default
        aborts_if x == 2;
    }

    fun abort_at_2_or_3_spec_incorrect(x: u64) {
        if (x == 2 || x == 3) abort 1;
    }
    spec abort_at_2_or_3_spec_incorrect {
        // Even with the pragma, wrong aborts_if will be flagged.
        pragma aborts_if_is_partial = true;
        aborts_if x == 4;
    }

    fun abort_at_2_or_3_strict_incorrect(x: u64) {
        if (x == 2 || x == 3) abort 1;
    }
    spec abort_at_2_or_3_strict_incorrect {
        // When the strict mode is enabled, no aborts_if clause means aborts_if false.
        pragma aborts_if_is_strict = true;
    }

    fun abort_1() {
        abort 1
    }
    spec abort_1 {
        pragma opaque;
        aborts_if true with 1;
    }

    fun aborts_if_with_code(x: u64) {
        if (x == 2 || x == 3) abort_1();
    }
    spec aborts_if_with_code {
        // It is ok to specify only one abort condition of we set partial to true.
        pragma aborts_if_is_partial = true;
        aborts_if x == 2 with 1;
        aborts_with 1;
    }
}
