module VerifyLoops {

    spec module {
        pragma verify=true;
    }

    // ----------------------
    // `aborts_if` statements
    // ----------------------

    public fun iter10_missing_inc_spec1() {
        let i = 0;
        while (i <= 10) { // an infinite loop
            if (i > 10) abort 10;
        }
    }
    spec fun iter10_missing_inc_spec1 { // Verified. This is expected because Prover checks the partial correctness of this function which contains an infinite loop.
        aborts_if false;
        ensures false;
    }

    public fun iter10_missing_inc_spec2() {
        let i = 0;
        while (i <= 10) { // an infinite loop
            if (i > 10) abort 10;
        }
    }
    spec fun iter10_missing_inc_spec2 { // Verified. This is expected because Prover checks the partial correctness of this function which contains an infinite loop.
        aborts_if true;
        ensures false;
    }

    public fun iter10_no_abort() {
        let i = 0;
        while ({
            spec { assert i <= 11; };
            (i <= 10)
        }) {
            if (i > 10) abort 10;
            i = i + 1;
        }
    }
    spec fun iter10_no_abort { // Verified. Abort cannot happen.
        aborts_if false;
    }

    public fun iter10_no_abort_incorrect() {
        let i = 0;
        while ({
            spec { assert i <= 11; };
            (i <= 10)
        }) {
            if (i > 10) abort 10;
            i = i + 1;
        }
    }
    spec fun iter10_no_abort_incorrect { // Disproved. Abort cannot happen.
        aborts_if true;
    }

    public fun iter10_abort() {
        let i = 0;
        while ({
            spec { assert i <= 7; };
            (i <= 10)
        }) {
            if (i == 7) abort 7;
            i = i + 1;
        }
    }
    spec fun iter10_abort { // Verified. Abort always happens.
        aborts_if true;
    }

    public fun iter10_abort_incorrect() {
        let i = 0;
        while ({
            spec { assert i <= 7; };
            (i <= 10)
        }) {
            if (i == 7) abort 7;
            i = i + 1;
        }
    }
    spec fun iter10_abort_incorrect { // Disproved. Abort always happens.
        aborts_if false;
    }
}
