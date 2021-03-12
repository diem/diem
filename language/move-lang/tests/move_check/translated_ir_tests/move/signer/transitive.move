module 0x8675309::M {
    struct S<T> has drop {
        f: T,
    }

    fun t(s: signer) {
        let _ = S<signer> { f: s };
    }
}
