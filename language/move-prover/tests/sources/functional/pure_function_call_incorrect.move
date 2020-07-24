module TestPureFun {
    use 0x1::CoreAddresses;
    use 0x1::Signer;

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
        // error: calling impure function `init` is disallowed.
        aborts_if !init(account);
    }

    spec module {
        define lr_x(): u64 {
            get_x(CoreAddresses::LIBRA_ROOT_ADDRESS())
        }
    }

}
