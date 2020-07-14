address 0x1 {
module ChainId {
    use 0x1::CoreAddresses;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;

    resource struct ChainId {
        id: u8
    }

    const ENOT_GENESIS: u64 = 0;
    const ENOT_LIBRA_ROOT: u64 = 1;

    /// Publish the chain ID `id` of this Libra instance under the LibraRoot account
    public fun initialize(lr_account: &signer, id: u8) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(
            Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(),
            ENOT_LIBRA_ROOT
        );

        move_to(lr_account, ChainId { id })
    }

    /// Return the chain ID of this Libra instance
    public fun get(): u8 acquires ChainId {
        borrow_global<ChainId>(CoreAddresses::LIBRA_ROOT_ADDRESS()).id
    }
}
}
