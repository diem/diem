module 0x8675309::M {
    fun t(s: &signer): signer {
        *s
    }
}
// check: READREF_RESOURCE_ERROR
