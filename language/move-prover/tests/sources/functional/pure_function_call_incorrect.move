module TestPureFun {
    use 0x1::CoreAddresses;
    use 0x1::Signer;
    use 0x1::Vector;

    resource struct T {
        x: u64,
    }

    public fun init(account: &signer): bool {
        if (exists<T>(Signer::address_of(account))) {
            return true
        };
        move_to(account, T { x: 0 });
        false
    }

    spec fun init {
        ensures get_x(Signer::spec_address_of(account)) == 0;
    }

    public fun get_x(addr: address): u64 acquires T {
        *&borrow_global<T>(addr).x
    }

    public fun increment_x_incorrect(account: &signer) acquires T {
        let t = borrow_global_mut<T>(Signer::address_of(account));
        t.x = t.x + 1;
    }

    spec fun increment_x_incorrect {
        // error: calling impure function `init` is not allowed.
        aborts_if !init(account);
    }

    /// impure function
    public fun impure_f_0(): u64 {
        if (true) { abort 42 };
        0
    }

    public fun impure_f_1(): u64 {
        impure_f_0() + 1
    }

    /// pure-looking function which indirectly calls an impure function
    public fun impure_f_2(): u64 {
        impure_f_1() + 1
    }

    public fun remove_elem(v: &mut vector<T>): T {
        Vector::pop_back(v)
    }

    spec fun remove_elem {
        // error: calling impure function `pop_back` is not allowed.
        ensures result == Vector::pop_back(old(v));
    }

    spec module {
        define two(): u64 {
            // error: calling impure function `impure_f_2` is not allowed.
            impure_f_2()
        }

        define dr_x(): u64 {
            get_x(CoreAddresses::DIEM_ROOT_ADDRESS())
        }
    }

}
