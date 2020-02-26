use 0x0::FixedPoint32;

fun main() {
    let f1 = FixedPoint32::create_from_raw_value(18446744073709551615);
    // Multiply 2^33 by the maximum fixed-point value. This should overflow.
    let overflow = FixedPoint32::multiply_u64(8589934592, copy f1);
    // The above should fail at runtime so that the following assertion
    // is never even tested.
    0x0::Transaction::assert(overflow == 999, 1);
}

// check: ABORTED
// check: 16
