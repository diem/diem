module 0x8675309::M {
    fun t(s: signer): signer {
        copy s
    }
}
// check: COPYLOC_RESOURCE_ERROR
