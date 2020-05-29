script {
use 0x0::FixedPoint32;

fun main() {
    // The minimum non-zero value is 2^-32. Check that anything smaller
    // aborts.
    let f1 = FixedPoint32::create_from_rational(1, 8589934592); // 2^-33
    // The above should fail at runtime so that the following assertion
    // is never even tested.
    0x0::Transaction::assert(FixedPoint32::get_raw_value(f1) == 999, 1);
}
}
// check: ABORTED
// check: 16
