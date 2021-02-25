module M {
    // Check misspelling of "copy" constraint.
    struct S<T: copyable> { }
}
