use 0x0::FixedPoint32;

fun main() {
    let f1 = FixedPoint32::create_from_rational(3, 4); // 0.75
    let nine = FixedPoint32::multiply_u64(12, copy f1); // 12 * 0.75
    0x0::Transaction::assert(nine == 9, nine);
    let twelve = FixedPoint32::divide_u64(9, copy f1); // 9 / 0.75
    0x0::Transaction::assert(twelve == 12, twelve);

    let f2 = FixedPoint32::create_from_rational(1, 3); // 0.333...
    let not_three = FixedPoint32::multiply_u64(9, copy f2); // 9 * 0.333...
    // multiply_u64 does NOT round -- it truncates -- so values that
    // are not perfectly representable in binary may be off by one.
    0x0::Transaction::assert(not_three == 2, not_three);

    // Try again with a fraction slightly larger than 1/3.
    let f3 = FixedPoint32::create_from_raw_value(FixedPoint32::get_raw_value(copy f2) + 1);
    let three = FixedPoint32::multiply_u64(9, copy f3);
    0x0::Transaction::assert(three == 3, three);

    // Test creating a 1.0 fraction from the maximum u64 value.
    let f4 = FixedPoint32::create_from_rational(18446744073709551615, 18446744073709551615);
    let one = FixedPoint32::get_raw_value(copy f4);
    0x0::Transaction::assert(one == 4294967296, 4); // 0x1.00000000
}
