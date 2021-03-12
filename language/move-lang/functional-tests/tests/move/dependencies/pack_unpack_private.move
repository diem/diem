module {{default}}::C {
    struct T {}
}

//! new-transaction
module {{default}}::B {
    use {{default}}::C;
    public fun foo(): C::T {
        C::T {}
    }
    public fun bar(c: C::T) {
        let C::T {} = c;
    }
}
