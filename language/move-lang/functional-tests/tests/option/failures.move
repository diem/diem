// Test that Option functions abort when they should

script {
use 0x1::Option;

fun main() {
    let _ = Option::borrow(&Option::none<u64>());
}
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED 1


//! new-transaction
script {
use 0x1::Option;

fun main() {
    let _ = Option::borrow_mut(&mut Option::none<u64>());
}
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED 1

//! new-transaction
script {
use 0x1::Option;

fun main() {
    let _ = Option::extract(&mut Option::none<u64>());
}
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED 1


//! new-transaction
script {
use 0x1::Option;

fun main() {
    let _ = Option::destroy_some(Option::none<u64>());
}
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED 1


//! new-transaction
script {
use 0x1::Option;

fun main() {
    Option::fill(&mut Option::some(3), 7);
}
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 0
