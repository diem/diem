address 0x0:

module LibraVersion {
    use 0x0::LibraConfig;
    use 0x0::Sender;
    use 0x0::Transaction;

    struct T {
        major: u64,
    }

    public fun initialize(sender: &Sender::T) {
        LibraConfig::publish_new_config<Self::T>(
            T { major: 1 },
            sender
        )
    }

    public fun set(major: u64, sender: &Sender::T) {
        let sender_address = Sender::address_(sender);
        let old_config = LibraConfig::get<Self::T>(sender_address);

        Transaction::assert(
            old_config.major < major,
            25
        );

        LibraConfig::set<Self::T>(sender_address, T { major } )
    }
}
