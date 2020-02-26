use 0x0::FixedPoint32;

fun main() {
    // The maximum value is 2^32 - 1. Check that anything larger aborts
    // with an overflow.
    let f1 = FixedPoint32::create_from_rational(4294967296, 1); // 2^32
    // The above should fail at runtime so that the following assertion
    // is never even tested.
    0x0::Transaction::assert(FixedPoint32::get_raw_value(f1) == 999, 1);
}

// check: ABORTED
// check: 16
