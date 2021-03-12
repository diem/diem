
module 0x8675309::M {
    struct R has key { f: bool }
    fun t0(s: &signer, a: address) {
        move_to<R>(s, R { f: false }, a);
    }
}
// check: POSITIVE_STACK_SIZE_AT_BLOCK_END

//! new-transaction

module 0x8675309::N {
    struct R<T> has key { f: T }
    fun t0<T>(s: &signer, a: address) {
        move_to<R<bool>>(a, s, a, R<bool> { f: false });
    }
}
// check: POSITIVE_STACK_SIZE_AT_BLOCK_END
