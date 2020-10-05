address 0x1 {

/// Publishes configuration information for validators, and issues reconfiguration events
/// to synchronize configuration changes for the validators.

module LibraConfig {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::Event;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
    use 0x1::Roles;

    /// A generic singleton resource that holds a value of a specific type.
    resource struct LibraConfig<Config: copyable> {
        /// Holds specific info for instance of `Config` type.
        payload: Config
    }

    /// Event that signals LibraBFT algorithm to start a new epoch,
    /// with new configuration information. This is also called a
    /// "reconfiguration event"
    struct NewEpochEvent {
        epoch: u64,
    }

    /// Holds information about state of reconfiguration
    resource struct Configuration {
        /// Epoch number
        epoch: u64,
        /// Time of last reconfiguration. Only changes on reconfiguration events.
        last_reconfiguration_time: u64,
        /// Event handle for reconfiguration events
        events: Event::EventHandle<NewEpochEvent>,
    }

    /// Accounts with this privilege can modify LibraConfig<TypeName> under Libra root address.
    resource struct ModifyConfigCapability<TypeName> {}

    /// The `Configuration` resource is in an invalid state
    const ECONFIGURATION: u64 = 0;
    /// A `LibraConfig` resource is in an invalid state
    const ELIBRA_CONFIG: u64 = 1;
    /// A `ModifyConfigCapability` is in a different state than was expected
    const EMODIFY_CAPABILITY: u64 = 2;
    /// An invalid block time was encountered.
    const EINVALID_BLOCK_TIME: u64 = 4;
    /// The largest possible u64 value
    const MAX_U64: u64 = 18446744073709551615;

    /// Publishes `Configuration` resource. Can only be invoked by Libra root, and only a single time in Genesis.
    public fun initialize(
        lr_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        CoreAddresses::assert_libra_root(lr_account);
        assert(!exists<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::already_published(ECONFIGURATION));
        move_to<Configuration>(
            lr_account,
            Configuration {
                epoch: 0,
                last_reconfiguration_time: 0,
                events: Event::new_event_handle<NewEpochEvent>(lr_account),
            }
        );
    }
    spec fun initialize {
        pragma opaque;
        include InitializeAbortsIf;
        include InitializeEnsures;
        modifies global<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
    }
    spec schema InitializeAbortsIf {
        lr_account: signer;
        include LibraTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot{account: lr_account};
        aborts_if spec_has_config() with Errors::ALREADY_PUBLISHED;
    }
    spec schema InitializeEnsures {
        ensures spec_has_config();
        let new_config = global<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        ensures new_config.epoch == 0;
        ensures new_config.last_reconfiguration_time == 0;
    }


    /// Returns a copy of `Config` value stored under `addr`.
    public fun get<Config: copyable>(): Config
    acquires LibraConfig {
        let addr = CoreAddresses::LIBRA_ROOT_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), Errors::not_published(ELIBRA_CONFIG));
        *&borrow_global<LibraConfig<Config>>(addr).payload
    }
    spec fun get {
        pragma opaque;
        include AbortsIfNotPublished<Config>;
        ensures result == get<Config>();
    }
    spec schema AbortsIfNotPublished<Config> {
        aborts_if !exists<LibraConfig<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS()) with Errors::NOT_PUBLISHED;
    }

    /// Set a config item to a new value with the default capability stored under config address and trigger a
    /// reconfiguration. This function requires that the signer be Libra root.
    public fun set<Config: copyable>(account: &signer, payload: Config)
    acquires LibraConfig, Configuration {
        let signer_address = Signer::address_of(account);
        // Next should always be true if properly initialized.
        assert(exists<ModifyConfigCapability<Config>>(signer_address), Errors::requires_capability(EMODIFY_CAPABILITY));

        let addr = CoreAddresses::LIBRA_ROOT_ADDRESS();
        assert(exists<LibraConfig<Config>>(addr), Errors::not_published(ELIBRA_CONFIG));
        let config = borrow_global_mut<LibraConfig<Config>>(addr);
        config.payload = payload;

        reconfigure_();
    }
    spec fun set {
        pragma opaque;
        include SetAbortsIf<Config>;
        include SetEnsures<Config>;
    }
    spec schema SetAbortsIf<Config> {
        account: signer;
        include AbortsIfNotModifiable<Config>;
        include AbortsIfNotPublished<Config>;
        include ReconfigureAbortsIf;
    }
    spec schema AbortsIfNotModifiable<Config> {
        account: signer;
        aborts_if !exists<ModifyConfigCapability<Config>>(Signer::spec_address_of(account))
            with Errors::REQUIRES_CAPABILITY;
    }
    spec schema SetEnsures<Config> {
        payload: Config;
        ensures spec_is_published<Config>();
        ensures get<Config>() == payload;
    }

    /// Set a config item to a new value and trigger a reconfiguration. This function
    /// requires a reference to a `ModifyConfigCapability`, which is returned when the
    /// config is published using `publish_new_config_and_get_capability`.
    /// It is called by `LibraSystem::update_config_and_reconfigure`, which allows
    /// validator operators to change the validator set.  All other config changes require
    /// a Libra root signer.
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
    spec fun set_with_capability_and_reconfigure {
        pragma opaque;
        modifies global<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include AbortsIfNotPublished<Config>;
        include ReconfigureAbortsIf;
        modifies global<LibraConfig<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include SetEnsures<Config>;
    }

    /// Publishes a new config.
    /// The caller will use the returned ModifyConfigCapability to specify the access control
    /// policy for who can modify the config.
    /// Does not trigger a reconfiguration.
    public fun publish_new_config_and_get_capability<Config: copyable>(
        lr_account: &signer,
        payload: Config,
    ): ModifyConfigCapability<Config> {
        LibraTimestamp::assert_genesis();
        Roles::assert_libra_root(lr_account);
        assert(
            !exists<LibraConfig<Config>>(Signer::address_of(lr_account)),
            Errors::already_published(ELIBRA_CONFIG)
        );
        move_to(lr_account, LibraConfig { payload });
        ModifyConfigCapability<Config> {}
    }
    spec fun publish_new_config_and_get_capability {
        pragma opaque;
        modifies global<LibraConfig<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include LibraTimestamp::AbortsIfNotGenesis;
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        include AbortsIfPublished<Config>;
        include SetEnsures<Config>;
    }
    spec schema AbortsIfPublished<Config> {
        aborts_if exists<LibraConfig<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS()) with Errors::ALREADY_PUBLISHED;
    }

    /// Publish a new config item. Only Libra root can modify such config.
    /// Publishes the capability to modify this config under the Libra root account.
    /// Does not trigger a reconfiguration.
    public fun publish_new_config<Config: copyable>(
        lr_account: &signer,
        payload: Config
    ) {
        let capability = publish_new_config_and_get_capability<Config>(lr_account, payload);
        assert(
            !exists<ModifyConfigCapability<Config>>(Signer::address_of(lr_account)),
            Errors::already_published(EMODIFY_CAPABILITY)
        );
        move_to(lr_account, capability);
    }
    spec fun publish_new_config {
        pragma opaque;
        modifies global<LibraConfig<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        modifies global<ModifyConfigCapability<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include PublishNewConfigAbortsIf<Config>;
        include PublishNewConfigEnsures<Config>;
    }
    spec schema PublishNewConfigAbortsIf<Config> {
        lr_account: signer;
        include LibraTimestamp::AbortsIfNotGenesis;
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        aborts_if spec_is_published<Config>();
        aborts_if exists<ModifyConfigCapability<Config>>(Signer::spec_address_of(lr_account));
    }
    spec schema PublishNewConfigEnsures<Config> {
        lr_account: signer;
        payload: Config;
        include SetEnsures<Config>;
        ensures exists<ModifyConfigCapability<Config>>(Signer::spec_address_of(lr_account));
    }

    /// Signal validators to start using new configuration. Must be called by Libra root.
    public fun reconfigure(
        lr_account: &signer,
    ) acquires Configuration {
        Roles::assert_libra_root(lr_account);
        reconfigure_();
    }
    spec fun reconfigure {
        pragma opaque;
        modifies global<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        include ReconfigureAbortsIf;
    }

    /// Private function to do reconfiguration.  Updates reconfiguration status resource
    /// `Configuration` and emits a `NewEpochEvent`
    fun reconfigure_() acquires Configuration {
        // Do not do anything if genesis has not finished.
        if (LibraTimestamp::is_genesis() || LibraTimestamp::now_microseconds() == 0) {
            return ()
        };

        let config_ref = borrow_global_mut<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        let current_time = LibraTimestamp::now_microseconds();
        assert(current_time > config_ref.last_reconfiguration_time, Errors::invalid_state(EINVALID_BLOCK_TIME));
        config_ref.last_reconfiguration_time = current_time;
        config_ref.epoch = config_ref.epoch + 1;

        Event::emit_event<NewEpochEvent>(
            &mut config_ref.events,
            NewEpochEvent {
                epoch: config_ref.epoch,
            },
        );
    }
    spec define spec_reconfigure_omitted(): bool {
       LibraTimestamp::is_genesis() || LibraTimestamp::spec_now_microseconds() == 0
    }
    spec fun reconfigure_ {
        pragma opaque;
        modifies global<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());

        let config = global<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        let now = LibraTimestamp::spec_now_microseconds();
        let epoch = config.epoch;

        include !spec_reconfigure_omitted() ==> InternalReconfigureAbortsIf && ReconfigureAbortsIf;

        ensures spec_reconfigure_omitted() ==> config == old(config);
        ensures !spec_reconfigure_omitted() ==> config ==
            update_field(
            update_field(old(config),
                epoch, old(config.epoch) + 1),
                last_reconfiguration_time, now);
    }
    /// The following schema describes aborts conditions which we do not want to be propagated to the verification
    /// of callers, and which are therefore marked as `concrete` to be only verified against the implementation.
    /// These conditions are unlikely to happen in reality, and excluding them avoids formal noise.
    spec schema InternalReconfigureAbortsIf {
        let config = global<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        let current_time = LibraTimestamp::spec_now_microseconds();
        aborts_if [concrete] current_time <= config.last_reconfiguration_time with Errors::INVALID_STATE;
        aborts_if [concrete] config.epoch == MAX_U64 with EXECUTION_FAILURE;
    }
    /// This schema is to be used by callers of `reconfigure`
    spec schema ReconfigureAbortsIf {
        let config = global<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        let current_time = LibraTimestamp::spec_now_microseconds();
        aborts_if LibraTimestamp::is_operating()
            && LibraTimestamp::spec_now_microseconds() > 0
            && config.epoch < MAX_U64
            && current_time == config.last_reconfiguration_time
                with Errors::INVALID_STATE;
    }

    /// Emit a `NewEpochEvent` event. This function will be invoked by genesis directly to generate the very first
    /// reconfiguration event.
    fun emit_genesis_reconfiguration_event() acquires Configuration {
        assert(exists<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::not_published(ECONFIGURATION));
        let config_ref = borrow_global_mut<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        assert(config_ref.epoch == 0 && config_ref.last_reconfiguration_time == 0, Errors::invalid_state(ECONFIGURATION));
        config_ref.epoch = 1;

        Event::emit_event<NewEpochEvent>(
            &mut config_ref.events,
            NewEpochEvent {
                epoch: config_ref.epoch,
            },
        );
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Initialization
    spec module {
        /// After genesis, the `Configuration` is published.
        invariant [global] LibraTimestamp::is_operating() ==> spec_has_config();
    }

    /// # Invariants
    spec module {
        /// Configurations are only stored at the libra root address.
        invariant [global]
            forall config_address: address, config_type: type where exists<LibraConfig<config_type>>(config_address):
                config_address == CoreAddresses::LIBRA_ROOT_ADDRESS();

        /// After genesis, no new configurations are added.
        invariant update [global]
            LibraTimestamp::is_operating() ==>
                (forall config_type: type where spec_is_published<config_type>(): old(spec_is_published<config_type>()));

        /// Published configurations are persistent.
        invariant update [global]
            (forall config_type: type where old(spec_is_published<config_type>()): spec_is_published<config_type>());
    }

    /// # Helper Functions
    spec module {
        define spec_has_config(): bool {
            exists<Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS())
        }

        define spec_is_published<Config>(): bool {
            exists<LibraConfig<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS())
        }

        define spec_get_config<Config>(): Config {
            global<LibraConfig<Config>>(CoreAddresses::LIBRA_ROOT_ADDRESS()).payload
        }
    }

}
}
