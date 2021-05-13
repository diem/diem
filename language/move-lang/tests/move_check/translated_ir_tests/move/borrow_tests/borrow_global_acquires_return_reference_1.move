module 0x8675309::A {
    use Std::Signer;
    struct T1 has key {v: u64}

    public fun test1(account: &signer, x: &mut T1) acquires T1 {
        // the acquire of t1 does not pollute y
        let y = borrow_acquires_t1(account, x);
        acquires_t1(account);
        move y;
    }

    fun borrow_acquires_t1(account: &signer, x: &mut T1): &mut u64 acquires T1 {
        borrow_global_mut<T1>(Signer::address_of(account));
        &mut x.v
    }

    fun acquires_t1(account: &signer) acquires T1 {
        T1 { v:_ } = move_from<T1>(Signer::address_of(account))
    }

}
