address 0x1 {
module M {
    #[test]
    fun timeout_fail() {
        while (true) {}
    }

    #[test]
    #[expected_failure]
    fun timeout_fail_with_expected_failure() {
        while (true) {}
    }

    #[test]
    fun no_timeout() { }

    #[test]
    fun no_timeout_fail() { abort 0 }

    #[test]
    fun no_timeout_while_loop() {
        let i = 0;
        while (i < 10) {
            i = i + 1;
        };
    }
}
}
