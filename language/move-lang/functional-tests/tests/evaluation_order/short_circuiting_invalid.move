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
// check:"ABORTED { code: 42,"

//! new-transaction
script {
use {{default}}::X;
fun main() {
    true && X::error();
}
}
// check:"ABORTED { code: 42,"

//! new-transaction
script {
use {{default}}::X;
fun main() {
    X::error() && false;
}
}
// check:"ABORTED { code: 42,"

//! new-transaction
script {
use {{default}}::X;
fun main() {
    X::error() || true;
}
}
// check:"ABORTED { code: 42,"
