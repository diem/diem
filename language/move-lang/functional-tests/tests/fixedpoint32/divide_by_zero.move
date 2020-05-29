script {
use 0x0::FixedPoint32;

fun main() {
    // Dividing by zero should cause an arithmetic error.
    let f1 = FixedPoint32::create_from_raw_value(0);
    let fail = FixedPoint32::divide_u64(1, copy f1);
    // The above should fail at runtime so that the following assertion
    // is never even tested.
    0x0::Transaction::assert(fail == 999, 1);
}
}
// check: ARITHMETIC_ERROR
