address 0x0:

// These modules only change resources stored under the three addresses:
// LibraAssociation's account address: 0xA550C18
// ValidatorSet resource address: 0x1D8
// DiscoverySet resource address: 0xD15C0
// ValidatorSet and DiscoverySet are stored under different addresses in order to facilitate
// cheaper reads of these sets from storage.
module LibraSystem2 {
    use 0x0::LibraAccount;
    use 0x0::LibraTimestamp;
    use 0x0::Transaction;
    use 0x0::ValidatorConfig2;
    use 0x0::Vector;

    struct ValidatorInfo {
        addr: address,
        consensus_voting_power: u64,
        config: ValidatorConfig2::Config,
        // To protect against DDoS, the force updates are allowed only if the time passed
        // since the last force update is larger than a threshold (24hr).
        last_force_update_time: u64,
    }

    struct ValidatorSetChangeEvent {
        new_validator_set: vector<ValidatorInfo>,
    }

    resource struct ValidatorSet {
        validators: vector<ValidatorInfo>,
        last_reconfiguration_time: u64,
        change_events: LibraAccount::EventHandle<ValidatorSetChangeEvent>,
    }

    // ValidatorInfo public accessors

    // Get validator's address from ValidatorInfo
    public fun get_validator_address(v: &ValidatorInfo): &address {
        &v.addr
    }

    // Get validator's config from ValidatorInfo
    public fun get_validator_config(v: &ValidatorInfo): &ValidatorConfig2::Config {
        &v.config
    }

    // Get validator's voting power from ValidatorInfo
    public fun get_validator_voting_power(v: &ValidatorInfo): &u64 {
        &v.consensus_voting_power
    }

    // Get last forced update time
    public fun get_last_force_update_time(v: &ValidatorInfo): &u64 {
        &v.last_force_update_time
    }

    // This can only be invoked by the ValidatorSet address to instantiate the resource under that address.
    // It can only be called a single time. Currently, it is invoked in the genesis transaction.
    public fun initialize_validator_set() {
        Transaction::assert(Transaction::sender() == 0x1D8, 1);
        move_to_sender<ValidatorSet>(ValidatorSet {
            validators: Vector::empty(),
            last_reconfiguration_time: 0,
            change_events: LibraAccount::new_event_handle<ValidatorSetChangeEvent>(),
        });
    }

    // Adds a new validator, only callable by the LibraAssociation address
    // TODO(valerini): allow the Association to add multiple validators in a single block
    public fun add_validator(account_address: &address) acquires ValidatorSet {
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        // A prospective validator must have a validator config resource
        Transaction::assert(ValidatorConfig2::has(*account_address), 17);

        let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);
        // Ensure that this address is not already a validator
        Transaction::assert(!is_validator_(account_address, &validator_set_ref.validators), 18);

        let config = ValidatorConfig2::get_config(*account_address);
        let current_block_time = LibraTimestamp::now_microseconds();
        Vector::push_back(&mut validator_set_ref.validators, ValidatorInfo {
            addr: *account_address,
            config: config, // copy the config over to ValidatorSet
            consensus_voting_power: 1,
            last_force_update_time: current_block_time,
        });

        emit_reconfiguration_event();
    }

    // Removes a validator, only callable by the LibraAssociation address
    public fun remove_validator(account_address: &address) acquires ValidatorSet {
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);
        let size = Vector::length(&validator_set_ref.validators);
        // Make sure the size of validator set does not go beyond 4, which is the minimum safe number of validators
        Transaction::assert(size > 4, 22);

        // Ensure that this address is an active validator
        let to_remove_index_vec = get_validator_index_(&validator_set_ref.validators, account_address);
        Transaction::assert(!Vector::is_empty(&to_remove_index_vec), 21);
        // Remove corresponding ValidatorInfo from the validator set
        _  = Vector::swap_remove(&mut validator_set_ref.validators, *Vector::borrow(&to_remove_index_vec, 0));

        emit_reconfiguration_event();
    }

    // Copies validator's config to ValidatorSet
    // Callable by the LibraAssociation address only.
    // Aborts if there are no changes to the config.
    public fun update_validator_info(addr: &address) acquires ValidatorSet {
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);

        let validator_index_vec = get_validator_index_(&validator_set_ref.validators, addr);
        Transaction::assert(!Vector::is_empty(&validator_index_vec), 19);
        Transaction::assert(update_ith_validator_info_(&mut validator_set_ref.validators, *Vector::borrow(&validator_index_vec, 0)), 42);
        emit_reconfiguration_event();
    }

    // This function can be called by the validator's operator,
    // which is either the validator itself or the validator's delegate account.
    // Aborts if there are no changes to the config.
    public fun force_update_validator_info(validator_address: &address) acquires ValidatorSet {
        // Check the sender
        Transaction::assert(ValidatorConfig2::get_validator_operator_account(*validator_address) == Transaction::sender(), 1);

        let validator_vec_ref = borrow_global_mut<ValidatorSet>(0x1D8);
        let validator_index_vec = get_validator_index_(&validator_vec_ref.validators, validator_address);
        Transaction::assert(!Vector::is_empty(&validator_index_vec), 19);
        let validator_index = *Vector::borrow(&validator_index_vec, 0);

        // Update force update timer to enforce rate-limits
        let validator_info_ref = Vector::borrow(&mut validator_vec_ref.validators, validator_index);

        // If less than 24hr has passed since the last forced update, abort.
        // Note that validators can always ask LibraAssocition offchain to invoke update.
        let current_block_time = LibraTimestamp::now_microseconds();
        Transaction::assert(current_block_time - validator_info_ref.last_force_update_time >= 86400000000, 11);

        // Copy validator config from validator's account and assert that there are changes to the config
        Transaction::assert(update_ith_validator_info_(&mut validator_vec_ref.validators, validator_index), 42);
        let validator_info = Vector::borrow_mut(&mut validator_vec_ref.validators, validator_index);
        *&mut validator_info.last_force_update_time = current_block_time;
        emit_reconfiguration_event();
    }

    // This function can be invoked by the LibraAssociation or by the 0x0 LibraVM address in
    // block_prologue to facilitate reconfigurations at regular intervals.
    // Here for all of the validators the information from ValidatorConfig2 will get copied into the ValidatorSet,
    // if any components of the configs have changed, appropriate events will be fired.
    public fun update_all_validator_info() acquires ValidatorSet {
        Transaction::assert(Transaction::sender() == 0xA550C18 || Transaction::sender() == 0x0, 1);
        let validators = &mut borrow_global_mut<ValidatorSet>(0x1D8).validators;

        let size = Vector::length(validators);
        if (size == 0) {
            return
        };

        let i = 0;
        let configs_changed = false;
        configs_changed = configs_changed || update_ith_validator_info_(validators, i);
        loop {
            i = i + 1;
            if (i >= size) { break };
            configs_changed = configs_changed || update_ith_validator_info_(validators, i);
        };
        Transaction::assert(configs_changed, 42);
        emit_reconfiguration_event();
    }

    // Public getters

    // Return true if addr is a current validator
    public fun is_validator(addr: &address): bool acquires ValidatorSet { is_validator_(addr, &borrow_global<ValidatorSet>(0x1D8).validators) }

    // Returns validator info
    // If the address is not a validator, abort
    public fun get_validator_info(addr: &address): ValidatorInfo acquires ValidatorSet {
        let validator_set_ref = borrow_global<ValidatorSet>(0x1D8);
        let validator_index_vec = get_validator_index_(&validator_set_ref.validators, addr);
        Transaction::assert(!Vector::is_empty(&validator_index_vec), 19);

        *Vector::borrow(&validator_set_ref.validators, *Vector::borrow(&validator_index_vec, 0))
    }

    // Return the size of the current validator set
    public fun validator_set_size(): u64 acquires ValidatorSet {
        Vector::length(&borrow_global<ValidatorSet>(0x1D8).validators)
    }

    // Private helper functions

    // Emit a reconfiguration event. This function will be invoked by the genesis to generate the very first
    // reconfiguration event.
    fun emit_reconfiguration_event() acquires ValidatorSet {
        let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);

        // Ensure only one reconfiguration event is emitted per block
        let current_block_time = LibraTimestamp::now_microseconds();

        // Abort the transaction if reconfiguration was already called within the same block
        Transaction::assert(current_block_time > validator_set_ref.last_reconfiguration_time, 14);

        validator_set_ref.last_reconfiguration_time = current_block_time;
        LibraAccount::emit_event<ValidatorSetChangeEvent>(
            &mut validator_set_ref.change_events,
            ValidatorSetChangeEvent {
                new_validator_set: *&validator_set_ref.validators,
            });
    }

    // Get the index of the validator by address in the `validators` vector
    fun get_validator_index_(validators: &vector<ValidatorInfo>, addr: &address): vector<u64> {
        let size = Vector::length(validators);
        let result: vector<u64> = Vector::empty();
        if (size == 0) {
            return result
        };

        let i = 0;
        let validator_info_ref = Vector::borrow(validators, i);
        loop {
            if (&validator_info_ref.addr == addr) {
                Vector::push_back(&mut result, i);
                return result
            };
            i = i + 1;
            if (i >= size) { break };
            validator_info_ref = Vector::borrow(validators, i);
        };

        result
    }

    // Updates ith validator info, if nothing changed, return false
    fun update_ith_validator_info_(validators: &mut vector<ValidatorInfo>, i: u64): bool {
        let size = Vector::length(validators);
        if (i >= size) {
            return false
        };
        let validator_info = Vector::borrow(validators, i);
        let new_validator_config = ValidatorConfig2::get_config(validator_info.addr);
        // check if information is the same
        let config_ref = &mut Vector::borrow_mut(validators, i).config;
        if (config_ref == &new_validator_config) {
            return false
        };
        *config_ref = *&new_validator_config;
        true
    }

    fun is_validator_(addr: &address, validators_vec_ref: &vector<ValidatorInfo>): bool {
        !Vector::is_empty(&get_validator_index_(validators_vec_ref, addr))
    }
}
