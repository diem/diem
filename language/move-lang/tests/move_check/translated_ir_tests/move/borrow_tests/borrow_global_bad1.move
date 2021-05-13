module 0x8675309::A {
    use Std::Signer;
    struct T has key {v: u64}

    public fun A0(account: &signer, addr: address) acquires T {
        let sender = Signer::address_of(account);
        let x = borrow_global_mut<T>(sender);
        let y = borrow_global_mut<T>(addr);
        x;
        y;
    }
}

// check: GLOBAL_REFERENCE_ERROR
