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
        pragma verify=true;
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
        pragma verify=true;
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
        pragma verify=true;
        aborts_if false;
    }

    public fun nested_loop_correct(x: u64, y: u64) {
        loop {
            loop {
                if (x <= y) {
                    break
                };
                y = y + 1;
            };

            if (y <= x) {
                break
            };
            x = x + 1;
        };
        spec {
            assert x == y;
        };
    }
    spec fun nested_loop_correct {
        aborts_if false;
    }

    public fun nested_loop_outer_invariant_incorrect(x: u64, y: u64) {
        spec {
            assume x != y;
        };
        loop {
            spec {
                assert x != y;
            };
            loop {
                if (x <= y) {
                    break
                };
                y = y + 1;
            };

            if (y <= x) {
                break
            };
            x = x + 1;
        };
    }
    spec fun nested_loop_outer_invariant_incorrect {
        aborts_if false;
    }

    public fun nested_loop_inner_invariant_incorrect(x: u64, y: u64) {
        spec {
            assume x != y;
        };
        loop {
            loop {
                spec {
                    assert x != y;
                };
                if (x <= y) {
                    break
                };
                y = y + 1;
            };

            if (y <= x) {
                break
            };
            x = x + 1;
        };
    }
    spec fun nested_loop_inner_invariant_incorrect {
        aborts_if false;
    }

    public fun loop_with_two_back_edges_correct(x: u64, y: u64) {
        loop {
            if (x > y) {
                y = y + 1;
                continue
            };
            if (y > x) {
                x = x + 1;
                continue
            };
            break
        };
        spec {
            assert x == y;
        };
    }
    spec fun loop_with_two_back_edges_correct {
        aborts_if false;
    }

    public fun loop_with_two_back_edges_incorrect(x: u64, y: u64) {
        spec {
            assume x != y;
        };
        loop {
            spec {
                assert x != y;
            };
            if (x > y) {
                y = y + 1;
                continue
            };
            if (y > x) {
                x = x + 1;
                continue
            };
            break
        };
    }
    spec fun loop_with_two_back_edges_incorrect {
        aborts_if false;
    }
}
