module M {
    fun t(s: signer): signer {
        copy s
    }
}
// check: COPYLOC_RESOURCE_ERROR
