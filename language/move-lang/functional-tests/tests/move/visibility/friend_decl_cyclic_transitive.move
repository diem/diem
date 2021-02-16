module {{default}}::A {
}

//! new-transaction
module {{default}}::B {
    friend {{default}}::A;
}

//! new-transaction
module {{default}}::C {
    friend {{default}}::B;
}

//! new-transaction
module {{default}}::A {
    friend {{default}}::C;
}
