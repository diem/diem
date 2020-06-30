address 0x1 {
module LibraConfig {
    use 0x1::CoreAddresses;
    use 0x1::Event;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
    use 0x1::Offer;
    use 0x1::Roles::{Self, Capability, LibraRootRole};

    resource struct CreateOnChainConfig {}

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

    // Accounts with this privilege can modify config of type TypeName under default_address
    resource struct ModifyConfigCapability<TypeName> {}

    /// Will fail if the account is not association root
    public fun grant_privileges(account: &signer) {
        Roles::add_privilege_to_account_association_root_role(account, CreateOnChainConfig{});
    }

    // This can only be invoked by the config address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize(
        config_account: &signer,
        _: &Capability<CreateOnChainConfig>,
    ) {
        // Operational constraint
        assert(Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), 1);
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
        let addr = CoreAddresses::LIBRA_ROOT_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), 24);
        *&borrow_global<LibraConfig<Config>>(addr).payload
    }

    // Set a config item to a new value with the default capability stored under config address and trigger a
    // reconfiguration.
    public fun set<Config: copyable>(account: &signer, payload: Config) acquires LibraConfig, Configuration {
        let addr = CoreAddresses::LIBRA_ROOT_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), 24);
        let signer_address = Signer::address_of(account);
        assert(exists<ModifyConfigCapability<Config>>(signer_address), 24);

        let config = borrow_global_mut<LibraConfig<Config>>(addr);
        config.payload = payload;

        reconfigure_();
    }

    // Set a config item to a new value and trigger a reconfiguration.
    public fun set_with_capability<Config: copyable>(
        _cap: &ModifyConfigCapability<Config>,
        payload: Config
    ) acquires LibraConfig, Configuration {
        let addr = CoreAddresses::LIBRA_ROOT_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), 24);
        let config = borrow_global_mut<LibraConfig<Config>>(addr);
        config.payload = payload;
        reconfigure_();
    }

    // Publish a new config item. The caller will use the returned ModifyConfigCapability to specify the access control
    // policy for who can modify the config.
    public fun publish_new_config_with_capability<Config: copyable>(
        config_account: &signer,
        _: &Capability<CreateOnChainConfig>,
        payload: Config,
    ): ModifyConfigCapability<Config> {
        assert(Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), 1);
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
        _: &Capability<CreateOnChainConfig>,
        payload: Config,
    ) {
        move_to(config_account, LibraConfig { payload });
        move_to(tc_account, ModifyConfigCapability<Config> {});
    }

    // Publish a new config item. Only the config address can modify such config.
    public fun publish_new_config<Config: copyable>(
        config_account: &signer,
        _: &Capability<CreateOnChainConfig>,
        payload: Config
    ) {
        assert(Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), 1);
        move_to(config_account, ModifyConfigCapability<Config> {});
        move_to(config_account, LibraConfig{ payload });
        // We don't trigger reconfiguration here, instead we'll wait for all validators update the binary
        // to register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction to change
        // the value which triggers the reconfiguration.
    }

    // Publish a new config item. Only the delegated address can modify such config after redeeming the capability.
    public fun publish_new_config_with_delegate<Config: copyable>(
        config_account: &signer,
        _: &Capability<CreateOnChainConfig>,
        payload: Config,
        delegate: address,
    ) {
        assert(Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), 1);
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

    public fun reconfigure(
        _: &Capability<LibraRootRole>,
    ) acquires Configuration {
        // Only callable by association address or by the VM internally.
        reconfigure_();
    }

    fun reconfigure_() acquires Configuration {
       // Do not do anything if time is not set up yet, this is to avoid genesis emit too many epochs.
       if (LibraTimestamp::is_not_initialized()) {
           return ()
       };

       let config_ref = borrow_global_mut<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());

       // Ensure that there is at most one reconfiguration per transaction. This ensures that there is a 1-1
       // correspondence between system reconfigurations and emitted ReconfigurationEvents.

       let current_block_time = LibraTimestamp::now_microseconds();
       assert(current_block_time > config_ref.last_reconfiguration_time, 23);
       config_ref.last_reconfiguration_time = current_block_time;

       emit_reconfiguration_event();
    }
    spec fun reconfigure_ {
        /// Consider only states for verification where this function does not abort
        /// for a caller. This prevents that callers need to propagate the abort conditions of this
        /// function up the call chain. The abort conditions of this function represent
        /// internal programming errors.
        ///
        /// > TODO(wrwg): we should have a convention to distinguish error codes resulting from
        /// contract program errors and from errors coming from inputs to transaction
        /// scripts. In most cases, only the later one should be propagated upwards to callers.
        /// For now, we use the pragma below to simulate this.
        pragma assume_no_abort_from_here = true;
    }

    // Emit a reconfiguration event. This function will be invoked by the genesis directly to generate the very first
    // reconfiguration event.
    fun emit_reconfiguration_event() acquires Configuration {
        let config_ref = borrow_global_mut<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
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

        /// > TODO(wrwg): We've removed an invariant in RegisteredCurrencies that config is only stored
        //  > at ROOT_ADDRESS. We should bring this back here for all config types.

        /// Specifications of LibraConfig are very incomplete.  There are just a few
        /// definitions that are used by RegisteredCurrencies

        pragma verify = true;

        define spec_has_config(): bool {
            exists<Configuration>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS())
        }

        /// Spec version of `LibraConfig::get<Config>`.
        define spec_get<Config>(): Config {
            global<LibraConfig<Config>>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS()).payload
        }

        /// Spec version of `LibraConfig::is_published<Config>`.
        define spec_is_published<Config>(): bool {
            exists<LibraConfig<Config>>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS())
        }
    }

    // check spec_is_published
    spec fun publish_new_config {
        /// TODO(wrwg): enable
        /// aborts_if spec_is_published<Config>();
        ensures old(!spec_is_published<Config>());
        ensures spec_is_published<Config>();
    }

    spec fun publish_new_config_with_capability {
        /// TODO(wrwg): enable
        /// aborts_if spec_is_published<Config>();
        ensures old(!spec_is_published<Config>());
        ensures spec_is_published<Config>();
    }

}
}
