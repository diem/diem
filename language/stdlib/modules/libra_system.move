address 0x0:

// These modules only change resources stored under the three addresses:
// LibraAssociation's account address: 0xA550C18
// ValidatorSet resource address: 0x1D8
// DiscoverySet resource address: 0xD15C0
// ValidatorSet and DiscoverySet are stored under different addresses in order to facilitate
// cheaper reads of these sets from storage.
module LibraSystem {
    use 0x0::Event;
    use 0x0::LibraConfig;
    use 0x0::Transaction;
    use 0x0::ValidatorConfig;
    use 0x0::Vector;

    // ValidatorInfo stores consensus config
    // TODO(valerini): consider writing a map module (address -> ValidatorInfo),
    //                 currently implemented with Vector
    struct ValidatorInfo {
        addr: address,
        consensus_voting_power: u64,
        config: ValidatorConfig::Config,
    }

    struct T {
        // The current consensus crypto scheme.
        scheme: u8,
        // The current validator set. Updated only at epoch boundaries via reconfiguration.
        validators: vector<ValidatorInfo>,
    }

    // DiscoveryInfo stores discovery config.
    // It describes how other validators can establish a network
    // connection with this validator.
    // It also describes how other full-nodes and clients can establish
    // a network connection with this full node.
    struct DiscoveryInfo {
        addr: address,
        discovery_config: ValidatorConfig::DiscoveryConfig,
    }

    struct DiscoverySetChangeEvent {
        new_discovery_set: vector<DiscoveryInfo>,
    }

    resource struct DiscoverySet {
        // The current discovery set. Updated only at epoch boundaries via reconfiguration.
        discovery_set: vector<DiscoveryInfo>,
        // Handle where discovery set change events are emitted
        change_events: Event::EventHandle<DiscoverySetChangeEvent>,
    }

    // ValidatorInfo public accessors

    // Get validator's address from ValidatorInfo
    public fun get_validator_address(v: &ValidatorInfo): address {
        v.addr
    }

    // Get validator's config from ValidatorInfo
    public fun get_validator_config(v: &ValidatorInfo): &ValidatorConfig::Config {
        &v.config
    }

    // Get validator's voting power from ValidatorInfo
    public fun get_validator_voting_power(v: &ValidatorInfo): u64 {
        v.consensus_voting_power
    }

    // This can only be invoked by the ValidatorSet address to instantiate
    // the resource under that address.
    // It can only be called a single time. Currently, it is invoked in the genesis transaction.
    public fun initialize_validator_set() {
        Transaction::assert(Transaction::sender() == 0x1D8, 1);

        LibraConfig::publish_new_config<T>(T {
            scheme: 0,
            validators: Vector::empty(),
        });
    }

    // This returns a copy of the current validator set.
    public fun get_validator_set(): T {
        LibraConfig::get<T>(0x1D8)
    }

    // This copies the vector of validators into the LibraConfig's resource
    // under ValidatorSet address
    fun set_validator_set(value: T) {
        LibraConfig::set<T>(0x1D8, value)
    }

    // This can only be invoked by the DiscoverySet address
    // to instantiate the resource under that address.
    // It can only be called a single time. Currently, it is invoked in the genesis transaction.
    public fun initialize_discovery_set() {
        // Only callable by the discovery set address
        Transaction::assert(Transaction::sender() == 0xD15C0, 1);

        move_to_sender<DiscoverySet>(DiscoverySet {
            discovery_set: Vector::empty(),
            change_events: Event::new_event_handle<DiscoverySetChangeEvent>(),
        });
    }

    fun add_validator_no_discovery_event(account_address: address) acquires DiscoverySet {
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

        let discovery_config = ValidatorConfig::get_discovery_config(account_address);
        let discovery_set_ref = &mut borrow_global_mut<DiscoverySet>(0xD15C0).discovery_set;
        Vector::push_back(
            discovery_set_ref, DiscoveryInfo {
                addr: account_address,
                discovery_config: discovery_config
            });

        set_validator_set(validator_set);
    }

    // Adds a new validator, only callable by the LibraAssociation address
    // TODO(valerini): allow the Association to add multiple validators in a single block
    public fun add_validator(account_address: address) acquires DiscoverySet {
        add_validator_no_discovery_event(account_address);
        emit_discovery_set_change();
    }

    // Removes a validator, only callable by the LibraAssociation address
    public fun remove_validator(account_address: address) acquires DiscoverySet {
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        let validator_set = get_validator_set();

        // Ensure that this address is an active validator
        let to_remove_index_vec = get_validator_index_(&validator_set.validators, account_address);
        Transaction::assert(!Vector::is_empty(&to_remove_index_vec), 21);
        let to_remove_index = *Vector::borrow(&to_remove_index_vec, 0);
        // Remove corresponding ValidatorInfo from the validator set
        _  = Vector::swap_remove(&mut validator_set.validators, to_remove_index);

        // Remove corresponding DiscoveryInfo from the discovery set, the indices should match
        let discovery_set_ref = &mut borrow_global_mut<DiscoverySet>(0xD15C0).discovery_set;
        _  = Vector::swap_remove(discovery_set_ref, to_remove_index);

        set_validator_set(validator_set);
        emit_discovery_set_change();
    }

    // This function can be invoked by the LibraAssociation or by the 0x0 LibraVM address in
    // block_prologue to facilitate reconfigurations at regular intervals.
    // Here for all of the validators the information from ValidatorConfig will
    // get copied into the ValidatorSet.
    // Here validators' consensus_pubkey, validator's networking information and/or
    // validator's full-node information may change.
    // NewEpochEvent event and/or DiscoverySetChangeEvent will be fired.
    public fun update_and_reconfigure() acquires DiscoverySet {
        Transaction::assert(is_sender_authorized_(), 22);

        let validator_set = get_validator_set();
        let validators = &mut validator_set.validators;
        let discoveries = &mut borrow_global_mut<DiscoverySet>(0xD15C0).discovery_set;

        let size = Vector::length(validators);
        if (size == 0) {
            return
        };

        let i = 0;
        let configs_changed = false;
        let discovery_configs_changed = false;
        while (i < size) {
            let validator_info_update = update_ith_validator_info_(validators, i);
            let discovery_info_update = update_ith_discovery_info_(discoveries, i);

            configs_changed = configs_changed || validator_info_update;
            discovery_configs_changed = discovery_configs_changed || discovery_info_update;
            i = i + 1;
        };
        if (configs_changed) {
            set_validator_set(validator_set);
        };
        if (discovery_configs_changed) {
            emit_discovery_set_change();
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
    fun update_ith_discovery_info_(validators: &mut vector<DiscoveryInfo>, i: u64): bool {
        let size = Vector::length(validators);
        if (i >= size) {
            return false
        };
        let validator_info = Vector::borrow_mut(validators, i);

        let new_discovery_config = ValidatorConfig::get_discovery_config(validator_info.addr);
        let discovery_config_ref = &mut Vector::borrow_mut(validators, i).discovery_config;

        if (discovery_config_ref == &new_discovery_config) {
            return false
        };
        *discovery_config_ref = *&new_discovery_config;

        true
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

    // Discovery methods

    fun emit_discovery_set_change() acquires DiscoverySet {
        let discovery_set_ref = borrow_global_mut<DiscoverySet>(0xD15C0);
        Event::emit_event<DiscoverySetChangeEvent>(
            &mut discovery_set_ref.change_events,
            DiscoverySetChangeEvent {
                new_discovery_set: *&discovery_set_ref.discovery_set,
            },
        );
    }

    public fun get_discovery_address(d: &DiscoveryInfo): address {
        d.addr
    }

    public fun get_discovery_config(d: &DiscoveryInfo): &ValidatorConfig::DiscoveryConfig {
        &d.discovery_config
    }

    // Get the DiscoveryInfo for the ith validator
    public fun get_ith_discovery_info(i: u64): DiscoveryInfo acquires DiscoverySet {
        let discovery_vec_ref = &borrow_global<DiscoverySet>(0xD15C0).discovery_set;
        *Vector::borrow(discovery_vec_ref, i)
    }

    // Get the index of the discovery info with address `addr` in `discovery_set`.
    // Aborts if `addr` is not in discovery set.
    public fun get_discovery_index(discovery_set: &vector<DiscoveryInfo>, addr: address): u64 {
        let len = Vector::length(discovery_set);
        let i = 0;
        loop {
            if (get_discovery_address(Vector::borrow(discovery_set, i)) == addr) {
                return i
            };

            i = i + 1;
            if (i >= len) { break };
        };

        abort 99
    }
}
