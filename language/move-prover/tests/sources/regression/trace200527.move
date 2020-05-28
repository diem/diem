address 0x0 {

module TraceBug {
    use 0x0::Transaction;

    resource struct Root { }

    spec module {
        define only_root_addr_has_root_privilege(): bool {
            // With no TRACE, everything is fine.
            //   all(domain<address>(), |addr| exists<Root>(addr) ==> addr == root_address())
            // With one TRACE, everything is fine.
            //   all(domain<address>(), |addr| exists<Root>(addr) ==> TRACE(addr) == root_address())
            // BUG: With two TRACE, verification fails
            all(domain<address>(), |addr| exists<Root>(TRACE(addr)) ==> TRACE(addr) == root_address())
        }
        define root_address(): address { 0xA550C18 }
    }

    public fun assert_sender_is_root() {
        // Here we abort if the sender does not have Root privilege.
        Transaction::assert(exists<Root>(Transaction::sender()), 1001);
    }
    spec fun assert_sender_is_root {
        // The following two conditions usually come from invariants, but we have expanded them here for
        // minimality.
        requires only_root_addr_has_root_privilege();
        ensures only_root_addr_has_root_privilege();

        // Here we state that from the invariant it follows that the sender has root address.
        ensures sender() == root_address();
    }
}
}
