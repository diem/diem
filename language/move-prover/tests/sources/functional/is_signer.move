// separate_baseline: cvc4
// separate_baseline: no_opaque
// The separate baseline is legit and caused by a different choice in the generated model.
module 0x42::SimpleIsSigner {
    use Std::Signer;
    use DiemFramework::Roles;

    // ---------------
    // Simple examples
    // ---------------

    public fun f1_incorrect() {
        spec { assert Signer::is_signer(@0x7); } // This is unprovable because it is not true in general.
    }

    public fun f2_incorrect(_account: &signer) {
        spec { assert Signer::is_signer(@0x7); } // This is unprovable because it is not true in general.
    }

    public fun f3(account: &signer) {
        assert(Signer::address_of(account) == @0x7, 1);
        spec { assert Signer::is_signer(@0x7); } // It's true in general.
    }

    public fun f4_incorrect(account: &signer) {
        assert(Signer::address_of(account) == @0x3, 1);
        spec { assert Signer::is_signer(@0x7); } // This is unprovable because it is not true in general.
    }

    fun f5() {
        spec { assert Signer::is_signer(@0x7); } // This is provable because of the "requires" condition.
    }
    spec f5 {
        requires Signer::is_signer(@0x7); // f5 requires this to be true at its callers' sites
    }

    public fun f6(account: &signer) { // This function produces no error because it satisfies f5's requires condition.
        assert(Signer::address_of(account) == @0x7, 1);
        f5();
    }

    public fun f7_incorrect() {
        // This function does not satisfy f5's requires condition.
        f5();
    }


    // ------------------------------
    // Simple access control examples
    // ------------------------------

    spec fun hasPermissionAddr(addr: address): bool {
        Roles::spec_has_diem_root_role_addr(addr)
    }
    fun hasPermission(account: &signer): bool {
        Roles::has_diem_root_role(account)
    }

    public fun g_incorrect(_a: &signer, _b: &signer) {
        spec {
            assert exists addr:address: hasPermissionAddr(addr);
        };
        // privileged operation
    }

    public fun g1(a: &signer, _b: &signer) {
        if(hasPermission(a))
        {
            spec {
                assert exists addr:address: (Signer::is_signer(addr) && hasPermissionAddr(addr));
            };
            // privileged operation
        }
    }

    public fun g2(a: &signer, _b: &signer) {
        assert (hasPermission(a), 1);
        spec {
            assert exists addr:address: (Signer::is_signer(addr) && hasPermissionAddr(addr));
        };
        // privileged operation
    }

    public fun g3(a: &signer, _b: &signer) {
        assert (hasPermission(a), 1);
        helper()
    }

    public fun helper() {
        spec {
            assert exists addr:address: (Signer::is_signer(addr) && hasPermissionAddr(addr));
        };
        // privileged operation
    }
    spec helper {
        requires exists addr:address: (Signer::is_signer(addr) && hasPermissionAddr(addr));
    }


    // -----------------------------------
    // Access control for Counter resource
    // -----------------------------------

    struct Counter has key { i: u64 }

    public fun ADMIN_ADDRESS(): address {
        @0x7
    }

    const AUTH_FAILED: u64 = 1;

    public fun publish(account: &signer) {
        spec { assume Signer::is_signer(Signer::spec_address_of(account)); };

        assert(Signer::address_of(account) == ADMIN_ADDRESS(), AUTH_FAILED);
        move_to(account, Counter { i: 0 });
    }

    public fun get_counter(): u64 acquires Counter {
        borrow_global<Counter>(ADMIN_ADDRESS()).i
    }

    public fun increment(account: &signer) acquires Counter {
        assert(Signer::address_of(account) == ADMIN_ADDRESS(), AUTH_FAILED); // permission check
        let c_ref = &mut borrow_global_mut<Counter>(ADMIN_ADDRESS()).i;
        *c_ref = *c_ref + 1;
    }

    // This function is incorrect because it omits the permission check.
    public fun increment_incorrect(_account: &signer) acquires Counter {
        let c_ref = &mut borrow_global_mut<Counter>(ADMIN_ADDRESS()).i;
        *c_ref = *c_ref + 1;
    }

    spec module {
        // Access control spec: Only the admin is allowed to increment the counter value.
        invariant update (old(exists<Counter>(ADMIN_ADDRESS())) && global<Counter>(ADMIN_ADDRESS()).i != old(global<Counter>(ADMIN_ADDRESS()).i))
            ==> Signer::is_signer(ADMIN_ADDRESS());
    }
}
