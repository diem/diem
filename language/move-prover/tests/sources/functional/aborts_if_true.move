module TestEnsuresFalseSmokeTest {
    spec module {
        pragma verify = true;
    }

    /// Expect this not to return any errors
    public fun aborts_if_true_succeeds() {
    }
    spec fun aborts_if_true_succeeds {
        pragma always_aborts_test = true;
    }

    /// Expect this to return an error because it passes the ensures false
    public fun aborts_if_true_smoke_test_fails() {
        abort 1
    }
    spec fun aborts_if_true_smoke_test_fails {
        pragma always_aborts_test = true;
    }

    /// More complicated example where smoke test should catch an error
    public fun aborts_if_true_smoke_test_fails_2(): u64 {
        let x: u64;
        x = 1;
        if (x > 0) {
            abort 1
        };
        x
    }
    spec fun aborts_if_true_smoke_test_fails_2 {
        pragma always_aborts_test = true;
    }
}
