address 0x0:
module LibraConfig {
    use 0x0::Transaction;
    use 0x0::LibraAccount;
    use 0x0::LibraTimestamp;

    // A generic singleton resource that holds a value of a specific type.
    resource struct T<Config: copyable> { payload: Config }

    struct NewEpochEvent {
        epoch: u64,
    }

    resource struct Configuration {
        epoch: u64,
        last_reconfiguration_time: u64,
        events: LibraAccount::EventHandle<NewEpochEvent>,
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
            events: LibraAccount::new_event_handle<NewEpochEvent>(),
        });
    }

    // We can't return reference here, this returns a copy of the config value.
    public fun get<Config: copyable>(): Config acquires T {
        let sender = Transaction::sender();

        Transaction::assert(::exists<T<Config>>(sender), 24);
        let t_ref = borrow_global<T<Config>>(sender);

        *&t_ref.payload
    }

    // Set a config item to a new value and trigger a reconfiguration.
    // This can only be invoked by the Association address
    public fun set<Config: copyable>(payload: Config) acquires T, Configuration {
        let sender = Transaction::sender();

        Transaction::assert(::exists<T<Config>>(sender), 24);
        let config = borrow_global_mut<T<Config>>(sender);
        config.payload = payload;

        reconfigure();
    }

    // Publish a new config item to a new value and trigger a reconfiguration.
    public fun publish_new_config<Config: copyable>(payload: Config) acquires Configuration {
        move_to_sender(T{ payload });
        reconfigure();
    }

    public fun reconfigure() acquires Configuration {
        // TODO: Transform this method to user capability pattern.
        // Only callable by the Association address for now.
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);

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

       LibraAccount::emit_event<NewEpochEvent>(
           &mut config_ref.events,
           NewEpochEvent {
               epoch: config_ref.epoch,
           },
       );
   }
}
