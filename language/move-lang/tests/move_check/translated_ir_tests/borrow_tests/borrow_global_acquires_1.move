module A {
    use 0x0::Signer;
    resource struct T1 {v: u64}

    public fun test(account: &signer) acquires T1 {
        borrow_global_mut<T1>(Signer::address_of(account));
        acquires_t1(account);
    }

    fun acquires_t1(account: &signer) acquires T1 {
        T1 { v: _ } = move_from<T1>(Signer::address_of(account));
    }

}
