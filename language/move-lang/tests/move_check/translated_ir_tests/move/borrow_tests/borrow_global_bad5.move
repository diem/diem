module 0x8675309::A {
    use Std::Signer;
    struct T has key {v: u64}

    public fun A5(account: &signer, b: bool) acquires T {
        let sender = Signer::address_of(account);
        let t = T { v: 0 };
        let t1_ref = if (b) borrow_global_mut<T>(sender) else &mut t;
        let t2_ref = borrow_global_mut<T>(sender);
        t1_ref;
        t2_ref;
        T { v: _ } = t;
    }
}

// check: GLOBAL_REFERENCE_ERROR
