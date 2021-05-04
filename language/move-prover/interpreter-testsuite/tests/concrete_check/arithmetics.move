module 0x2::A {
    #[test]
    public fun add_ok(): u8 {
        let a = 1 + 2;
        a
    }

    #[test, expected_failure]
    public fun add_overflow(): u8 {
        let a = 255 + 1;
        a
    }

    #[test]
    public fun sub_ok(): u64 {
        let a = 1 - 1;
        a
    }

    #[test, expected_failure]
    public fun sub_underflow(): u64 {
        let a = 1 - 2;
        a
    }

    #[test]
    public fun mul_ok(): u8 {
        let a = 2 * 3;
        a
    }

    #[test, expected_failure]
    public fun mul_overflow(): u8 {
        let a = 16 * 16;
        a
    }

    #[test]
    public fun div_ok(): u128 {
        let a = 5 / 2;
        a
    }

    #[test, expected_failure]
    public fun div_by_zero(): u128 {
        let a = 5 / 0;
        a
    }

    #[test]
    public fun mod_ok(): u128 {
        let a = 5 % 2;
        a
    }

    #[test, expected_failure]
    public fun mod_by_zero(): u128 {
        let a = 5 % 0;
        a
    }
}
