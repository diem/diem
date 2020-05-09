address 0x0 {

module LibraSystem {
    use 0x0::LibraConfig;
    use 0x0::Transaction;
    use 0x0::Signer;
    use 0x0::ValidatorConfig;
    use 0x0::Vector;

    struct ValidatorInfo {
        addr: address,
        consensus_voting_power: u64,
        config: ValidatorConfig::Config,
    }

    resource struct CapabilityHolder {
        cap: LibraConfig::ModifyConfigCapability<Self::T>,
    }

    struct T {
        // The current consensus crypto scheme.
        scheme: u8,
        // The current validator set. Updated only at epoch boundaries via reconfiguration.
        validators: vector<ValidatorInfo>,
    }

    // ValidatorInfo public accessors

    public fun get_validator_config(addr: address): ValidatorConfig::Config {
        let validator_set = get_validator_set();
        let validator_index_vec = get_validator_index_(&validator_set.validators, addr);
        *&(*Vector::borrow(&validator_set.validators, *Vector::borrow(&validator_index_vec, 0))).config
    }

    // This can only be invoked by the ValidatorSet address to instantiate
    // the resource under that address.
    // It can only be called a single time. Currently, it is invoked in the genesis transaction.
    public fun initialize_validator_set(config_account: &signer) {
        Transaction::assert(Signer::address_of(config_account) == LibraConfig::default_config_address(), 1);

        let cap = LibraConfig::publish_new_config_with_capability<T>(
            T {
                scheme: 0,
                validators: Vector::empty(),
            },
            config_account
        );
        move_to(config_account, CapabilityHolder { cap })
    }

    // This returns a copy of the current validator set.
    public fun get_validator_set(): T {
        LibraConfig::get<T>()
    }

    // This copies the vector of validators into the LibraConfig's resource
    // under ValidatorSet address
    fun set_validator_set(value: T) acquires CapabilityHolder {
        LibraConfig::set_with_capability<T>(&borrow_global<CapabilityHolder>(LibraConfig::default_config_address()).cap, value)
    }

    public fun add_validator(account_address: address) acquires CapabilityHolder {
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        // A prospective validator must have a validator config resource
        Transaction::assert(ValidatorConfig::has(account_address), 17);

        let validator_set = get_validator_set();
        // Ensure that this address is not already a validator
        Transaction::assert(!is_validator_(account_address, &validator_set.validators), 18);

        let config = ValidatorConfig::get_config(account_address);
        Vector::push_back(&mut validator_set.validators, ValidatorInfo {
            addr: account_address,
            config, // copy the config over to ValidatorSet
            consensus_voting_power: 1,
        });

        set_validator_set(validator_set);
    }

    // Removes a validator, only callable by the LibraAssociation address
    public fun remove_validator(account_address: address) acquires CapabilityHolder {
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        let validator_set = get_validator_set();

        // Ensure that this address is an active validator
        let to_remove_index_vec = get_validator_index_(&validator_set.validators, account_address);
        Transaction::assert(!Vector::is_empty(&to_remove_index_vec), 21);
        let to_remove_index = *Vector::borrow(&to_remove_index_vec, 0);
        // Remove corresponding ValidatorInfo from the validator set
        _  = Vector::swap_remove(&mut validator_set.validators, to_remove_index);

        set_validator_set(validator_set);
    }

    // This function can be invoked by the LibraAssociation or by the 0x0 LibraVM address in
    // block_prologue to facilitate reconfigurations at regular intervals.
    // Here for all of the validators the information from ValidatorConfig will
    // get copied into the ValidatorSet.
    // Here validators' consensus_pubkey, validator's networking information and/or
    // validator's full-node information may change.
    // NewEpochEvent event will be fired.
    public fun update_and_reconfigure() acquires CapabilityHolder {
        Transaction::assert(is_sender_authorized_(), 22);

        let validator_set = get_validator_set();
        let validators = &mut validator_set.validators;

        let size = Vector::length(validators);
        if (size == 0) {
            return
        };

        let i = 0;
        let configs_changed = false;
        while (i < size) {
            let validator_info_update = update_ith_validator_info_(validators, i);

            configs_changed = configs_changed || validator_info_update;
            i = i + 1;
        };
        if (configs_changed) {
            set_validator_set(validator_set);
        };
    }

    // Public getters

    // Return true if addr is a current validator
    public fun is_validator(addr: address): bool {
        is_validator_(addr, &get_validator_set().validators)
    }

    // Returns validator info
    // If the address is not a validator, abort
    public fun get_validator_info(addr: address): ValidatorInfo {
        let validator_set = get_validator_set();
        let validator_index_vec = get_validator_index_(&validator_set.validators, addr);
        Transaction::assert(!Vector::is_empty(&validator_index_vec), 19);

        *Vector::borrow(&validator_set.validators, *Vector::borrow(&validator_index_vec, 0))
    }

    // Return the size of the current validator set
    public fun validator_set_size(): u64 {
        Vector::length(&get_validator_set().validators)
    }

    // This function is used in transaction_fee.move to distribute transaction fees among validators
    public fun get_ith_validator_address(i: u64): address {
        Vector::borrow(&get_validator_set().validators, i).addr
    }

    // Private helper functions

    // The Association, the VM and the validator operator from the current validator set
    // are authorized to update the set of validator infos
    fun is_sender_authorized_(): bool {
        // succeed fast
        if (Transaction::sender() == 0xA550C18 || Transaction::sender() == 0x0) {
            return true
        };
        let validators = &get_validator_set().validators;
        // scan the validators to find a match
        let size = Vector::length(validators);
        // always true: size > 3 (see remove_validator code)

        let i = 0;
        while (i < size) {
            if (ValidatorConfig::get_validator_operator_account(Vector::borrow(validators, i).addr) ==
                Transaction::sender()) {
                return true
            };
            i = i + 1;
        };
        return false
    }

    // Get the index of the validator by address in the `validators` vector
    fun get_validator_index_(validators: &vector<ValidatorInfo>, addr: address): vector<u64> {
        let size = Vector::length(validators);
        let result: vector<u64> = Vector::empty();
        if (size == 0) {
            return result
        };

        let i = 0;
        while (i < size) {
            let validator_info_ref = Vector::borrow(validators, i);
            if (validator_info_ref.addr == addr) {
                Vector::push_back(&mut result, i);
                return result
            };
            i = i + 1;
        };

        result
    }

    // Updates ith validator info, if nothing changed, return false
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
        *config_ref = *&new_validator_config;

        true
    }

    fun is_validator_(addr: address, validators_vec_ref: &vector<ValidatorInfo>): bool {
        !Vector::is_empty(&get_validator_index_(validators_vec_ref, addr))
    }

}
}
