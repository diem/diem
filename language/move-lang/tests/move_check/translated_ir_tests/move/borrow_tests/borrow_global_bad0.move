module 0x8675309::A {
    use 0x1::Signer;
    struct T has key {v: u64}

    public fun t0(account: &signer) acquires T {
        let sender = Signer::address_of(account);
        let x = borrow_global_mut<T>(sender);
        copy x;
        x = borrow_global_mut<T>(sender);
        copy x;
    }
}
