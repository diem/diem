// Test that Option functions abort when they should

script {
use 0x0::Option;

fun main() {
    let _ = Option::borrow(&Option::none<u64>());
}
}

// check: NATIVE_FUNCTION_ERROR


//! new-transaction
script {
use 0x0::Option;

fun main() {
    let _ = Option::borrow_mut(&mut Option::none<u64>());
}
}

// check: NATIVE_FUNCTION_ERROR


//! new-transaction
script {
use 0x0::Option;

fun main() {
    let _ = Option::extract(&mut Option::none<u64>());
}
}

// check: NATIVE_FUNCTION_ERROR


//! new-transaction
script {
use 0x0::Option;

fun main() {
    let _ = Option::destroy_some(Option::none<u64>());
}
}

// check: NATIVE_FUNCTION_ERROR


//! new-transaction
script {
use 0x0::Option;

fun main() {
    Option::fill(&mut Option::some(3), 7);
}
}

// check: ABORTED
// check: 99
