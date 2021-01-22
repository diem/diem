module M {
    fun consume(s: signer) {
        consume(move s)
    }

    fun t(cond: bool, s: signer) {
        if (cond) consume(s)
    }
}

// check: UNSAFE_RET_UNUSED_RESOURCES
