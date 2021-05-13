module 0x8675309::A {
    use Std::Signer;
    struct T1 has key {v: u64}
    struct T2 has key {v: u64}

    public fun test1(account: &signer) acquires T1, T2 {
        let x = borrow_global_mut<T1>(Signer::address_of(account));
        let y = get_v(x);
        acquires_t1(account);
        acquires_t2(account);
        move y;
    }

    public fun test2(account: &signer) acquires T1, T2 {
        let x = borrow_global_mut<T1>(Signer::address_of(account));
        let y = get_v(x);
        acquires_t2(account);
        acquires_t1(account);
        move y;
    }

    public fun test3(account: &signer) acquires T1, T2 {
        let x = borrow_global_mut<T1>(Signer::address_of(account));
        let y = get_v(x);
        acquires_t1(account);
        move y;
        acquires_t2(account);
    }

    fun get_v(x: &mut T1): &mut u64 {
        &mut x.v
    }

    fun acquires_t1(account: &signer) acquires T1 {
        T1 { v: _ } = move_from<T1>(Signer::address_of(account))
    }

    fun acquires_t2(account: &signer) acquires T2 {
        T2 { v: _ } = move_from<T2>(Signer::address_of(account))
    }

}
