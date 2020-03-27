module M {
    struct S {}

    // this does not currently produce a warning
    fun unused(i: u64, s: S) {
    }
}
