address 0x1 {
/// The chain id distinguishes between different chains (e.g., testnet and the main Libra network).
/// One important role is to prevent transactions intended for one chain from being executed on another.
/// This code provides a container for storing a chain id and functions to initialize and get it.
module ChainId {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;

    resource struct ChainId {
        id: u8
    }

    /// The `ChainId` resource was not in the required state
    const ECHAIN_ID: u64 = 0;

    /// Publish the chain ID `id` of this Libra instance under the LibraRoot account
    public fun initialize(lr_account: &signer, id: u8) {
        LibraTimestamp::assert_genesis();
        CoreAddresses::assert_libra_root(lr_account);
        assert(!exists<ChainId>(Signer::address_of(lr_account)), Errors::already_published(ECHAIN_ID));
        move_to(lr_account, ChainId { id })
    }

    spec fun initialize {
        pragma opaque;
        let lr_addr = Signer::address_of(lr_account);
        modifies global<ChainId>(lr_addr);
        include LibraTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot{account: lr_account};
        aborts_if exists<ChainId>(lr_addr) with Errors::ALREADY_PUBLISHED;
        ensures exists<ChainId>(lr_addr);
    }

    /// Return the chain ID of this Libra instance
    public fun get(): u8 acquires ChainId {
        LibraTimestamp::assert_operating();
        borrow_global<ChainId>(CoreAddresses::LIBRA_ROOT_ADDRESS()).id
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Initialization

    /// When Libra is operating, the chain id is always available.
    spec module {
        invariant [global] LibraTimestamp::is_operating() ==> exists<ChainId>(CoreAddresses::LIBRA_ROOT_ADDRESS());
    }

    /// # Helper Functions

    spec define spec_get_chain_id(): u8 {
        global<ChainId>(CoreAddresses::LIBRA_ROOT_ADDRESS()).id
    }
}
}
