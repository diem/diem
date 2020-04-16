// dep: tests/sources/stdlib/modules/libra_configs.move
// dep: tests/sources/stdlib/modules/transaction.move
// dep: tests/sources/stdlib/modules/libra_system.move
// dep: tests/sources/stdlib/modules/libra_account.move
// dep: tests/sources/stdlib/modules/hash.move
// dep: tests/sources/stdlib/modules/lbr.move
// dep: tests/sources/stdlib/modules/lcs.move
// dep: tests/sources/stdlib/modules/libra.move
// dep: tests/sources/stdlib/modules/libra_transaction_timeout.move
// dep: tests/sources/stdlib/modules/vector.move
// dep: tests/sources/stdlib/modules/libra_time.move
// dep: tests/sources/stdlib/modules/validator_config.move
// no-verify

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
