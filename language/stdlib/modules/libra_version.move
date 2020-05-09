address 0x0 {

module LibraVersion {
    use 0x0::LibraConfig;
    use 0x0::Signer;
    use 0x0::Transaction;

    struct T {
        major: u64,
    }

    public fun initialize(account: &signer) {
        Transaction::assert(Signer::address_of(account) == LibraConfig::default_config_address(), 1);

        LibraConfig::publish_new_config<Self::T>(
            T { major: 1 },
            account,
        );
    }

    public fun set(major: u64, account: &signer) {
        let old_config = LibraConfig::get<Self::T>();

        Transaction::assert(
            old_config.major < major,
            25
        );

        LibraConfig::set<Self::T>(
            T { major },
            account
        );
    }
}

}
