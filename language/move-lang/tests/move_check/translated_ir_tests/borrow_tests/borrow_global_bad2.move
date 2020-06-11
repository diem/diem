module A {
    use 0x0::Signer;
    resource struct T {v: u64}

    public fun A2(account: &signer) acquires T {
        let sender = Signer::address_of(account);
        let t_ref = borrow_global_mut<T>(sender);
        T { v: _ } = move_from<T>(sender);
        t_ref;
    }
}

// check: GLOBAL_REFERENCE_ERROR
