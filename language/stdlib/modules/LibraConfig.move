address 0x1 {
module LibraConfig {
    use 0x1::CoreAddresses;
    use 0x1::Event;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
    use 0x1::Association;
    use 0x1::Offer;

    // A generic singleton resource that holds a value of a specific type.
    resource struct LibraConfig<Config: copyable> { payload: Config }

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
    public fun initialize(config_account: &signer, association_account: &signer) {
        assert(Signer::address_of(config_account) == CoreAddresses::DEFAULT_CONFIG_ADDRESS(), 1);
        Association::grant_privilege<CreateConfigCapability>(association_account, config_account);
        Association::grant_privilege<CreateConfigCapability>(association_account, association_account);


        move_to<Configuration>(
            config_account,
            Configuration {
                epoch: 0,
                last_reconfiguration_time: 0,
                events: Event::new_event_handle<NewEpochEvent>(config_account),
            }
        );
    }

    // Get a copy of `Config` value stored under `addr`.
    public fun get<Config: copyable>(): Config acquires LibraConfig {
        let addr = CoreAddresses::DEFAULT_CONFIG_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), 24);
        *&borrow_global<LibraConfig<Config>>(addr).payload
    }

    // Set a config item to a new value with the default capability stored under config address and trigger a
    // reconfiguration.
    public fun set<Config: copyable>(account: &signer, payload: Config) acquires LibraConfig, Configuration {
        let addr = CoreAddresses::DEFAULT_CONFIG_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), 24);
        let signer_address = Signer::address_of(account);
        assert(
            exists<ModifyConfigCapability<Config>>(signer_address)
            || Association::addr_is_association(signer_address),
            24
        );

        let config = borrow_global_mut<LibraConfig<Config>>(addr);
        config.payload = payload;

        reconfigure_();
    }

    // Set a config item to a new value and trigger a reconfiguration.
    public fun set_with_capability<Config: copyable>(
        _cap: &ModifyConfigCapability<Config>,
        payload: Config
    ) acquires LibraConfig, Configuration {
        let addr = CoreAddresses::DEFAULT_CONFIG_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), 24);
        let config = borrow_global_mut<LibraConfig<Config>>(addr);
        config.payload = payload;

        reconfigure_();
    }

    // Publish a new config item. The caller will use the returned ModifyConfigCapability to specify the access control
    // policy for who can modify the config.
    public fun publish_new_config_with_capability<Config: copyable>(
        config_account: &signer,
        payload: Config,
    ): ModifyConfigCapability<Config> {
        assert(
            Association::has_privilege<CreateConfigCapability>(Signer::address_of(config_account)),
            1
        );

        move_to(config_account, LibraConfig { payload });
        // We don't trigger reconfiguration here, instead we'll wait for all validators update the binary
        // to register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction to change
        // the value which triggers the reconfiguration.

        return ModifyConfigCapability<Config> {}
    }

    // publish config and give capability only to TC account
    public fun publish_new_treasury_compliance_config<Config: copyable>(
        config_account: &signer,
        tc_account: &signer,
        payload: Config,
    ) {
        assert(
            Association::has_privilege<CreateConfigCapability>(Signer::address_of(config_account)),
            1
        );
        move_to(config_account, LibraConfig { payload });
        move_to(tc_account, ModifyConfigCapability<Config> {});
    }

    // Publish a new config item. Only the config address can modify such config.
    public fun publish_new_config<Config: copyable>(config_account: &signer, payload: Config) {
        assert(
            Association::has_privilege<CreateConfigCapability>(Signer::address_of(config_account)),
            1
        );

        move_to(config_account, ModifyConfigCapability<Config> {});
        move_to(config_account, LibraConfig{ payload });
        // We don't trigger reconfiguration here, instead we'll wait for all validators update the binary
        // to register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction to change
        // the value which triggers the reconfiguration.
    }

    // Publish a new config item. Only the delegated address can modify such config after redeeming the capability.
    public fun publish_new_config_with_delegate<Config: copyable>(
        config_account: &signer,
        payload: Config,
        delegate: address,
    ) {
        assert(
            Association::has_privilege<CreateConfigCapability>(Signer::address_of(config_account)),
            1
        );

        Offer::create(config_account, ModifyConfigCapability<Config>{}, delegate);
        move_to(config_account, LibraConfig { payload });
        // We don't trigger reconfiguration here, instead we'll wait for all validators update the
        // binary to register this config into ON_CHAIN_CONFIG_REGISTRY then send another
        // transaction to change the value which triggers the reconfiguration.
    }

    // Claim a delegated modify config capability granted by publish_new_config_with_delegate.
    public fun claim_delegated_modify_config<Config>(account: &signer, offer_address: address) {
        move_to(account, Offer::redeem<ModifyConfigCapability<Config>>(account, offer_address))
    }

    public fun reconfigure(account: &signer) acquires Configuration {
        // Only callable by association address or by the VM internally.
        assert(
            Association::has_privilege<Self::CreateConfigCapability>(Signer::address_of(account)),
            1
        );
        reconfigure_();
    }

    fun reconfigure_() acquires Configuration {
       // Do not do anything if time is not set up yet, this is to avoid genesis emit too many epochs.
       if (LibraTimestamp::is_genesis()) {
           return ()
       };

       let config_ref = borrow_global_mut<Configuration>(CoreAddresses::DEFAULT_CONFIG_ADDRESS());

       // Ensure that there is at most one reconfiguration per transaction. This ensures that there is a 1-1
       // correspondence between system reconfigurations and emitted ReconfigurationEvents.

       let current_block_time = LibraTimestamp::now_microseconds();
       assert(current_block_time > config_ref.last_reconfiguration_time, 23);
       config_ref.last_reconfiguration_time = current_block_time;

       emit_reconfiguration_event();
    }

    // Emit a reconfiguration event. This function will be invoked by the genesis directly to generate the very first
    // reconfiguration event.
    fun emit_reconfiguration_event() acquires Configuration {
        let config_ref = borrow_global_mut<Configuration>(CoreAddresses::DEFAULT_CONFIG_ADDRESS());
        config_ref.epoch = config_ref.epoch + 1;

        Event::emit_event<NewEpochEvent>(
            &mut config_ref.events,
            NewEpochEvent {
                epoch: config_ref.epoch,
            },
        );
    }

    // **************** Specifications ****************

    spec module {

        /// Specifications of LibraConfig are very incomplete.  There are just a few
        /// definitions that are used by RegisteredCurrencies

        pragma verify = true;

        // spec_get is the spec version of get<Config>
        define spec_get<Config>(): Config {
            global<LibraConfig<Config>>(0xF1A95).payload
        }

        define spec_is_published<Config>(addr: address): bool {
            exists<LibraConfig<Config>>(addr)
        }
    }

    // check spec_is_published
    spec fun publish_new_config {
        // aborts_if spec_is_published<Config>();
        ensures old(!spec_is_published<Config>(Signer::get_address(config_account)));
        ensures spec_is_published<Config>(Signer::get_address(config_account));
    }

    spec fun publish_new_config_with_capability {
        // aborts_if spec_is_published<Config>();
        ensures old(!spec_is_published<Config>(Signer::get_address(config_account)));
        ensures spec_is_published<Config>(Signer::get_address(config_account));
    }


}
}
