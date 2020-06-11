address 0x0 {

module LibraVersion {
    use 0x0::CoreAddresses;
    use 0x0::LibraConfig;
    use 0x0::Signer;
    use 0x0::Transaction;

    struct LibraVersion {
        major: u64,
    }

    public fun initialize(account: &signer) {
        Transaction::assert(Signer::address_of(account) == CoreAddresses::DEFAULT_CONFIG_ADDRESS(), 1);

        LibraConfig::publish_new_config<LibraVersion>(
            account,
            LibraVersion { major: 1 },
        );
    }

    public fun set(account: &signer, major: u64) {
        let old_config = LibraConfig::get<LibraVersion>();

        Transaction::assert(
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
