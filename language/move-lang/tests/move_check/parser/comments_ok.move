/// This is a test.
module M {
    /**
     * One can have /* nested */
     * // block comments
     */
    fun f() { }

    /* This is a nested /* regular comment // */ */
    fun g() {}

    // This is a line comment which contains unbalanced /* delimiter.
    fun h() {}

    // Comments in strings are not comments at all.
    fun str(): vector<u8> {
        b"http://libra.org"
    }

    // This is a regular comment which appears where a doc comment would not be allowed.
}
