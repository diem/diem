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

    // This is a regular comment which appears where a doc comment would not be allowed.
}
