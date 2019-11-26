module M {
    resource struct X {}
    // Test a trailing comma in the acquires list.
    f() acquires X, {
        borrow_global_mut<X>(0x1);
    }
}
