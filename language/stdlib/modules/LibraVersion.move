address 0x1 {

module LibraVersion {
    use 0x1::CoreAddresses;
    use 0x1::LibraConfig;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;

    struct LibraVersion {
        major: u64,
    }

    const ENOT_GENESIS: u64 = 0;
    const EINVALID_SINGLETON_ADDRESS: u64 = 1;
    const EINVALID_MAJOR_VERSION_NUMBER: u64 = 2;

    public fun initialize(
        lr_account: &signer,
    ) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);

        LibraConfig::publish_new_config<LibraVersion>(
            lr_account,
            LibraVersion { major: 1 },
        );
    }

    public fun set(account: &signer, major: u64) {
        let old_config = LibraConfig::get<LibraVersion>();

        assert(
            old_config.major < major,
            EINVALID_MAJOR_VERSION_NUMBER
        );

        LibraConfig::set<LibraVersion>(
            account,
            LibraVersion { major }
        );
    }

    spec module {
        /// The permission "UpdateLibraProtocolVersion" is granted to LibraRoot [B20].
        invariant forall addr: address where exists<LibraVersion>(addr):
            addr == CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS();
    }
}

}
