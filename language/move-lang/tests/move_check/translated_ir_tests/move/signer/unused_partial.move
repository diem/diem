module 0x8675309::M {
    fun consume(s: signer) {
        consume(move s)
    }

    fun t(cond: bool, s: signer) {
        if (cond) consume(s)
    }
}
