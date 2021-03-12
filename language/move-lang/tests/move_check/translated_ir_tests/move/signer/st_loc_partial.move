module 0x8675309::M {
    fun consume(s: signer) {
        consume(move s)
    }

    fun t(cond: bool, s1: signer, s2: signer) {
        if (cond) consume(s1);
        s1 = s2;
        consume(s1);
    }
}

// check: STLOC_UNSAFE_TO_DESTROY_ERROR
