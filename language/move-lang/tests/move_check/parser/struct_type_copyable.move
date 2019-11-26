module M {
    // Check misspelling of "copyable" constraint.
    struct S<T: copy> { }
}
