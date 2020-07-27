address 0x1 {
module ChainId {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;

    resource struct ChainId {
        id: u8
    }

    const ECHAIN_ID: u64 = 0;

    /// Publish the chain ID `id` of this Libra instance under the LibraRoot account
    public fun initialize(lr_account: &signer, id: u8) {
        LibraTimestamp::assert_genesis();
        CoreAddresses::assert_libra_root(lr_account);
        assert(!exists<ChainId>(Signer::address_of(lr_account)), Errors::already_published(ECHAIN_ID));
        move_to(lr_account, ChainId { id })
    }

    /// Return the chain ID of this Libra instance
    public fun get(): u8 acquires ChainId {
        LibraTimestamp::assert_operating();
        borrow_global<ChainId>(CoreAddresses::LIBRA_ROOT_ADDRESS()).id
    }

    spec module {
        invariant [global] LibraTimestamp::is_operating() ==> exists<ChainId>(CoreAddresses::LIBRA_ROOT_ADDRESS());
    }
}
}
