use 0x0::FixedPoint32;

fun main() {
    let f1 = FixedPoint32::create_from_raw_value(1); // 0x0.00000001
    // Divide 2^32 by the minimum fractional value. This should overflow.
    let overflow = FixedPoint32::divide_u64(4294967296, copy f1);
    // The above should fail at runtime so that the following assertion
    // is never even tested.
    0x0::Transaction::assert(overflow == 999, 1);
}

// check: ARITHMETIC_ERROR
