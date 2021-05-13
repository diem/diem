module 0x8675309::A {
    use Std::Signer;
    struct T has key {v: u64}
    struct U has key {v: u64}

    public fun A0(account: &signer) acquires T {
        let sender = Signer::address_of(account);
        let t_ref = borrow_global_mut<T>(sender);
        move t_ref;
        borrow_global_mut<T>(sender);
    }

    public fun A1(account: &signer) acquires T, U {
        let sender = Signer::address_of(account);
        let t_ref = borrow_global_mut<T>(sender);
        let u_ref = borrow_global_mut<U>(sender);
        t_ref;
        u_ref;
    }

    public fun A2(account: &signer, b: bool) acquires T {
        let sender = Signer::address_of(account);
        let t_ref = if (b) borrow_global_mut<T>(sender) else borrow_global_mut<T>(sender);
        t_ref;
    }

    public fun A3(account: &signer, b: bool) acquires T {
        let sender = Signer::address_of(account);
        if (b) {
            borrow_global_mut<T>(sender);
        }
    }

    public fun A4(account: &signer) acquires T {
        let sender = Signer::address_of(account);
        let x = move_from<T>(sender);
        borrow_global_mut<T>(sender);
        move_to<T>(account, x);
    }

    public fun A5(account: &signer) acquires T {
        let sender = Signer::address_of(account);
        borrow_global<T>(sender);
    }
}
