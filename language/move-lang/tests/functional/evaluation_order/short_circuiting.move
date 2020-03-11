module X {
    public fun error(): bool {
        abort 42
    }
}

//! new-transaction
use {{default}}::X;
fun main() {
    let vtrue = true;
    let vfalse = false;

    true || X::error();
    vtrue || X::error();
    vtrue || { let r = X::error(); r };
    { let x = vtrue; x} || X::error();

    false && X::error();
    vfalse && X::error();
    vfalse && { let r = X::error(); r };
    { let x = vfalse; x} && X::error();
}
