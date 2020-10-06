address 0x1 {

/// Maintains information about the set of validators used during consensus.
/// Provides functions to add, remove, and update validators in the
/// validator set.
///
/// > Note: When trying to understand this code, it's important to know that "config"
/// and "configuration" are used for several distinct concepts.
module LibraSystem {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::LibraConfig::{Self, ModifyConfigCapability};
    use 0x1::Option::{Self, Option};
    use 0x1::Signer;
    use 0x1::ValidatorConfig;
    use 0x1::Vector;
    use 0x1::Roles;
    use 0x1::LibraTimestamp;

    /// Information about a Validator Owner.
    struct ValidatorInfo {
        /// The address (account) of the Validator Owner
        addr: address,
        /// The voting power of the Validator Owner (currently always 1).
        consensus_voting_power: u64,
        /// Configuration information about the Validator, such as the
        /// Validator Operator, human name, and info such as consensus key
        /// and network addresses.
        config: ValidatorConfig::Config,
        /// The time of last reconfiguration invoked by this validator
        /// in microseconds
        last_config_update_time: u64,
    }

    /// Enables a scheme that restricts the LibraSystem config
    /// in LibraConfig from being modified by any other module.  Only
    /// code in this module can get a reference to the ModifyConfigCapability<LibraSystem>,
    /// which is required by `LibraConfig::set_with_capability_and_reconfigure` to
    /// modify the LibraSystem config. This is only needed by `update_config_and_reconfigure`.
    /// Only Libra root can add or remove a validator from the validator set, so the
    /// capability is not needed for access control in those functions.
    resource struct CapabilityHolder {
        /// Holds a capability returned by `LibraConfig::publish_new_config_and_get_capability`
        /// which is called in `initialize_validator_set`.
        cap: ModifyConfigCapability<LibraSystem>,
    }

    /// The LibraSystem struct stores the validator set and crypto scheme in
    /// LibraConfig. The LibraSystem struct is stored by LibraConfig, which publishes a
    /// LibraConfig<LibraSystem> resource.
    struct LibraSystem {
        /// The current consensus crypto scheme.
        scheme: u8,
        /// The current validator set.
        validators: vector<ValidatorInfo>,
    }
    spec struct LibraSystem {
        /// Members of `validators` vector (the validator set) have unique addresses.
        invariant
            forall i in 0..len(validators), j in 0..len(validators):
                validators[i].addr == validators[j].addr ==> i == j;
    }

    /// The `CapabilityHolder` resource was not in the required state
    const ECAPABILITY_HOLDER: u64 = 0;
    /// Tried to add a validator with an invalid state to the validator set
    const EINVALID_PROSPECTIVE_VALIDATOR: u64 = 1;
    /// Tried to add a validator to the validator set that was already in it
    const EALREADY_A_VALIDATOR: u64 = 2;
    /// An operation was attempted on an address not in the vaidator set
    const ENOT_AN_ACTIVE_VALIDATOR: u64 = 3;
    /// The validator operator is not the operator for the specified validator
    const EINVALID_TRANSACTION_SENDER: u64 = 4;
    /// An out of bounds index for the validator set was encountered
    const EVALIDATOR_INDEX: u64 = 5;
    /// Rate limited when trying to update config
    const ECONFIG_UPDATE_RATE_LIMITED: u64 = 6;

    /// Number of microseconds in 5 minutes
    const FIVE_MINUTES: u64 = 300000000;

    ///////////////////////////////////////////////////////////////////////////
    // Setup methods
    ///////////////////////////////////////////////////////////////////////////


    /// Publishes the LibraConfig for the LibraSystem struct, which contains the current
    /// validator set. Also publishes the `CapabilityHolder` with the
    /// ModifyConfigCapability<LibraSystem> returned by the publish function, which allows
    /// code in this module to change LibraSystem config (including the validator set).
    /// Must be invoked by the Libra root a single time in Genesis.
    public fun initialize_validator_set(
        lr_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        Roles::assert_libra_root(lr_account);

        let cap = LibraConfig::publish_new_config_and_get_capability<LibraSystem>(
            lr_account,
            LibraSystem {
                scheme: 0,
                validators: Vector::empty(),
            },
        );
        assert(
            !exists<CapabilityHolder>(CoreAddresses::LIBRA_ROOT_ADDRESS()),
            Errors::already_published(ECAPABILITY_HOLDER)
        );
        move_to(lr_account, CapabilityHolder { cap })
    }
    spec fun initialize_validator_set {
        modifies global<LibraConfig::LibraConfig<LibraSystem>>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include LibraTimestamp::AbortsIfNotGenesis;
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        let lr_addr = Signer::spec_address_of(lr_account);
        // TODO: The next two aborts_if's are not independent. Perhaps they can be
        // simplified.
        aborts_if LibraConfig::spec_is_published<LibraSystem>() with Errors::ALREADY_PUBLISHED;
        aborts_if exists<CapabilityHolder>(lr_addr) with Errors::ALREADY_PUBLISHED;
        ensures exists<CapabilityHolder>(lr_addr);
        ensures LibraConfig::spec_is_published<LibraSystem>();
        ensures len(spec_get_validators()) == 0;
    }

    /// Copies a LibraSystem struct into the LibraConfig<LibraSystem> resource
    /// Called by the add, remove, and update functions.
    fun set_libra_system_config(value: LibraSystem) acquires CapabilityHolder {
        LibraTimestamp::assert_operating();
        assert(
            exists<CapabilityHolder>(CoreAddresses::LIBRA_ROOT_ADDRESS()),
            Errors::not_published(ECAPABILITY_HOLDER)
        );
        // Updates the LibraConfig<LibraSystem> and emits a reconfigure event.
        LibraConfig::set_with_capability_and_reconfigure<LibraSystem>(
            &borrow_global<CapabilityHolder>(CoreAddresses::LIBRA_ROOT_ADDRESS()).cap,
            value
        )
    }
    spec fun set_libra_system_config {
        pragma opaque;
        modifies global<LibraConfig::LibraConfig<LibraSystem>>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        modifies global<LibraConfig::Configuration>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include LibraTimestamp::AbortsIfNotOperating;
        include LibraConfig::ReconfigureAbortsIf;
        /// `payload` is the only field of LibraConfig, so next completely specifies it.
        ensures global<LibraConfig::LibraConfig<LibraSystem>>(CoreAddresses::LIBRA_ROOT_ADDRESS()).payload == value;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Methods operating the Validator Set config callable by the libra root account
    ///////////////////////////////////////////////////////////////////////////

    /// Adds a new validator to the validator set.
    public fun add_validator(
        lr_account: &signer,
        validator_addr: address
    ) acquires CapabilityHolder {

        LibraTimestamp::assert_operating();
        Roles::assert_libra_root(lr_account);
        // A prospective validator must have a validator config resource
        assert(ValidatorConfig::is_valid(validator_addr), Errors::invalid_argument(EINVALID_PROSPECTIVE_VALIDATOR));

        let libra_system_config = get_libra_system_config();

        // Ensure that this address is not already a validator
        assert(
            !is_validator_(validator_addr, &libra_system_config.validators),
            Errors::invalid_argument(EALREADY_A_VALIDATOR)
        );
        // it is guaranteed that the config is non-empty
        let config = ValidatorConfig::get_config(validator_addr);
        Vector::push_back(&mut libra_system_config.validators, ValidatorInfo {
            addr: validator_addr,
            config, // copy the config over to ValidatorSet
            consensus_voting_power: 1,
            last_config_update_time: LibraTimestamp::now_microseconds(),
        });

        set_libra_system_config(libra_system_config);
    }
    spec fun add_validator {
        modifies global<LibraConfig::LibraConfig<LibraSystem>>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include AddValidatorAbortsIf;
        include AddValidatorEnsures;
    }
    spec schema AddValidatorAbortsIf {
        lr_account: signer;
        validator_addr: address;
        include LibraTimestamp::AbortsIfNotOperating;
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        include LibraConfig::ReconfigureAbortsIf;
        aborts_if !ValidatorConfig::is_valid(validator_addr) with Errors::INVALID_ARGUMENT;
        aborts_if spec_is_validator(validator_addr) with Errors::INVALID_ARGUMENT;
    }
    spec schema AddValidatorEnsures {
        validator_addr: address;
        /// LIP-6 property: validator has validator role. The code does not check this explicitly,
        /// but it is implied by the `assert ValidatorConfig::is_valid`, since there
        /// is an invariant (in ValidatorConfig) that a an address with a published ValidatorConfig has
        /// a ValidatorRole
        ensures Roles::spec_has_validator_role_addr(validator_addr);
        ensures ValidatorConfig::is_valid(validator_addr);
        ensures spec_is_validator(validator_addr);
        let vs = spec_get_validators();
        ensures Vector::eq_push_back(vs,
                                     old(vs),
                                     ValidatorInfo {
                                         addr: validator_addr,
                                         config: ValidatorConfig::spec_get_config(validator_addr),
                                         consensus_voting_power: 1,
                                         last_config_update_time: LibraTimestamp::spec_now_microseconds(),
                                      }
                                   );
    }


    /// Removes a validator, aborts unless called by libra root account
    public fun remove_validator(
        lr_account: &signer,
        validator_addr: address
    ) acquires CapabilityHolder {
        LibraTimestamp::assert_operating();
        Roles::assert_libra_root(lr_account);
        let libra_system_config = get_libra_system_config();
        // Ensure that this address is an active validator
        let to_remove_index_vec = get_validator_index_(&libra_system_config.validators, validator_addr);
        assert(Option::is_some(&to_remove_index_vec), Errors::invalid_argument(ENOT_AN_ACTIVE_VALIDATOR));
        let to_remove_index = *Option::borrow(&to_remove_index_vec);
        // Remove corresponding ValidatorInfo from the validator set
        _  = Vector::swap_remove(&mut libra_system_config.validators, to_remove_index);

        set_libra_system_config(libra_system_config);
    }
    spec fun remove_validator {
        modifies global<LibraConfig::LibraConfig<LibraSystem>>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include RemoveValidatorAbortsIf;
        include RemoveValidatorEnsures;
    }
    spec schema RemoveValidatorAbortsIf {
        lr_account: signer;
        validator_addr: address;
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        include LibraTimestamp::AbortsIfNotOperating;
        include LibraConfig::ReconfigureAbortsIf;
        aborts_if !spec_is_validator(validator_addr) with Errors::INVALID_ARGUMENT;
    }
    spec schema RemoveValidatorEnsures {
        validator_addr: address;
        let vs = spec_get_validators();
        ensures forall vi in vs where vi.addr != validator_addr: exists ovi in old(vs): vi == ovi;
        /// Removed validator is no longer a validator.  Depends on no other entries for same address
        /// in validator_set
        ensures !spec_is_validator(validator_addr);
    }

    /// Copy the information from ValidatorConfig into the validator set.
    /// This function makes no changes to the size or the members of the set.
    /// If the config in the ValidatorSet changes, it stores the new LibraSystem
    /// and emits a reconfigurationevent.
    public fun update_config_and_reconfigure(
        validator_operator_account: &signer,
        validator_addr: address,
    ) acquires CapabilityHolder {
        LibraTimestamp::assert_operating();
        Roles::assert_validator_operator(validator_operator_account);
        assert(
            ValidatorConfig::get_operator(validator_addr) == Signer::address_of(validator_operator_account),
            Errors::invalid_argument(EINVALID_TRANSACTION_SENDER)
        );
        let libra_system_config = get_libra_system_config();
        let to_update_index_vec = get_validator_index_(&libra_system_config.validators, validator_addr);
        assert(Option::is_some(&to_update_index_vec), Errors::invalid_argument(ENOT_AN_ACTIVE_VALIDATOR));
        let to_update_index = *Option::borrow(&to_update_index_vec);
        let is_validator_info_updated = update_ith_validator_info_(&mut libra_system_config.validators, to_update_index);
        if (is_validator_info_updated) {
            let validator_info = Vector::borrow_mut(&mut libra_system_config.validators, to_update_index);
            assert(LibraTimestamp::now_microseconds() >
                   validator_info.last_config_update_time + FIVE_MINUTES,
                   ECONFIG_UPDATE_RATE_LIMITED);
            validator_info.last_config_update_time = LibraTimestamp::now_microseconds();
            set_libra_system_config(libra_system_config);
        }
    }
    spec fun update_config_and_reconfigure {
        pragma opaque;
        pragma verify_duration_estimate = 100;
        modifies global<LibraConfig::LibraConfig<LibraSystem>>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        include UpdateConfigAndReconfigureAbortsIf;
        include UpdateConfigAndReconfigureEnsures;
        // The property below is not in `UpdateConfigAndReconfigureEnsures` because that is reused
        // with a different condition in the transaction script `set_validator_config_and_reconfigure`.
        // The validator set is not updated if: (1) `validator_addr` is not in the validator set, or
        // (2) the validator info would not change. The code only does a reconfiguration if the
        // validator set would change.  `ReconfigureAbortsIf` complains if the block time does not
        // advance, but block time only advances if there is a reconfiguration.
        let is_validator_info_updated =
            ValidatorConfig::is_valid(validator_addr) &&
            (exists v_info in spec_get_validators():
                v_info.addr == validator_addr
                && v_info.config != ValidatorConfig::spec_get_config(validator_addr));
        include is_validator_info_updated ==> LibraConfig::ReconfigureAbortsIf;
    }
    spec schema UpdateConfigAndReconfigureAbortsIf {
        validator_addr: address;
        validator_operator_account: signer;
        let validator_operator_addr = Signer::address_of(validator_operator_account);
        include LibraTimestamp::AbortsIfNotOperating;
        /// Must abort if the signer does not have the ValidatorOperator role [[H14]][PERMISSION].
        include Roles::AbortsIfNotValidatorOperator{validator_operator_addr: validator_operator_addr};
        include ValidatorConfig::AbortsIfNoValidatorConfig{addr: validator_addr};
        aborts_if ValidatorConfig::get_operator(validator_addr) != validator_operator_addr
            with Errors::INVALID_ARGUMENT;
        aborts_if !spec_is_validator(validator_addr) with Errors::INVALID_ARGUMENT;
    }


    /// Does not change the length of the validator set, only changes ValidatorInfo
    /// for validator_addr, and doesn't change any addresses.
    spec schema UpdateConfigAndReconfigureEnsures {
        validator_addr: address;
        let vs = spec_get_validators();
        ensures len(vs) == len(old(vs));
        /// No addresses change in the validator set
        ensures forall i in 0..len(vs): vs[i].addr == old(vs)[i].addr;
        /// If the `ValidatorInfo` address is not the one we're changing, the info does not change.
        ensures forall i in 0..len(vs) where old(vs)[i].addr != validator_addr:
                         vs[i] == old(vs)[i];
        /// It updates the correct entry in the correct way
        ensures forall i in 0..len(vs): vs[i].config == old(vs[i].config) ||
                    (old(vs)[i].addr == validator_addr &&
                    vs[i].config == ValidatorConfig::get_config(validator_addr));
        /// LIP-6 property
        ensures Roles::spec_has_validator_role_addr(validator_addr);
    }

    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs: getters
    ///////////////////////////////////////////////////////////////////////////

    /// Get the LibraSystem configuration from LibraConfig
    public fun get_libra_system_config(): LibraSystem {
        LibraConfig::get<LibraSystem>()
    }
    spec fun get_libra_system_config {
        pragma opaque;
        include LibraConfig::AbortsIfNotPublished<LibraSystem>;
        ensures result == LibraConfig::get<LibraSystem>();
    }

    /// Return true if `addr` is in the current validator set
    public fun is_validator(addr: address): bool {
        is_validator_(addr, &get_libra_system_config().validators)
    }
    spec fun is_validator {
        pragma opaque;
        // TODO: Publication of LibraConfig<LibraSystem> in initialization
        // and persistence of configs implies that the next abort cannot
        // actually happen.
        include LibraConfig::AbortsIfNotPublished<LibraSystem>;
        ensures result == spec_is_validator(addr);
    }
    spec define spec_is_validator(addr: address): bool {
        exists v in spec_get_validators(): v.addr == addr
    }

    /// Returns validator config. Aborts if `addr` is not in the validator set.
    public fun get_validator_config(addr: address): ValidatorConfig::Config {
        let libra_system_config = get_libra_system_config();
        let validator_index_vec = get_validator_index_(&libra_system_config.validators, addr);
        assert(Option::is_some(&validator_index_vec), Errors::invalid_argument(ENOT_AN_ACTIVE_VALIDATOR));
        *&(Vector::borrow(&libra_system_config.validators, *Option::borrow(&validator_index_vec))).config
    }
    spec fun get_validator_config {
        pragma opaque;
        include LibraConfig::AbortsIfNotPublished<LibraSystem>;
        aborts_if !spec_is_validator(addr) with Errors::INVALID_ARGUMENT;
        ensures
            exists info in LibraConfig::get<LibraSystem>().validators where info.addr == addr:
                result == info.config;
    }

    /// Return the size of the current validator set
    public fun validator_set_size(): u64 {
        Vector::length(&get_libra_system_config().validators)
    }
    spec fun validator_set_size {
        pragma opaque;
        include LibraConfig::AbortsIfNotPublished<LibraSystem>;
        ensures result == len(spec_get_validators());
    }

    /// Used in `transaction_fee.move` to distribute transaction fees among validators
    public fun get_ith_validator_address(i: u64): address {
        assert(i < validator_set_size(), Errors::invalid_argument(EVALIDATOR_INDEX));
        Vector::borrow(&get_libra_system_config().validators, i).addr
    }
    spec fun get_ith_validator_address {
        pragma opaque;
        include LibraConfig::AbortsIfNotPublished<LibraSystem>;
        aborts_if i >= len(spec_get_validators()) with Errors::INVALID_ARGUMENT;
        ensures result == spec_get_validators()[i].addr;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Private functions
    ///////////////////////////////////////////////////////////////////////////

    /// Get the index of the validator by address in the `validators` vector
    /// It has a loop, so there are spec blocks in the code to assert loop invariants.
    fun get_validator_index_(validators: &vector<ValidatorInfo>, addr: address): Option<u64> {
        let size = Vector::length(validators);
        let i = 0;
        while ({
            spec {
                assert i <= size;
                assert forall j in 0..i: validators[j].addr != addr;
            };
            (i < size)
        })
        {
            let validator_info_ref = Vector::borrow(validators, i);
            if (validator_info_ref.addr == addr) {
                spec {
                    assert validators[i].addr == addr;
                };
                return Option::some(i)
            };
            i = i + 1;
        };
        spec {
            assert i == size;
            assert forall j in 0..size: validators[j].addr != addr;
        };
        return Option::none()
    }
    spec fun get_validator_index_ {
        pragma opaque;
        aborts_if false;
        let size = len(validators);
        /// If `addr` is not in validator set, returns none.
        ensures (forall i in 0..size: validators[i].addr != addr) ==> Option::is_none(result);
        /// If `addr` is in validator set, return the least index of an entry with that address.
        /// The data invariant associated with the LibraSystem.validators that implies
        /// that there is exactly one such address.
        ensures
            (exists i in 0..size: validators[i].addr == addr) ==>
                Option::is_some(result)
                && {
                        let at = Option::borrow(result);
                        0 <= at && at < size && validators[at].addr == addr
                    };
    }

    /// Updates *i*th validator info, if nothing changed, return false.
    /// This function never aborts.
    fun update_ith_validator_info_(validators: &mut vector<ValidatorInfo>, i: u64): bool {
        let size = Vector::length(validators);
        // This provably cannot happen, but left it here for safety.
        if (i >= size) {
            return false
        };
        let validator_info = Vector::borrow_mut(validators, i);
        // I believe "is_valid" below always holds, based on a global invariant later
        // in the file (which proves if we comment out some other specifications),
        // but left it here for safety.
        if (!ValidatorConfig::is_valid(validator_info.addr)) {
            return false
        };
        let new_validator_config = ValidatorConfig::get_config(validator_info.addr);
        // check if information is the same
        let config_ref = &mut validator_info.config;
        if (config_ref == &new_validator_config) {
            return false
        };
        *config_ref = new_validator_config;
        true
    }
    spec fun update_ith_validator_info_ {
        pragma opaque;
        aborts_if false;
        let new_validator_config = ValidatorConfig::spec_get_config(validators[i].addr);
        /// Prover is able to prove this because get_validator_index_ ensures it
        /// in calling context.
        requires 0 <= i && i < len(validators);
        /// Somewhat simplified from the code because of properties guaranteed
        /// by the calling context.
        ensures
            result ==
                (ValidatorConfig::is_valid(validators[i].addr) &&
                 new_validator_config != old(validators[i].config));
        /// It only updates validators at index `i`, and updates the
        /// `config` field to `new_validator_config`.
        ensures
            result ==>
                validators == update_vector(
                    old(validators),
                    i,
                    update_field(old(validators[i]), config, new_validator_config)
                );
        /// Does not change validators if result is false
        ensures !result ==> validators == old(validators);
        /// Updates the ith validator entry (and nothing else), as appropriate.
        ensures validators == update_vector(old(validators), i, validators[i]);
        /// Needed these assertions to make "consensus voting power is always 1" invariant
        /// prove (not sure why).
        requires forall i1 in 0..len(spec_get_validators()):
           spec_get_validators()[i1].consensus_voting_power == 1;
        ensures forall i1 in 0..len(spec_get_validators()):
           spec_get_validators()[i1].consensus_voting_power == 1;
    }

    /// Private function checks for membership of `addr` in validator set.
    fun is_validator_(addr: address, validators_vec_ref: &vector<ValidatorInfo>): bool {
        Option::is_some(&get_validator_index_(validators_vec_ref, addr))
    }
    spec fun is_validator_ {
        pragma opaque;
        aborts_if false;
        ensures result == (exists v in validators_vec_ref: v.addr == addr);
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Initialization
    spec module {
        /// After genesis, the `LibraSystem` configuration is published, as well as the capability
        /// which grants the right to modify it to certain functions in this module.
        invariant [global] LibraTimestamp::is_operating() ==>
            LibraConfig::spec_is_published<LibraSystem>() &&
            exists<CapabilityHolder>(CoreAddresses::LIBRA_ROOT_ADDRESS());
    }

    /// # Access Control

    /// Access control requirements for validator set are a bit more complicated than
    /// many parts of the framework because of `update_config_and_reconfigure`.
    /// That function updates the validator info (e.g., the network address) for a
    /// particular Validator Owner, but only if the signer is the Operator for that owner.
    /// Therefore, we must ensure that the information for other validators in the
    /// validator set are not changed, which is specified locally for
    /// `update_config_and_reconfigure`.

    spec module {
       /// The permission "{Add, Remove} Validator" is granted to LibraRoot [[H13]][PERMISSION].
       apply Roles::AbortsIfNotLibraRoot{account: lr_account} to add_validator, remove_validator;
    }

    spec schema ValidatorSetConfigRemainsSame {
        ensures spec_get_validators() == old(spec_get_validators());
    }
    spec module {
        /// Only {add, remove} validator [[H13]][PERMISSION] and update_config_and_reconfigure
        /// [[H14]][PERMISSION] may change the set of validators in the configuration.
        /// `set_libra_system_config` is a private function which is only called by other
        /// functions in the "except" list. `initialize_validator_set` is only called in
        /// Genesis.
        apply ValidatorSetConfigRemainsSame to *, *<T>
           except add_validator, remove_validator, update_config_and_reconfigure,
               initialize_validator_set, set_libra_system_config;
    }



    /// # Helper Functions
    spec module {
        /// Fetches the currently published validator set from the published LibraConfig<LibraSystem>
        /// resource.
        define spec_get_validators(): vector<ValidatorInfo> {
            LibraConfig::get<LibraSystem>().validators
        }
    }

    spec module {

       /// Every validator has a published ValidatorConfig whose config option is "some"
       /// (meaning of ValidatorConfig::is_valid).
       /// > Unfortunately, this times out for unknown reasons (it doesn't seem to be hard),
       /// so it is deactivated.
       /// The Prover can prove it if the uniqueness invariant for the LibraSystem resource
       /// is commented out, along with aborts for update_config_and_reconfigure and everything
       /// else that breaks (e.g., there is an ensures in remove_validator that has to be
       /// commented out)
       invariant [deactivated, global] forall i1 in 0..len(spec_get_validators()):
           ValidatorConfig::is_valid(spec_get_validators()[i1].addr);

       /// Every validator in the validator set has a validator role.
       /// > Note: Verification of LibraSystem seems to be very sensitive, and will
       /// often time out after small changes.  Disabling this property
       /// (with [deactivate, global]) is sometimes a quick temporary fix.
       invariant [global] forall i1 in 0..len(spec_get_validators()):
           Roles::spec_has_validator_role_addr(spec_get_validators()[i1].addr);

       /// `Consensus_voting_power` is always 1. In future implementations, this
       /// field may have different values in which case this property will have to
       /// change. It's here currently because and accidental or illicit change
       /// to the voting power of a validator could defeat the Byzantine fault tolerance
       /// of LibraBFT.
       invariant [global] forall i1 in 0..len(spec_get_validators()):
           spec_get_validators()[i1].consensus_voting_power == 1;

    }
}
}
