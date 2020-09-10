module A {
    fun three(): (u64, u64, u64) {
        (0, 1, 2)
    }

    fun pop_correct() {
        three();
        (_, _, _) = three();
    }

    fun pop() {
        three();
        (_, _) = ();
        (_) = ();
    }
}
