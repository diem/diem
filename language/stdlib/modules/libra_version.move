address 0x0:

module LibraVersion {
    use 0x0::LibraConfig;
    use 0x0::Transaction;

    struct T {
        major: u64,
    }

    public fun initialize() {

        LibraConfig::publish_new_config<Self::T>(T {
            major: 1,
        })
    }

    public fun set(major: u64) {
        let old_config = LibraConfig::read_config<Self::T>();

        Transaction::assert(
            old_config.major < major,
            25
        );

        LibraConfig::set<Self::T>(T { major } )
    }
}
