script {
use 0x0::FixedPoint32;

fun main() {
    let f1 = FixedPoint32::create_from_rational(3, 2); // 1.5
    // Multiply the maximum u64 value by 1.5. This should overflow.
    let overflow = FixedPoint32::multiply_u64(18446744073709551615, copy f1);
    // The above should fail at runtime so that the following assertion
    // is never even tested.
    0x0::Transaction::assert(overflow == 999, 1);
}
}
// check: ARITHMETIC_ERROR
