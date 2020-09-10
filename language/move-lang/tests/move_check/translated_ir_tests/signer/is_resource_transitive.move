module M {
    struct S<T> {
        f: T,
    }

    fun t(s: signer) {
        let x = S<signer> { f: s };
    }
}

// check: UNSAFE_RET_UNUSED_RESOURCES
