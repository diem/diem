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
    use 0x1::Roles::{has_libra_root_role};

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
    // If successful, a NewEpochEvent is fired    
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
    // If successful, a NewEpochEvent is fired    
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

    // For all of the validators the information from ValidatorConfig gets copied into the ValidatorSet.
    // This function makes no changes to the size of the set or the members of the set.
    // NewEpochEvent event will be fired if at least one of the configs was changed.
    public fun update_and_reconfigure(
        lr_account: &signer
        ) acquires CapabilityHolder {
        // TODO: abort code
        assert(has_libra_root_role(lr_account), 919421);
        let validator_set = get_validator_set();
        let validators = &mut validator_set.validators;
        let configs_changed = false;

        let size = Vector::length(validators);
        let i = 0;
        while (i < size) {
            let validator_info_update = update_ith_validator_info_(validators, i);
            configs_changed = configs_changed || validator_info_update;
            i = i + 1;
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
        let i = 0;
        while (i < size) {
            let validator_info_ref = Vector::borrow(validators, i);
            if (validator_info_ref.addr == addr) {
                return Option::some(i)
            };
            i = i + 1;
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

}
}
