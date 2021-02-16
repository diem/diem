// NOTE: given that we don't have multi-module publishing yet, we have to go through the
// republishing flow (i.e., module N) to keep the loader happy --- the loader expects module N
// to exist when loading module M.

module N {
    fun foo() {}
}

//! new-transaction
module M {
    use {{default}}::N;
    friend N;
    public(friend) fun foo() {}
}

//! new-transaction
module N {
    use {{default}}::M;
    fun foo() { M::foo() }
}
