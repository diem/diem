module 0x8675309::M {
    struct S<T> has copy, drop { s: T }
    fun t(s: signer): S<signer> {
        let x = S<signer> { s };
        copy x
    }
}

// check: COPYLOC_RESOURCE_ERROR
