address 0x1 {
module LibraConfig {
    use 0x1::CoreAddresses;
    use 0x1::Event;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
    // TODO: Can't use aliasing because spec functions won't import
    use 0x1::Roles;

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
    // Currently, the capability is only published on Libra root.
    resource struct ModifyConfigCapability<TypeName> {}

    const ENOT_GENESIS: u64 = 0;
    const ENOT_LIBRA_ROOT: u64 = 1;
    const EINVALID_SINGLETON_ADDRESS: u64 = 2;
    const ECONFIG_DOES_NOT_EXIST: u64 = 3;
    const EMODIFY_CAPABILITY_NOT_HELD: u64 = 4;
    const ENO_CONFIG_PRIVILEGE: u64 = 5;
    const EINVALID_BLOCK_TIME: u64 = 6;

    // This can only be invoked by the config address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize(
        config_account: &signer,
    ) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        // Operational constraint
        assert(Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);
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
    public fun get<Config: copyable>(): Config
    acquires LibraConfig {
        let addr = CoreAddresses::LIBRA_ROOT_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), ECONFIG_DOES_NOT_EXIST);
        *&borrow_global<LibraConfig<Config>>(addr).payload
    }

    // Set a config item to a new value with the default capability stored under config address and trigger a
    // reconfiguration.
    public fun set<Config: copyable>(account: &signer, payload: Config)
    acquires LibraConfig, Configuration {
        let addr = CoreAddresses::LIBRA_ROOT_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), ECONFIG_DOES_NOT_EXIST);
        let signer_address = Signer::address_of(account);
        assert(exists<ModifyConfigCapability<Config>>(signer_address), EMODIFY_CAPABILITY_NOT_HELD);

        let config = borrow_global_mut<LibraConfig<Config>>(addr);
        config.payload = payload;

        reconfigure_();
    }

    // Publish a new config item. Only the config address can modify such config.
    public fun publish_new_config<Config: copyable>(
        config_account: &signer,
        payload: Config
    ) {
        assert(Roles::has_on_chain_config_privilege(config_account), ENO_CONFIG_PRIVILEGE);
        assert(Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);
        move_to(config_account, ModifyConfigCapability<Config> {});
        move_to(config_account, LibraConfig{ payload });
        // We don't trigger reconfiguration here, instead we'll wait for all validators update the binary
        // to register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction to change
        // the value which triggers the reconfiguration.
    }

    // Publish a new config item. Only the delegated address can modify such config after redeeming the capability.

    public fun reconfigure(
        lr_account: &signer,
    ) acquires Configuration {
        // Only callable by libra root account or by the VM internally.
        assert(Roles::has_libra_root_role(lr_account), ENOT_LIBRA_ROOT);
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
       assert(current_block_time > config_ref.last_reconfiguration_time, EINVALID_BLOCK_TIME);
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

        define spec_has_modify_config_capability<Config>(): bool {
            exists<ModifyConfigCapability<Config>>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS())
        }

        define spec_has_on_chain_config_privilege(account: signer): bool {
            Roles::spec_has_libra_root_role(account)
        }
    }

    // check spec_is_published
    spec fun publish_new_config {
        /// TODO(wrwg): enable
        /// aborts_if spec_is_published<Config>();
        ensures old(!spec_is_published<Config>());
        ensures spec_is_published<Config>();
    }

}
}
