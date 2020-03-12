address 0x0:

module ScriptWhitelist {
    use 0x0::LibraConfig;

    struct T { payload: vector<u8> }

    public fun initialize(payload: vector<u8>) {
        LibraConfig::publish_new_config<Self::T>(T { payload })
    }

    public fun set(payload: vector<u8>) {
        LibraConfig::set<Self::T>(T { payload } )
    }
}
