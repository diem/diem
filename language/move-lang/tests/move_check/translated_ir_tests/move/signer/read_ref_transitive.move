module M {
    struct S<T> has copy, drop { s: T }
    fun t(s: signer): S<signer> {
        let x = S<signer> { s };
        *&x
    }
}
// check: READREF_RESOURCE_ERROR

//! new-transaction
module N {
    struct S<T> has copy, drop { s: T }
    fun t(s: signer): signer {
        let x = S<signer> { s };
        x.s
    }
}
// check: READREF_RESOURCE_ERROR
