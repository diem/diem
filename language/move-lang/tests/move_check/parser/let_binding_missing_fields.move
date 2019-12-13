module M {
    struct Generic<T> {
        g: T
    }
    g(g: Generic<u64>) {
        let Generic<u64> = g; // Test a type name with no field bindings
    }
}
