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
// TODO(status_migration) remove duplicate check
// check:"ABORTED { code: 42,"
// check:"ABORTED { code: 42,"

//! new-transaction
script {
use {{default}}::X;
fun main() {
    true && X::error();
}
}
// TODO(status_migration) remove duplicate check
// check:"ABORTED { code: 42,"
// check:"ABORTED { code: 42,"

//! new-transaction
script {
use {{default}}::X;
fun main() {
    X::error() && false;
}
}
// TODO(status_migration) remove duplicate check
// check:"ABORTED { code: 42,"
// check:"ABORTED { code: 42,"

//! new-transaction
script {
use {{default}}::X;
fun main() {
    X::error() || true;
}
}
// TODO(status_migration) remove duplicate check
// check:"ABORTED { code: 42,"
// check:"ABORTED { code: 42,"

//! new-transaction
script {
fun main() {
    false || { abort 0 };
}
}
// TODO(status_migration) remove duplicate check
// check:"ABORTED { code: 0,"
// check:"ABORTED { code: 0,"
