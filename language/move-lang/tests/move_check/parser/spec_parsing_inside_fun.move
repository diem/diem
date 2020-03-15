module M {
    fun specs_in_fun(x: u64, n: u64) {
        // an ordinary assume
        spec {
            assume x > 42;
        };

        // loop invariant is written in a spec block inside the loop condition
        while ({spec {assert x < 42;}; n < 64}) {
            spec {
                assert x > 42;
                assert 0 < x;
            };
            n = n + 1
        };

        // an ordinary assert
        spec {
            assert x > 42;
        };

        // loop invariant is written in a spec block at the beginning of loop body
        loop {
            spec {
                assert x > 42;
                assert 0 < x;
            };
            n = n + 1
        };

        // the following should parse successfully but fail typing
        spec {} + 1;
        spec {} && spec {};
        &mut spec {};
    }
}
