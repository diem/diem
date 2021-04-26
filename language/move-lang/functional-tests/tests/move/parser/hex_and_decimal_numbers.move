script {
    fun main() {
        assert(15u8 == 0xFu8, 42);
        assert(15u8 == 0x0Fu8, 42);
        assert(255u8 == 0xFFu8, 42);
        assert(255u8 == 0x0FFu8, 42);

        assert(15u64 == 0xFu64, 42);
        assert(15u64 == 0x0Fu64, 42);
        assert(255u64 == 0xFFu64, 42);
        assert(255u64 == 0x0FFu64, 42);
        assert(18446744073709551615u64 == 0xFFFFFFFFFFFFFFFFu64, 42);
        assert(18446744073709551615u64 == 0x0FFFFFFFFFFFFFFFFu64, 42);

        assert(15u128 == 0xFu128, 42);
        assert(15u128 == 0x0Fu128, 42);
        assert(255u128 == 0xFFu128, 42);
        assert(255u128 == 0x0FFu128, 42);
        assert(18446744073709551615u128 == 0xFFFFFFFFFFFFFFFFu128, 42);
        assert(18446744073709551615u128 == 0x0FFFFFFFFFFFFFFFFu128, 42);
        assert(
            340282366920938463463374607431768211455u128 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128,
            42,
        );
        assert(
            340282366920938463463374607431768211455u128 == 0x0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128,
            42,
        );
    }
}
