address 0x0 {
module LibraConfig {
    use 0x0::Transaction;
    use 0x0::Event;
    use 0x0::LibraTimestamp;
    use 0x0::Association;
    use 0x0::Offer;

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

    // Accounts with this association privilege can publish new config under default_address
    struct CreateConfigCapability {}

    // Accounts with this privilege can modify config of type TypeName under default_address
    resource struct ModifyConfigCapability<TypeName> {}

    // This can only be invoked by the config address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize_configuration() {
        let sender = Transaction::sender();
        Transaction::assert(sender == default_config_address(), 1);

        move_to_sender<Configuration>(Configuration {
            epoch: 0,
            last_reconfiguration_time: 0,
            events: Event::new_event_handle<NewEpochEvent>(),
        });
    }

    public fun apply_for_creator_privilege() {
        Association::apply_for_association();
        Association::apply_for_privilege<CreateConfigCapability>();
    }

    public fun grant_creator_privilege(addr: address) {
        Association::grant_association_address(addr);
        Association::grant_privilege<CreateConfigCapability>(addr);
    }

    // Get a copy of `Config` value stored under `addr`.
    public fun get<Config: copyable>(): Config acquires T {
        let addr = default_config_address();
        Transaction::assert(::exists<T<Config>>(addr), 24);
        *&borrow_global<T<Config>>(addr).payload
    }

    // Set a config item to a new value with the default capability stored under config address and trigger a
    // reconfiguration.
    public fun set<Config: copyable>(payload: Config) acquires T, Configuration {
        let addr = default_config_address();
        Transaction::assert(::exists<T<Config>>(addr), 24);
        Transaction::assert(
            ::exists<ModifyConfigCapability<Config>>(Transaction::sender())
             || Transaction::sender() == Association::root_address(),
            24
        );

        let config = borrow_global_mut<T<Config>>(addr);
        config.payload = payload;

        reconfigure_();
    }

    // Set a config item to a new value and trigger a reconfiguration.
    public fun set_with_capability<Config: copyable>(
        _cap: &ModifyConfigCapability<Config>,
        payload: Config
    ) acquires T, Configuration {
        let addr = default_config_address();
        Transaction::assert(::exists<T<Config>>(addr), 24);
        let config = borrow_global_mut<T<Config>>(addr);
        config.payload = payload;

        reconfigure_();
    }

    // Publish a new config item. The caller will use the returned ModifyConfigCapability to specify the access control
    // policy for who can modify the config.
    public fun publish_new_config_with_capability<Config: copyable>(payload: Config): ModifyConfigCapability<Config> {
        Transaction::assert(
            Association::has_privilege<CreateConfigCapability>(Transaction::sender()),
            1
        );

        move_to_sender(T{ payload });
        // We don't trigger reconfiguration here, instead we'll wait for all validators update the binary
        // to register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction to change
        // the value which triggers the reconfiguration.

        return ModifyConfigCapability<Config> {}
    }

    // Publish a new config item. Only the config address can modify such config.
    public fun publish_new_config<Config: copyable>(
        payload: Config,
    ) {
        Transaction::assert(
            Association::has_privilege<CreateConfigCapability>(Transaction::sender()),
            1
        );

        move_to_sender(ModifyConfigCapability<Config> {});
        move_to_sender(T{ payload });
        // We don't trigger reconfiguration here, instead we'll wait for all validators update the binary
        // to register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction to change
        // the value which triggers the reconfiguration.
    }

    // Publish a new config item. Only the delegated address can modify such config after redeeming the capability.
    public fun publish_new_config_with_delegate<Config: copyable>(
        payload: Config,
        delegate: address,
    ) {
        Transaction::assert(
            Association::has_privilege<CreateConfigCapability>(Transaction::sender()),
            1
        );

        Offer::create(ModifyConfigCapability<Config> {}, delegate);
        move_to_sender(T{ payload });
        // We don't trigger reconfiguration here, instead we'll wait for all validators update the binary
        // to register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction to change
        // the value which triggers the reconfiguration.
    }

    // Claim a delegated modify config capability granted by publish_new_config_with_delegate.
    public fun claim_delegated_modify_config<Config>(offer_address: address) {
        move_to_sender(Offer::redeem<ModifyConfigCapability<Config>>(offer_address))
    }

    public fun reconfigure() acquires Configuration {
        // Only callable by association address or by the VM internally.
        Transaction::assert(
            Association::has_privilege<Self::CreateConfigCapability>(Transaction::sender()),
            1
        );
        reconfigure_();
    }

    fun reconfigure_() acquires Configuration {
       // Do not do anything if time is not set up yet, this is to avoid genesis emit too many epochs.
       if(LibraTimestamp::is_genesis()) {
           return ()
       };

       let config_ref = borrow_global_mut<Configuration>(default_config_address());

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
        let config_ref = borrow_global_mut<Configuration>(default_config_address());
        config_ref.epoch = config_ref.epoch + 1;

        Event::emit_event<NewEpochEvent>(
            &mut config_ref.events,
            NewEpochEvent {
                epoch: config_ref.epoch,
            },
        );
    }

    public fun default_config_address(): address {
        0xF1A95
    }
}
}
