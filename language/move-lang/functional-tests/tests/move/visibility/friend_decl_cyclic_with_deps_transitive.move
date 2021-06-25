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
    // TODO cyclic dep not caught by interface generator
    friend A;
    public fun foo() { B::foo() }
}
