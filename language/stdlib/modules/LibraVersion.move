address 0x1 {

module LibraVersion {
    use 0x1::CoreAddresses;
    use 0x1::LibraConfig::{Self, CreateOnChainConfig};
    use 0x1::Signer;
    use 0x1::Roles::Capability;

    struct LibraVersion {
        major: u64,
    }

    public fun initialize(
        account: &signer,
        create_config_capability: &Capability<CreateOnChainConfig>,
    ) {
        assert(Signer::address_of(account) == CoreAddresses::DEFAULT_CONFIG_ADDRESS(), 1);

        LibraConfig::publish_new_config<LibraVersion>(
            account,
            create_config_capability,
            LibraVersion { major: 1 },
        );
    }

    public fun set(account: &signer, major: u64) {
        let old_config = LibraConfig::get<LibraVersion>();

        assert(
            old_config.major < major,
            25
        );

        LibraConfig::set<LibraVersion>(
            account,
            LibraVersion { major }
        );
    }
}

}
