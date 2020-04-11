module M {
    struct S {}

    // this produces unused parameter warnings for i and s, but not unused resource warnings
    fun unused(i: u64, s: S) {
    }
}
