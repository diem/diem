
module M {
    resource struct R { f: bool }
    fun t0(s: &signer, a: address) {
        move_to<R>(s, R { f: false }, a);
    }
}
// check: POSITIVE_STACK_SIZE_AT_BLOCK_END

//! new-transaction

module N {
    resource struct R<T> { f: T }
    fun t0<T>(s: &signer, a: address) {
        move_to<R<bool>>(a, s, a, R<bool> { f: false });
    }
}
// check: POSITIVE_STACK_SIZE_AT_BLOCK_END
