address 0x42 {
module X {
    struct S {}
}
module M {
    use 0x42::X;
    use 0x42::X::S as X;

    struct R {}
    struct S<T> { f: T }


    fun t() {
        // Assign to qualified name
        // should fail in non spec context
        X::S = ();
        Self::S<u64> = ();
        Self::R = ();

        // Assign to fully qualified name
        // should fail in non spec context
        0x42::X::S = ();
        0x42::M::S<u64> = ();
        0x42::M::R = ();

        // Assign to name with type args, qualified/nonqualified/struct doesnt matter
        // should fail in non spec context
        x<u64> = ();
        S<u64> = ();

        // Assign to a name that is aliased in local context
        // should fail in non spec context
        X = ();
        S = ();
        R = ();

        // Assign to a name that starts with A-Z
        // Should fail with unbound local even though it is not a valid local name
        Y = 0;
    }
}
}
