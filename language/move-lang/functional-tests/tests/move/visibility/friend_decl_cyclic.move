module {{default}}::A {
}

//! new-transaction
module {{default}}::B {
    friend {{default}}::A;
}

//! new-transaction
module {{default}}::A {
    friend {{default}}::B;
}
