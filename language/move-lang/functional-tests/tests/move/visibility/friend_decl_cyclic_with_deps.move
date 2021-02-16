module {{default}}::N {
    public fun foo() {}
}

//! new-transaction
module {{default}}::M {
    use {{default}}::N;
    friend N;
    public fun foo() { N::foo() }
}
