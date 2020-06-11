address 0x1 {

module Testnet {
    use 0x1::CoreAddresses;
    use 0x1::Signer;

    resource struct IsTestnet { }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), 0);
        move_to(account, IsTestnet{})
    }

    public fun is_testnet(): bool {
        exists<IsTestnet>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS())
    }

    // only used for testing purposes
    public fun remove_testnet(account: &signer)
    acquires IsTestnet {
        assert(Signer::address_of(account) == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), 0);
        IsTestnet{} = move_from<IsTestnet>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
    }

    // **************** SPECIFICATIONS ****************

    /**
       This module is used by Genesis, LibraAccount and Unhosted as part of
       the initialization and to check whether the transaction is on the
       testnet.
    **/

    /// # Module Specification

    spec module {
        /// Verify all functions in this module, including private ones.
        pragma verify = true;

        /// Returns the association root address.
        define spec_root_address(): address { 0xA550C18 }

        /// Returns true if the testnet has been initialized.
        /// Initialization can be reversed by function `remove_testnet`.
        define spec_is_initialized(): bool {
            exists<IsTestnet>(spec_root_address())
        }
    }

    /// ## Management of TestNet

    spec module {
        /// Returns true if no address has IsTestNet resource.
        define spec_no_addr_has_testnet(): bool {
            all(domain<address>(), |addr| !exists<IsTestnet>(addr))
        }

        /// Returns true if only the root address has an IsTestNet resource.
        define spec_only_root_addr_has_testnet(): bool {
            all(domain<address>(), |addr|
                exists<IsTestnet>(addr)
                    ==> addr == spec_root_address())
        }
    }

    spec schema OnlyRootAddressHasTestNet {
        /// Base case of the induction before the association address is
        /// initialized with IsTestnet.
        invariant module !spec_is_initialized()
                            ==> spec_no_addr_has_testnet();

        /// Inductive step after the initialization is complete.
        invariant module spec_is_initialized()
                            ==> spec_only_root_addr_has_testnet();
    }

    spec schema TestNetStaysInitialized {
        ensures old(spec_is_initialized()) ==> spec_is_initialized();
    }

    spec module {
        /// Apply 'OnlyRootAddressHasTestNet' to all functions.
        apply OnlyRootAddressHasTestNet to *;

        /// Apply 'TestNetStaysInitialized' to all functions except
        /// `remove_testnet`
        apply TestNetStaysInitialized to * except remove_testnet;
    }


    /// ## Specifications for individual functions

    spec module {
        /// Returns true if IsTestNet exists at the address of the account.
        define spec_exists_testnet(account: signer): bool {
            exists<IsTestnet>(Signer::get_address(account))
        }
    }

    spec fun initialize {
        aborts_if Signer::get_address(account) != spec_root_address();
        aborts_if spec_exists_testnet(account);
        ensures spec_exists_testnet(account);
    }

    spec fun is_testnet {
        aborts_if false;
        ensures result == exists<IsTestnet>(spec_root_address());
    }

    spec fun remove_testnet {
        aborts_if Signer::get_address(account) != spec_root_address();
        aborts_if !spec_exists_testnet(account);
        ensures !spec_exists_testnet(account);
    }
}
}
