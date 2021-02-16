module {{default}}::A {
    public fun foo() {}
}

//! new-transaction
module {{default}}::B {
    use {{default}}::A;
    public fun foo() { A::foo() }
}

//! new-transaction
module {{default}}::C {
    use {{default}}::A;
    use {{default}}::B;
    friend A;
    public fun foo() { B::foo() }
}
// check: INVALID_FRIEND_DECL_WITH_MODULES_IN_DEPENDENCIES
