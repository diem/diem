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
module LibraConfig {
    use 0x0::Transaction;
    use 0x0::LibraSystem;

    // A generic singleton resource that holds a value of a specific type.
    resource struct T<Config: copyable> { payload: Config }

    // Set a config item to a new value and trigger a reconfiguration.
    // This can only be invoked by the Association address
    public fun set<Config: copyable>(payload: Config) acquires T {
        let sender = Transaction::sender();

        // Only callable by the Association address for now.
        Transaction::assert(sender == 0xA550C18, 1);

        Transaction::assert(::exists<T<Config>>(sender), 24);
        let config = borrow_global_mut<T<Config>>(sender);
        config.payload = payload;

        LibraSystem::reconfigure();
    }

    // Publish a new config item to a new value and trigger a reconfiguration.
    public fun publish_new_config<Config: copyable>(payload: Config) {
        // Only callable by the Association address for now.
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);

        move_to_sender(T{ payload });
        LibraSystem::reconfigure();
    }
}
