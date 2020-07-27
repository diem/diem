address 0x1 {
module LibraConfig {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::Event;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
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

    const ECONFIGURATION: u64 = 0;
    const ELIBRA_CONFIG: u64 = 1;
    const EMODIFY_CAPABILITY: u64 = 2;
    const ECONFIG_DOES_NOT_EXIST: u64 = 3;
    const EINVALID_BLOCK_TIME: u64 = 4;

    // This can only be invoked by the config address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize(
        config_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        CoreAddresses::assert_libra_root(config_account);
        assert(!exists<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::already_published(ECONFIGURATION));
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
        assert(exists<LibraConfig<Config>>(addr), Errors::not_published(ELIBRA_CONFIG));
        *&borrow_global<LibraConfig<Config>>(addr).payload
    }
    spec fun get {
        pragma opaque;
        include AbortsIfNotPublished<Config>;
        ensures result == spec_get<Config>();
    }
    spec schema AbortsIfNotPublished<Config> {
        aborts_if !exists<LibraConfig<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS()) with Errors::NOT_PUBLISHED;
    }

    // Set a config item to a new value with the default capability stored under config address and trigger a
    // reconfiguration.
    public fun set<Config: copyable>(account: &signer, payload: Config)
    acquires LibraConfig, Configuration {
        let signer_address = Signer::address_of(account);
        assert(exists<ModifyConfigCapability<Config>>(signer_address), Errors::requires_privilege(EMODIFY_CAPABILITY));

        let addr = CoreAddresses::LIBRA_ROOT_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), Errors::not_published(ELIBRA_CONFIG));
        let config = borrow_global_mut<LibraConfig<Config>>(addr);
        config.payload = payload;

        reconfigure_();
    }
    spec fun set {
        include AbortsIfNotModifiable<Config>;
    }
    spec schema AbortsIfNotModifiable<Config> {
        account: signer;
        include AbortsIfNotPublished<Config>;
        aborts_if !exists<ModifyConfigCapability<Config>>(Signer::spec_address_of(account))
            with Errors::REQUIRES_PRIVILEGE;
    }

    // Set a config item to a new value and trigger a reconfiguration.
    public fun set_with_capability_and_reconfigure<Config: copyable>(
        _cap: &ModifyConfigCapability<Config>,
        payload: Config
    ) acquires LibraConfig, Configuration {
        let addr = CoreAddresses::LIBRA_ROOT_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), Errors::not_published(ELIBRA_CONFIG));
        let config = borrow_global_mut<LibraConfig<Config>>(addr);
        config.payload = payload;
        reconfigure_();
    }

    // Publish a new config item.
    // The caller will use the returned ModifyConfigCapability to specify the access control
    // policy for who can modify the config.
    // Does not trigger a reconfiguration.
    public fun publish_new_config_and_get_capability<Config: copyable>(
        config_account: &signer,
        payload: Config,
    ): ModifyConfigCapability<Config> {
        LibraTimestamp::assert_genesis();
        Roles::assert_libra_root(config_account);
        assert(
            !exists<LibraConfig<Config>>(Signer::address_of(config_account)),
            Errors::already_published(ELIBRA_CONFIG)
        );
        move_to(config_account, LibraConfig { payload });
        ModifyConfigCapability<Config> {}
    }

    // Publish a new config item. Only the config address can modify such config.
    // Does not trigger a reconfiguration. Will also publish the capability to
    // modify this config under the given account.
    public fun publish_new_config<Config: copyable>(
        config_account: &signer,
        payload: Config
    ) {
        let capability = publish_new_config_and_get_capability<Config>(config_account, payload);
        assert(
            !exists<ModifyConfigCapability<Config>>(Signer::address_of(config_account)),
            Errors::already_published(EMODIFY_CAPABILITY)
        );
        move_to(config_account, capability);
    }

    // Publish a new config item. Only the delegated address can modify such config after redeeming the capability.
    public fun reconfigure(
        lr_account: &signer,
    ) acquires Configuration {
        Roles::assert_libra_root(lr_account);
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
       assert(current_block_time > config_ref.last_reconfiguration_time, Errors::invalid_state(EINVALID_BLOCK_TIME));
       config_ref.last_reconfiguration_time = current_block_time;

       emit_reconfiguration_event();
    }
    spec fun reconfigure_ {
        /// The effect of this function is currently excluded from verification.
        /// > TODO: still specify this function using the `[concrete]` property so it can be locally verified.
        pragma opaque, verify = false;
        aborts_if false;
    }

    // Emit a reconfiguration event. This function will be invoked by the genesis directly to generate the very first
    // reconfiguration event.
    fun emit_reconfiguration_event() acquires Configuration {
        assert(exists<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::not_published(ECONFIGURATION));
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

        /// TODO: Specifications of LibraConfig are very incomplete.
        pragma verify = true;

        /// Configurations are only stored at the libra root address.
        invariant
            forall config_address: address, config_type: type where exists<LibraConfig<config_type>>(config_address):
                config_address == CoreAddresses::LIBRA_ROOT_ADDRESS();

        /// After genesis, no new configurations are added.
        invariant update [global]
            LibraTimestamp::is_operating() ==>
                (forall config_type: type
                 where old(!exists<LibraConfig<config_type>>(CoreAddresses::LIBRA_ROOT_ADDRESS())):
                     !exists<LibraConfig<config_type>>(CoreAddresses::LIBRA_ROOT_ADDRESS()));

        define spec_has_config(): bool {
            exists<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS())
        }

        /// Spec version of `LibraConfig::get<Config>`.
        define spec_get<Config>(): Config {
            global<LibraConfig<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS()).payload
        }

        /// Spec version of `LibraConfig::is_published<Config>`.
        define spec_is_published<Config>(): bool {
            exists<LibraConfig<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS())
        }

    }

    spec fun publish_new_config {
        include PublishNewConfigAbortsIf<Config>;
        include PublishNewConfigEnsures<Config>;
    }
    spec schema PublishNewConfigAbortsIf<Config> {
        config_account: signer;
        include LibraTimestamp::AbortsIfNotGenesis;
        include Roles::AbortsIfNotLibraRoot{account: config_account};
        aborts_if spec_is_published<Config>();
        aborts_if exists<ModifyConfigCapability<Config>>(Signer::spec_address_of(config_account));
    }
    spec schema PublishNewConfigEnsures<Config> {
        config_account: signer;
         ensures spec_is_published<Config>();
         ensures exists<ModifyConfigCapability<Config>>(Signer::spec_address_of(config_account));
    }

}
}
