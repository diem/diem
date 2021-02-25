module M {
    fun t(s1: signer, s2: signer): signer {
        s1 = s2;
        s1
    }
}
