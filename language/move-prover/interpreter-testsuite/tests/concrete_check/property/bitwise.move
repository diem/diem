module 0x2::A {
    #[test]
    public fun check_bitwise_ok() {
        spec {
            assert 0x1u8 & 0x2u64 == 0u128;
            assert 0x1u128 | 0x2u64 == 0x3u8;
            assert 0x1u8 << 8 == 0x100;
        };
    }

    #[test]
    public fun check_bitwise_fail() {
        spec {
            assert 0x1u128 ^ 0x2 != 0x3u64;
            assert 0x100u64 >> 16 != 0;
        };
    }
}
