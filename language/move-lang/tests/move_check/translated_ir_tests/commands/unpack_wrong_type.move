module Test {
    struct X { b: bool }
    struct T { b: bool }

    public fun destroy_t(t: T) {
        X { b: _ } = t;
    }

}
