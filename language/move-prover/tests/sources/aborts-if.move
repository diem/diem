module AbortsIf {

    // succeeds. Very basic test.
    fun abort01(x: u64, y: u64) {
        if (!(x > y)) abort 1
    }
    spec fun abort01 {
        aborts_if x <= y;
    }

    // fails, because it does not abort when x <= y.
    fun abort02(x: u64, y: u64) {
    }
    spec fun abort02 {
        aborts_if x <= y;
    }

    // succeeds, because the specification claims no 'aborts_if' condition.
    fun abort03(x: u64, y: u64) {
        abort 1
    }
    spec fun abort03 {
    }

    // succeeds.
    fun abort04(x: u64, y: u64) {
        abort 1
    }
    spec fun abort04 {
        aborts_if true;
    }

    // fails, because it does not abort when x <= y.
    fun abort05(x: u64, y: u64) {
        if (x > y) abort 1
    }
    spec fun abort05 {
        aborts_if x <= y;
    }

    // fails, because it also aborts when x == y which condition has not been specified.
    fun abort06(x: u64, y: u64) {
        if (x <= y) abort 1
    }
    spec fun abort06 {
        aborts_if x < y;
    }

    // fails, because it does not abort when x == y.
    fun abort07(x: u64, y: u64) {
        if (x < y) abort 1
    }
    spec fun abort07 {
        aborts_if x <= y;
    }

    // succeeds. Multiple 'aborts_if' conditions are 'or'ed.
    fun abort08(x: u64, y: u64) {
        if (x <= y) abort 1
    }
    spec fun abort08 {
        aborts_if x < y;
        aborts_if x == y;
    }

    // fails, because it does not abort when x == y.
    fun abort09(x: u64, y: u64) {
        if (x < y) abort 1
    }
    spec fun abort09 {
        aborts_if x < y;
        aborts_if x == y;
    }

    // fails, because it also abort when x > y which condition has not been specified.
    fun abort10(x: u64, y: u64) {
        abort 1
    }
    spec fun abort10 {
        aborts_if x < y;
        aborts_if x == y;
    }

    // succeeds. Aborts all the time.
    fun abort11(x: u64, y: u64) {
        abort 1
    }
    spec fun abort11 {
        aborts_if x < y;
        aborts_if x == y;
        aborts_if x > y;
    }
}
