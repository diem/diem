module AbortsIf {

    // succeeds. Very basic test.
    fun abort1(x: u64, y: u64) {
        if (!(x > y)) abort 1;
    }
    spec fun abort1 {
        aborts_if x <= y;
    }

    // fails, because it does not abort when x <= y
    fun abort2(x: u64, y: u64)
    {
    }
    spec fun abort2 {
        aborts_if x <= y;
    }

    // fails, because it aborts when not x<=y
    fun abort3(x: u64, y: u64) {
        if (x > y) abort 2;
    }
    spec fun abort3 {
        aborts_if x <= y;
    }

    // fails. Violates implicit succeeds_if x >= y when x == y
    fun abort4(x: u64, y: u64) {
        if (!(x > y)) abort 1;
    }
    spec fun abort4 {
        aborts_if x < y;
    }


    // fails. It doesn't abort when x > y
    // Boogie also (correctly) complains because it doesn't SUCCEED on x==y.
    fun abort9(x: u64, y: u64) {
        if (!(x > y)) abort 1;
    }
    spec fun abort9 {
        aborts_if x > y;
        aborts_if x < y;
        ensures x == y;
    }
}
