script {
use 0x1::FixedPoint32;

fun main() {
    let f1 = FixedPoint32::create_from_raw_value(18446744073709551615);
    // Multiply 2^33 by the maximum fixed-point value. This should overflow.
    let overflow = FixedPoint32::multiply_u64(8589934592, copy f1);
    // The above should fail at runtime so that the following assertion
    // is never even tested.
    assert(overflow == 999, 1);
}
}
// check: "Keep(ABORTED { code: 520"
