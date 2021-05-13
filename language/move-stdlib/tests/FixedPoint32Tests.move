#[test_only]
module Std::FixedPoint32Tests {
    use Std::FixedPoint32;

    #[test]
    #[expected_failure(abort_code = 7)]
    fun create_div_zero() {
        // A denominator of zero should cause an arithmetic error.
        FixedPoint32::create_from_rational(2, 0);
    }

    #[test]
    #[expected_failure(abort_code = 1032)]
    fun create_overflow() {
        // The maximum value is 2^32 - 1. Check that anything larger aborts
        // with an overflow.
        FixedPoint32::create_from_rational(4294967296, 1); // 2^32
    }

    #[test]
    #[expected_failure(abort_code = 1031)]
    fun create_underflow() {
        // The minimum non-zero value is 2^-32. Check that anything smaller
        // aborts.
        FixedPoint32::create_from_rational(1, 8589934592); // 2^-33
    }

    #[test]
    fun create_zero() {
        let x = FixedPoint32::create_from_rational(0, 1);
        assert(FixedPoint32::is_zero(x), 0);
    }

    #[test]
    #[expected_failure(abort_code = 775)]
    fun divide_by_zero() {
        // Dividing by zero should cause an arithmetic error.
        let f = FixedPoint32::create_from_raw_value(0);
        FixedPoint32::divide_u64(1, f);
    }

    #[test]
    #[expected_failure(abort_code = 264)]
    fun divide_overflow_small_divisore() {
        let f = FixedPoint32::create_from_raw_value(1); // 0x0.00000001
        // Divide 2^32 by the minimum fractional value. This should overflow.
        FixedPoint32::divide_u64(4294967296, f);
    }

    #[test]
    #[expected_failure(abort_code = 264)]
    fun divide_overflow_large_numerator() {
        let f = FixedPoint32::create_from_rational(1, 2); // 0.5
        // Divide the maximum u64 value by 0.5. This should overflow.
        FixedPoint32::divide_u64(18446744073709551615, f);
    }

    #[test]
    #[expected_failure(abort_code = 520)]
    fun multiply_overflow_small_multiplier() {
        let f = FixedPoint32::create_from_rational(3, 2); // 1.5
        // Multiply the maximum u64 value by 1.5. This should overflow.
        FixedPoint32::multiply_u64(18446744073709551615, f);
    }

    #[test]
    #[expected_failure(abort_code = 520)]
    fun multiply_overflow_large_multiplier() {
        let f = FixedPoint32::create_from_raw_value(18446744073709551615);
        // Multiply 2^33 by the maximum fixed-point value. This should overflow.
        FixedPoint32::multiply_u64(8589934592, f);
    }

    #[test]
    fun exact_multiply() {
        let f = FixedPoint32::create_from_rational(3, 4); // 0.75
        let nine = FixedPoint32::multiply_u64(12, f); // 12 * 0.75
        assert(nine == 9, 0);
    }

    #[test]
    fun exact_divide() {
        let f = FixedPoint32::create_from_rational(3, 4); // 0.75
        let twelve = FixedPoint32::divide_u64(9, f); // 9 / 0.75
        assert(twelve == 12, 0);
    }

    #[test]
    fun multiply_truncates() {
        let f = FixedPoint32::create_from_rational(1, 3); // 0.333...
        let not_three = FixedPoint32::multiply_u64(9, copy f); // 9 * 0.333...
        // multiply_u64 does NOT round -- it truncates -- so values that
        // are not perfectly representable in binary may be off by one.
        assert(not_three == 2, 0);

        // Try again with a fraction slightly larger than 1/3.
        let f = FixedPoint32::create_from_raw_value(FixedPoint32::get_raw_value(f) + 1);
        let three = FixedPoint32::multiply_u64(9, f);
        assert(three == 3, 1);
    }

    #[test]
    fun create_from_rational_max_numerator_denominator() {
        // Test creating a 1.0 fraction from the maximum u64 value.
        let f = FixedPoint32::create_from_rational(18446744073709551615, 18446744073709551615);
        let one = FixedPoint32::get_raw_value(f);
        assert(one == 4294967296, 0); // 0x1.00000000
    }
}
