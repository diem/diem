address 0x0:
module LibraConfig {
    use 0x0::Transaction;
    use 0x0::LibraSystem;

    // A generic singleton resource that holds a value of a specific type.
    resource struct T<U: copyable> { payload: U }

    resource struct ReconfigurationCapabilityHolder {
        capability: LibraSystem::ReconfigurationCapability,
    }

    public fun initialize() {
        let sender = Transaction::sender();

        // Only callable by the Association address for now.
        Transaction::assert(sender == 0xA550C18, 1);
        move_to_sender<ReconfigurationCapabilityHolder>(ReconfigurationCapabilityHolder {
            capability: LibraSystem::grant_reconfiguration_capability(),
        })
    }

    // This can only be invoked by the Association address, and only a single time.
    public fun set<U: copyable>(payload: U) acquires T, ReconfigurationCapabilityHolder {
        let sender = Transaction::sender();

        // Only callable by the Association address for now.
        Transaction::assert(sender == 0xA550C18, 1);

        if(::exists<T<U>>(sender)) {
            let config = borrow_global_mut<T<U>>(sender);
            config.payload = payload;
        } else {
            move_to_sender(T{ payload: payload });
        };
        LibraSystem::reconfigure_with_capability(
            &borrow_global<ReconfigurationCapabilityHolder>(sender).capability
        );
    }
}