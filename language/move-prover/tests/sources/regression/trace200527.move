address 0x1 {

// TODO: After changing this test to stop using Transaction::Sender(), it no longer
// reproduces the bug that it was intended to demonstrate. This should be investigated.

module TraceBug {
    resource struct Root { }

    spec module {
        define only_root_addr_has_root_privilege(): bool {
            // With no TRACE, everything is fine.
            //   all(domain<address>(), |addr| exists<Root>(addr) ==> addr == root_address())
            // With one TRACE, everything is fine.
            //   all(domain<address>(), |addr| exists<Root>(addr) ==> TRACE(addr) == root_address())
            // BUG: With two TRACE, verification fails
            forall addr: address: exists<Root>(TRACE(addr)) ==> TRACE(addr) == root_address()
        }
        define root_address(): address { 0xA550C18 }
    }

    public fun assert_sender_is_root(sender: address) {
        // Here we abort if the sender does not have Root privilege.
        assert(exists<Root>(sender), 1001);
    }
    spec fun assert_sender_is_root {
        // The following two conditions usually come from invariants, but we have expanded them here for
        // minimality.
        requires only_root_addr_has_root_privilege();
        ensures only_root_addr_has_root_privilege();

        // Here we state that from the invariant it follows that the sender has root address.
        ensures sender == root_address();
    }
}
}
