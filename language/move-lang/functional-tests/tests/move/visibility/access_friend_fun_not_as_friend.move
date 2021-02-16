module {{default}}::M {
    public(friend) fun foo() {}
}

//! new-transaction
module {{default}}::N {
    use {{default}}::M;
    fun foo() { M::foo() }
}
