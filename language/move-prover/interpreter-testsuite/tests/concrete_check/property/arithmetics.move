module 0x2::A {
    #[test]
    public fun check_arithmetics_ok() {
        spec {
            // there is no overflow/underflow in spec expressions
            assert 255u8 + 1u64 == 256u128;
            assert 1u8 - 2u64 < 0;
            assert 255u8 * 255u8 == 255u128 * 255u128;
            assert 5 / 2 == 2;
            assert 5 % 2 == 1;
        };
    }

    #[test]
    public fun check_arithmetics_div0() {
        spec {
            assert 5 / 0 == 1;
        };
    }

    #[test]
    public fun check_arithmetics_mod0() {
        spec {
            assert 5 % 0 == 1;
        };
    }
}
