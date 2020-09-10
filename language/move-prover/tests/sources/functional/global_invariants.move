module TestGlobalInvariants {
    use 0x1::Signer;

    spec module {
        pragma verify = true;
    }

    resource struct R {
        x: u64
    }

    resource struct S {
        x: u64
    }

    spec module {
        // Whenever the one resource exists at address, the other one must also exist.
        invariant [global] forall a: address where exists<R>(a): exists<S>(a);

        invariant update [global] forall a: address where old(exists_R(a)): exists<R>(a);

        // Use a spec function to test whether the right memory is accessed.
        define exists_R(addr: address): bool {
            exists<R>(addr)
        }
    }


    public fun create_R(account: &signer) {
        move_to<S>(account, S{x: 0});
        move_to<R>(account, R{x: 0});
    }

    public fun create_R_invalid(account: &signer) {
        // We cannot create an R without having an S.
        move_to<R>(account, R{x: 0});
    }

    public fun get_S_x(account: &signer): u64 acquires S {
        assert(exists<R>(Signer::address_of(account)), 0);
        borrow_global<S>(Signer::address_of(account)).x
    }
    spec fun get_S_x {
        // We do not need the aborts for exists<S> because exists<R> implies this.
        aborts_if !exists<R>(Signer::spec_address_of(account));
        ensures result == global<S>(Signer::spec_address_of(account)).x;
    }

    public fun remove_S_invalid(account: &signer) acquires S {
        // We cannot remove an S if there is an R.
        assert(exists<R>(Signer::address_of(account)), 0);
        let S{x:_} = move_from<S>(Signer::address_of(account));
    }
    spec fun remove_S_invalid {
        aborts_if !exists<R>(Signer::spec_address_of(account));
    }

    public fun remove_R_invalid(account: &signer) acquires R {
        // We cannot remove an R because of the update invariant.
        assert(exists<R>(Signer::address_of(account)), 0);
        let R{x:_} = move_from<R>(Signer::address_of(account));
    }
}
