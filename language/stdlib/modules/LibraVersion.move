address 0x1 {

module LibraVersion {
    use 0x1::CoreErrors;
    use 0x1::CoreAddresses;
    use 0x1::LibraConfig::{Self, CreateOnChainConfig};
    use 0x1::Signer;
    use 0x1::Roles::Capability;

    fun MODULE_ERROR_BASE(): u64 { 2000 }
    public fun INVALID_CONFIG_MAJOR_VERSION_BUMP(): u64 { MODULE_ERROR_BASE() + CoreErrors::CORE_ERR_RANGE() + 1 }

    struct LibraVersion {
        major: u64,
    }

    public fun initialize(
        account: &signer,
        create_config_capability: &Capability<CreateOnChainConfig>,
    ) {
        // Operational constraint
        assert(
            Signer::address_of(account) == CoreAddresses::DEFAULT_CONFIG_ADDRESS(),
            MODULE_ERROR_BASE() + CoreErrors::INVALID_SINGLETON_ADDRESS()
        );

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
            INVALID_CONFIG_MAJOR_VERSION_BUMP()
        );

        LibraConfig::set<LibraVersion>(
            account,
            LibraVersion { major }
        );
    }
}

}
