module A {
    use 0x1::Signer;
    resource struct T1 {v: u64}

    public fun test(account: &signer) {
        borrow_global_mut<T1>(Signer::address_of(account));
    }

}

// check: MISSING_ACQUIRES_RESOURCE_ANNOTATION_ERROR
