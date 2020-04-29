address 0x0 {
module LibraConfig {
    use 0x0::Transaction;
    use 0x0::Event;
    use 0x0::LibraTimestamp;

    // A generic singleton resource that holds a value of a specific type.
    resource struct T<Config: copyable> { payload: Config }

    struct NewEpochEvent {
        epoch: u64,
    }

    resource struct Configuration {
        epoch: u64,
        last_reconfiguration_time: u64,
        events: Event::EventHandle<NewEpochEvent>,
    }

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize_configuration() {
        let sender = Transaction::sender();
        // Only callable by the Association address for now.
        Transaction::assert(sender == 0xA550C18, 1);

        move_to_sender<Configuration>(Configuration {
            epoch: 0,
            last_reconfiguration_time: 0,
            events: Event::new_event_handle<NewEpochEvent>(),
        });
    }

    // We can't return reference here, this returns a copy of the config value.
    public fun get<Config: copyable>(addr: address): Config acquires T {
        Transaction::assert(::exists<T<Config>>(addr), 24);
        *&borrow_global<T<Config>>(addr).payload
    }

    // Set a config item to a new value and trigger a reconfiguration.
    public fun set<Config: copyable>(addr: address, payload: Config) acquires T, Configuration {
        // TODO: impose proper permission checks
        // Callable by the any address for now.

        Transaction::assert(::exists<T<Config>>(addr), 24);
        let config = borrow_global_mut<T<Config>>(addr);
        config.payload = payload;

        reconfigure_();
    }

    // Publish a new config item to a new value and trigger a reconfiguration.
    public fun publish_new_config<Config: copyable>(payload: Config) {
        // TODO: impose proper permission checks
        // Callable by the any address for now.

        move_to_sender(T{ payload });
        // We don't trigger reconfiguration here, instead we'll wait for all validators update the binary
        // to register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction to change
        // the value which triggers the reconfiguration.
    }

    public fun reconfigure() acquires Configuration {
        // Only callable by association address.
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        reconfigure_();
    }

    fun reconfigure_() acquires Configuration {
       // Do not do anything if time is not set up yet, this is to avoid genesis emit too many epochs.
       if(LibraTimestamp::is_genesis()) {
           return ()
       };

       let config_ref = borrow_global_mut<Configuration>(0xA550C18);

       // Ensure that there is at most one reconfiguration per transaction. This ensures that there is a 1-1
       // correspondence between system reconfigurations and emitted ReconfigurationEvents.

       let current_block_time = LibraTimestamp::now_microseconds();
       Transaction::assert(current_block_time > config_ref.last_reconfiguration_time, 23);
       config_ref.last_reconfiguration_time = current_block_time;

       emit_reconfiguration_event();
    }

    // Emit a reconfiguration event. This function will be invoked by the genesis directly to generate the very first
    // reconfiguration event.
    fun emit_reconfiguration_event() acquires Configuration {
        let config_ref = borrow_global_mut<Configuration>(0xA550C18);
        config_ref.epoch = config_ref.epoch + 1;

        Event::emit_event<NewEpochEvent>(
            &mut config_ref.events,
            NewEpochEvent {
                epoch: config_ref.epoch,
            },
        );
    }
}
}
