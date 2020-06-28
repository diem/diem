// Error codes:
// 1100 -> OPERATOR_ACCOUNT_DOES_NOT_EXIST
// 1101 -> INVALID_TRANSACTION_SENDER
address 0x1 {

module LibraSystem {
    use 0x1::CoreAddresses;
    use 0x1::LibraConfig::{Self, ModifyConfigCapability};
    use 0x1::Option::{Self, Option};
    use 0x1::Signer;
    use 0x1::ValidatorConfig;
    use 0x1::Vector;
    use 0x1::Roles::{Self, has_libra_root_role};

    struct ValidatorInfo {
        addr: address,
        consensus_voting_power: u64,
        config: ValidatorConfig::Config,
    }

    resource struct CapabilityHolder {
        cap: ModifyConfigCapability<LibraSystem>,
    }

    struct LibraSystem {
        // The current consensus crypto scheme.
        scheme: u8,
        // The current validator set. Updated only at epoch boundaries via reconfiguration.
        validators: vector<ValidatorInfo>,
    }

    ///////////////////////////////////////////////////////////////////////////
    // Setup methods
    ///////////////////////////////////////////////////////////////////////////

    // This can only be invoked by the ValidatorSet address to instantiate
    // the resource under that address.
    // It can only be called a single time. Currently, it is invoked in the genesis transaction.
    public fun initialize_validator_set(
        config_account: &signer,
    ) {
        assert(
            Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(),
            1
        );

        let cap = LibraConfig::publish_new_config_with_capability<LibraSystem>(
            config_account,
            LibraSystem {
                scheme: 0,
                validators: Vector::empty(),
            },
        );
        move_to(config_account, CapabilityHolder { cap })
    }

    // This copies the vector of validators into the LibraConfig's resource
    // under ValidatorSet address
    fun set_validator_set(value: LibraSystem) acquires CapabilityHolder {
        LibraConfig::set_with_capability<LibraSystem>(&borrow_global<CapabilityHolder>(CoreAddresses::LIBRA_ROOT_ADDRESS()).cap, value)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Methods operating the Validator Set config callable by the Association only
    ///////////////////////////////////////////////////////////////////////////

    // Adds a new validator, this validator should met the validity conditions
    public fun add_validator(
        lr_account: &signer,
        account_address: address
    ) acquires CapabilityHolder {
        // TODO: abort code
        assert(has_libra_root_role(lr_account), 919419);
        // A prospective validator must have a validator config resource
        assert(ValidatorConfig::is_valid(account_address), 33);

        let validator_set = get_validator_set();
        // Ensure that this address is not already a validator
        assert(!is_validator_(account_address, &validator_set.validators), 18);
        // it is guaranteed that the config is non-empty
        let config = ValidatorConfig::get_config(account_address);
        Vector::push_back(&mut validator_set.validators, ValidatorInfo {
            addr: account_address,
            config, // copy the config over to ValidatorSet
            consensus_voting_power: 1,
        });

        set_validator_set(validator_set);
    }

    // Removes a validator, only callable by the LibraAssociation address
    public fun remove_validator(
        lr_account: &signer,
        account_address: address
    ) acquires CapabilityHolder {
        // TODO: abort code
        assert(has_libra_root_role(lr_account), 919420);
        let validator_set = get_validator_set();
        // Ensure that this address is an active validator
        let to_remove_index_vec = get_validator_index_(&validator_set.validators, account_address);
        assert(Option::is_some(&to_remove_index_vec), 21);
        let to_remove_index = *Option::borrow(&to_remove_index_vec);
        // Remove corresponding ValidatorInfo from the validator set
        _  = Vector::swap_remove(&mut validator_set.validators, to_remove_index);

        set_validator_set(validator_set);
    }

    // For all of the validators the information from ValidatorConfig will
    // get copied into the ValidatorSet.
    // Invalid validators will get removed from the Validator Set.
    // NewEpochEvent event will be fired.
    public fun update_and_reconfigure(
        lr_account: &signer
        ) acquires CapabilityHolder {
        // TODO: abort code
        assert(has_libra_root_role(lr_account), 919421);
        let validator_set = get_validator_set();
        let validators = &mut validator_set.validators;

        let size = Vector::length(validators);
        if (size == 0) {
            return
        };

        let i = size;
        let configs_changed = false;
        while (i > 0) {
            i = i - 1;
            // if the validator is invalid, remove it from the set
            let validator_address = Vector::borrow(validators, i).addr;
            if (ValidatorConfig::is_valid(validator_address)) {
                let validator_info_update = update_ith_validator_info_(validators, i);
                configs_changed = configs_changed || validator_info_update;
            } else {
                _  = Vector::swap_remove(validators, i);
                configs_changed = true;
            }
        };
        if (configs_changed) {
            set_validator_set(validator_set);
        };
    }

    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs: getters
    ///////////////////////////////////////////////////////////////////////////

    // This returns a copy of the current validator set.
    public fun get_validator_set(): LibraSystem {
        LibraConfig::get<LibraSystem>()
    }

    // Return true if addr is a current validator
    public fun is_validator(addr: address): bool {
        is_validator_(addr, &get_validator_set().validators)
    }

    // Returns validator config
    // If the address is not a validator, abort
    public fun get_validator_config(addr: address): ValidatorConfig::Config {
        let validator_set = get_validator_set();
        let validator_index_vec = get_validator_index_(&validator_set.validators, addr);
        assert(Option::is_some(&validator_index_vec), 33);
        *&(Vector::borrow(&validator_set.validators, *Option::borrow(&validator_index_vec))).config
    }

    // Return the size of the current validator set
    public fun validator_set_size(): u64 {
        Vector::length(&get_validator_set().validators)
    }

    // This function is used in transaction_fee.move to distribute transaction fees among validators
    public fun get_ith_validator_address(i: u64): address {
        Vector::borrow(&get_validator_set().validators, i).addr
    }

    ///////////////////////////////////////////////////////////////////////////
    // Private functions
    ///////////////////////////////////////////////////////////////////////////

    // Get the index of the validator by address in the `validators` vector
    fun get_validator_index_(validators: &vector<ValidatorInfo>, addr: address): Option<u64> {
        let size = Vector::length(validators);
        if (size == 0) {
            return Option::none()
        };

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

    // Updates ith validator info, if nothing changed, return false.
    // This function should never throw an assertion.
    fun update_ith_validator_info_(validators: &mut vector<ValidatorInfo>, i: u64): bool {
        let size = Vector::length(validators);
        if (i >= size) {
            return false
        };
        let validator_info = Vector::borrow_mut(validators, i);
        let new_validator_config = ValidatorConfig::get_config(validator_info.addr);
        // check if information is the same
        let config_ref = &mut validator_info.config;

        if (config_ref == &new_validator_config) {
            return false
        };
        *config_ref = new_validator_config;

        true
    }

    fun is_validator_(addr: address, validators_vec_ref: &vector<ValidatorInfo>): bool {
        Option::is_some(&get_validator_index_(validators_vec_ref, addr))
    }

    // **************** Specifications ****************

    /// # Module specifications

    spec module {

        pragma verify = true, aborts_if_is_strict = true;

        /// Returns the validator set stored under association address.
        define spec_get_validator_set(): vector<ValidatorInfo> {
            LibraConfig::spec_get<LibraSystem>().validators
        }

        /// Returns true if there is a validator with address `addr`
        /// in the validator set.
        define spec_is_validator(addr: address): bool {
            exists v in spec_get_validator_set(): v.addr == addr
        }

        /// Returns true if the given validator vector is a set,
        /// meaning that all the validators have unique addresses.
        define spec_validators_is_set(v: vector<ValidatorInfo>): bool {
            forall ii: u64, jj: u64 where
                0 <= ii && ii < len(v) && 0 <= jj && jj < len(v) && ii != jj:
                    v[ii].addr != v[jj].addr
        }
    }

    /// ## Validator set is indeed a set
    spec schema ValidatorsHaveUniqueAddresses {
        invariant module spec_validators_is_set(spec_get_validator_set());
    }

    spec module {
        apply ValidatorsHaveUniqueAddresses to public *;
    }

    /// # Specifications for individual functions

    spec fun initialize_validator_set {
        aborts_if !Roles::spec_has_on_chain_config_privilege(config_account);
        aborts_if Signer::spec_address_of(config_account)
            != CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS();
        aborts_if LibraConfig::spec_is_published<LibraSystem>();
        aborts_if exists<CapabilityHolder>(Signer::spec_address_of(config_account));
        ensures exists<CapabilityHolder>(Signer::spec_address_of(config_account));
        ensures LibraConfig::spec_is_published<LibraSystem>();
        ensures len(spec_get_validator_set()) == 0;
    }

    spec fun set_validator_set {
        pragma assume_no_abort_from_here = true;
        aborts_if !LibraConfig::spec_is_published<LibraSystem>();
        aborts_if !exists<CapabilityHolder>(
            CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS()
        );
        ensures LibraConfig::spec_get<LibraSystem>() == value;
    }

    spec fun add_validator {
        aborts_if !Roles::spec_has_libra_root_role(lr_account);
        aborts_if !LibraConfig::spec_is_published<LibraSystem>();
        aborts_if spec_is_validator(account_address);
        aborts_if !ValidatorConfig::spec_is_valid(account_address);
        aborts_if !exists<CapabilityHolder>(
            CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS()
        );
        ensures spec_is_validator(account_address);
    }

    spec fun remove_validator {
        aborts_if !Roles::spec_has_libra_root_role(lr_account);
        aborts_if !LibraConfig::spec_is_published<LibraSystem>();
        aborts_if !spec_is_validator(account_address);
        aborts_if !exists<CapabilityHolder>(
            CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS()
        );
        ensures !spec_is_validator(account_address);
    }

    spec fun update_and_reconfigure {
        /// > TODO(emmazzz): turn verify on when we are able to
        /// > verify loop invariants.
        pragma verify = false;
        requires spec_validators_is_set(spec_get_validator_set());
        aborts_if !Roles::spec_has_libra_root_role(lr_account);
        aborts_if !LibraConfig::spec_is_published<LibraSystem>();
    }

    spec fun get_validator_set {
        aborts_if !LibraConfig::spec_is_published<LibraSystem>();
        ensures result == LibraConfig::spec_get<LibraSystem>();
    }

    spec fun is_validator {
        aborts_if !LibraConfig::spec_is_published<LibraSystem>();
        ensures result == spec_is_validator(addr);
    }

    spec fun get_validator_config {
        aborts_if !LibraConfig::spec_is_published<LibraSystem>();
        aborts_if !spec_is_validator(addr);
    }

    spec fun validator_set_size {
        aborts_if !LibraConfig::spec_is_published<LibraSystem>();
        ensures result == len(spec_get_validator_set());
    }

    spec fun get_ith_validator_address {
        aborts_if i >= len(spec_get_validator_set());
        aborts_if !LibraConfig::spec_is_published<LibraSystem>();
        ensures result == spec_get_validator_set()[i].addr;
    }

    spec fun get_validator_index_ {
        pragma opaque = true;
        requires module spec_validators_is_set(validators);
        aborts_if false;
        ensures Option::spec_is_none(result) ==
            (forall v in validators: v.addr != addr);
        ensures Option::spec_is_some(result) ==>
            validators[Option::spec_value_inside(result)].addr == addr;
        ensures Option::spec_is_some(result) ==>
            (forall j in 0..len(validators) where j != Option::spec_value_inside(result):
                validators[j].addr != addr);
    }

    spec fun update_ith_validator_info_ {
        aborts_if i < len(validators) &&
            !ValidatorConfig::spec_is_valid(validators[i].addr);
        ensures i < len(validators) ==> validators[i].config ==
             ValidatorConfig::spec_get_config(validators[i].addr);
        ensures i < len(validators) ==>
            result == (old(validators[i].config) !=
                ValidatorConfig::spec_get_config(validators[i].addr));
    }

    spec fun is_validator_ {
        pragma opaque = true;
        aborts_if false;
        requires module spec_validators_is_set(validators_vec_ref);
        ensures result == (exists v in validators_vec_ref: v.addr == addr);
        ensures spec_validators_is_set(validators_vec_ref);
    }
}
}
