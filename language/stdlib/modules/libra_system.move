// Error codes:
// 1100 -> OPERATOR_ACCOUNT_DOES_NOT_EXIST
// 1101 -> INVALID_TRANSACTION_SENDER
address 0x0 {

module LibraSystem {
    use 0x0::LibraAccount;
    use 0x0::LibraConfig;
    use 0x0::Option;
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

    ///////////////////////////////////////////////////////////////////////////
    // Setup methods
    ///////////////////////////////////////////////////////////////////////////

    // This can only be invoked by the ValidatorSet address to instantiate
    // the resource under that address.
    // It can only be called a single time. Currently, it is invoked in the genesis transaction.
    public fun initialize_validator_set(config_account: &signer) {
        Transaction::assert(Signer::address_of(config_account) == LibraConfig::default_config_address(), 1);

        let cap = LibraConfig::publish_new_config_with_capability<T>(
            config_account,
            T {
                scheme: 0,
                validators: Vector::empty(),
            },
        );
        move_to(config_account, CapabilityHolder { cap })
    }

    // This copies the vector of validators into the LibraConfig's resource
    // under ValidatorSet address
    fun set_validator_set(value: T) acquires CapabilityHolder {
        LibraConfig::set_with_capability<T>(&borrow_global<CapabilityHolder>(LibraConfig::default_config_address()).cap, value)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Methods operating the Validator Set config
    // callable by Validator's operators
    ///////////////////////////////////////////////////////////////////////////

    // Adds a new validator, this validator should met the validity conditions
    public fun add_validator(account_address: address) acquires CapabilityHolder {
        // Validator's operator can add its certified validator to the validator set
        Transaction::assert(Transaction::sender() ==
                            ValidatorConfig::get_operator(account_address), 22);

        // A prospective validator must have a validator config resource
        Transaction::assert(is_valid_and_certified(account_address), 33);

        let validator_set = get_validator_set();
        // Ensure that this address is not already a validator
        Transaction::assert(!is_validator_(account_address, &validator_set.validators), 18);
        // Since ValidatorConfig::is_valid(account_address) == true,
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
    public fun remove_validator(account_address: address) acquires CapabilityHolder {
        // Validator's operator can remove its certified validator from the validator set
        Transaction::assert(Transaction::sender() ==
                            ValidatorConfig::get_operator(account_address), 22);

        let validator_set = get_validator_set();
        // Ensure that this address is an active validator
        let to_remove_index_vec = get_validator_index_(&validator_set.validators, account_address);
        Transaction::assert(Option::is_some(&to_remove_index_vec), 21);
        let to_remove_index = *Option::borrow(&to_remove_index_vec);
        // Remove corresponding ValidatorInfo from the validator set
        _  = Vector::swap_remove(&mut validator_set.validators, to_remove_index);

        set_validator_set(validator_set);
    }

    ///////////////////////////////////////////////////////////////////////////
    // Methods operating the Validator Set config callable
    // by the Validator's operators and the Association
    ///////////////////////////////////////////////////////////////////////////

    // This function can be invoked by the LibraAssociation or by the 0x0 LibraVM address in
    // block_prologue to facilitate reconfigurations at regular intervals.
    // Here for all of the validators the information from ValidatorConfig will
    // get copied into the ValidatorSet.
    // Invalid or decertified validators will get removed from the Validator Set.
    // NewEpochEvent event will be fired.
    public fun update_and_reconfigure() acquires CapabilityHolder {
        Transaction::assert(is_sender_authorized_to_reconfigure_(), 22);

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
            if (is_valid_and_certified(validator_address)) {
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
    public fun get_validator_set(): T {
        LibraConfig::get<T>()
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
        Transaction::assert(Option::is_some(&validator_index_vec), 33);
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

    fun is_valid_and_certified(addr: address): bool {
        ValidatorConfig::is_valid(addr) &&
            LibraAccount::is_certified<LibraAccount::ValidatorRole>(addr)
            // TODO(valerini): only allow certified operators, i.e. uncomment the line
            // && LibraAccount::is_certified<LibraAccount::ValidatorOperatorRole>(ValidatorConfig::get_operator(addr))
    }

    // The Association, the VM, the validator operator or the validator from the current validator set
    // are authorized to update the set of validator infos and add/remove validators
    fun is_sender_authorized_to_reconfigure_(): bool {
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
            if (Vector::borrow(validators, i).addr == Transaction::sender()) {
                return true
            };
            if (ValidatorConfig::get_operator(Vector::borrow(validators, i).addr) ==
                Transaction::sender()) {
                return true
            };
            i = i + 1;
        };
        return false
    }

    // Get the index of the validator by address in the `validators` vector
    fun get_validator_index_(validators: &vector<ValidatorInfo>, addr: address): Option::T<u64> {
        let size = Vector::length(validators);
        if (size == 0) {
            return Option::none()
        };

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
