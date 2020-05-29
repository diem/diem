module X {
    public fun error(): bool {
        abort 42
    }
}

//! new-transaction
script {
use {{default}}::X;
fun main() {
    false || X::error();
}
}
// check:"major_status: ABORTED, sub_status: Some(42)"

//! new-transaction
script {
use {{default}}::X;
fun main() {
    true && X::error();
}
}
// check:"major_status: ABORTED, sub_status: Some(42)"

//! new-transaction
script {
use {{default}}::X;
fun main() {
    X::error() && false;
}
}
// check:"major_status: ABORTED, sub_status: Some(42)"

//! new-transaction
script {
use {{default}}::X;
fun main() {
    X::error() || true;
}
}
// check:"major_status: ABORTED, sub_status: Some(42)"
