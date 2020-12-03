address 0x1 {

module GenericBug {
    use 0x1::Signer;

    resource struct PrivilegedCapability<Privilege> { }

    struct T { }

    public fun initialize(sender: &signer) {
        assert(Signer::address_of(sender) == root_address(), 1000);
        move_to(sender, PrivilegedCapability<T>{ });
    }

    // Publish a specific privilege under the sending account.
    public fun apply_for_privilege<Privilege>(sender: &signer) {
        if (exists<PrivilegedCapability<Privilege>>(Signer::address_of(sender))) return;
        move_to(sender, PrivilegedCapability<Privilege>{ });
    }

    // Remove the `Privilege` from the address at `addr`. The sender must
    // be the diem root account.
    public fun remove_privilege<Privilege>(sender: &signer, addr: address)
    acquires PrivilegedCapability {
        assert(Signer::address_of(sender) == root_address(), 1001);
        //assert(exists<PrivilegedCapability<Privilege>>(addr), 1004);
        PrivilegedCapability<Privilege>{ } = move_from<PrivilegedCapability<Privilege>>(addr);
    }

    // **FIXED** BUG: Without this, the prover is [was] happy.
    // With this function, it reports an invariant violation.
    // However, the invariant violation is there in any case, because
    // remove_privilege<T>(sender, addr) is defined.
    //public  fun stupid_root(sender: &signer) acquires PrivilegedCapability {
    //     remove_privilege<T>(sender, root_address());
    //}

    // The address at which the root account will be published.
    public fun root_address(): address { 0xA550C18 }

    // **************** SPECIFICATIONS ****************
    spec module {} // switch documentation context back to module level

    spec module {
        pragma verify = true;

        // This mirrors the Move function root_address()
        define spec_root_address(): address { 0xA550C18 }

        // mirrors Move addr_is_association
        define spec_addr_is_association(addr: address): bool {
            exists<PrivilegedCapability<T>>(addr)
        }
     }

    // Invariant: Root address is always an association address.
    // SOUNDNESS BUG: Root can remove its own association privilege, by calling
    // remove_privilege<T>(root_address()).  The prover overlooks a violation of the
    // invariant when it verifies public fun remove_privilege<Privilege>(addr: address),
    // possibly because it doesn't realize Privilege can be T?
    // To demo, I added "stupid_root" above, which actually removes the privilege, and that
    // causes a violation if commented in.
    spec schema RootAddressIsAssociationAddress {
        sender: &signer;
        invariant spec_addr_is_association(Signer::spec_address_of(sender));
    }
    spec module {
        apply RootAddressIsAssociationAddress to *<Privilege>, *
            except root_address;
    }
}
}
