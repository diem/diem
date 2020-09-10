module M {
    struct S<T> { s: T }
    fun t(s: signer): S<signer> {
        let x = S<signer> { s };
        copy x
    }
}

// check: COPYLOC_RESOURCE_ERROR
