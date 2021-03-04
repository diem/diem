module M {
    fun t(s: &signer): signer {
        *s
    }
}

// check: READREF_RESOURCE_ERROR
