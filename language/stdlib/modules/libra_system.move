address 0x0:

module LibraSystem {
    use 0x0::LibraAccount;
    use 0x0::ValidatorConfig;
    use 0x0::Vector;
    use 0x0::Transaction;

    struct ValidatorInfo {
        addr: address,
        consensus_pubkey: vector<u8>,
        consensus_voting_power: u64,
        network_signing_pubkey: vector<u8>,
        network_identity_pubkey: vector<u8>,
    }

    struct ValidatorSetChangeEvent {
        new_validator_set: vector<ValidatorInfo>,
    }

    resource struct ValidatorSet {
        // The current validator set. Updated only at epoch boundaries via reconfiguration.
        validators: vector<ValidatorInfo>,
        // List of validators to add at the next reconfiguration.
        additions: vector<address>,
        // List of validators to removal at the next reconfiguration. Disjoint with additions.
        removals: vector<address>,
        // Handle where validator set change events are emitted
        change_events: LibraAccount::EventHandle<ValidatorSetChangeEvent>,
    }

    struct DiscoveryInfo {
        addr: address,
        validator_network_identity_pubkey: vector<u8>,
        validator_network_address: vector<u8>,
        fullnodes_network_identity_pubkey: vector<u8>,
        fullnodes_network_address: vector<u8>,
    }

    struct DiscoverySetChangeEvent {
        new_discovery_set: vector<DiscoveryInfo>,
    }

    resource struct DiscoverySet {
        // The current discovery set. Updated only at epoch boundaries via reconfiguration.
        discovery_set: vector<DiscoveryInfo>,
        // Handle where discovery set change events are emitted
        change_events: LibraAccount::EventHandle<DiscoverySetChangeEvent>,
    }

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize_validator_set() {
      // Only callable by the validator set address
      Transaction::assert(Transaction::sender() == 0x1D8, 1);

      move_to_sender<ValidatorSet>(ValidatorSet {
          validators: Vector::empty(),
          additions: Vector::empty(),
          removals: Vector::empty(),
          change_events: LibraAccount::new_event_handle<ValidatorSetChangeEvent>(),
      });
    }

    public fun initialize_discovery_set() {
        // Only callable by the discovery set address
        Transaction::assert(Transaction::sender() == 0xD15C0, 1);

        move_to_sender<DiscoverySet>(DiscoverySet {
            discovery_set: Vector::empty(),
            change_events: LibraAccount::new_event_handle<DiscoverySetChangeEvent>(),
        });
    }

    // ValidatorInfo public accessors

    public fun get_validator_address(v: &ValidatorInfo): &address {
      &v.addr
    }

    public fun get_consensus_pubkey(v: &ValidatorInfo): &vector<u8> {
      &v.consensus_pubkey
    }

    public fun get_consensus_voting_power(v: &ValidatorInfo): &u64 {
      &v.consensus_voting_power
    }

    public fun get_network_signing_pubkey(v: &ValidatorInfo): &vector<u8> {
      &v.network_signing_pubkey
    }

    public fun get_network_identity_pubkey(v: &ValidatorInfo): &vector<u8> {
      &v.network_identity_pubkey
    }

    // DiscoveryInfo public accessors

    public fun get_discovery_address(d: &DiscoveryInfo): &address {
        &d.addr
    }

    public fun get_validator_network_identity_pubkey(d: &DiscoveryInfo): &vector<u8> {
        &d.validator_network_identity_pubkey
    }

    public fun get_validator_network_address(d: &DiscoveryInfo): &vector<u8> {
        &d.validator_network_address
    }

    public fun get_fullnodes_network_identity_pubkey(d: &DiscoveryInfo): &vector<u8> {
        &d.fullnodes_network_identity_pubkey
    }

    public fun get_fullnodes_network_address(d: &DiscoveryInfo): &vector<u8> {
        &d.fullnodes_network_address
    }

    // Return the size of the current validator set
    public fun validator_set_size(): u64 acquires ValidatorSet {
        Vector::length(&borrow_global<ValidatorSet>(0x1D8).validators)
    }

   fun is_validator_(addr: &address, validators_vec_ref: &vector<ValidatorInfo>): bool {
        let size = Vector::length(validators_vec_ref);
        if (size == 0) {
            return false
        };

        let i = 0;
        // this is only needed to make the bytecode verifier happy
        let validator_info_ref = Vector::borrow(validators_vec_ref, i);
        loop {
            if (&validator_info_ref.addr == addr) {
                return true
            };
            i = i + 1;
            if (i >= size) break;
            validator_info_ref = Vector::borrow(validators_vec_ref, i);
        };

        false
    }

    // Return true if addr is a current validator
    public fun is_validator(addr: address): bool acquires ValidatorSet {
        is_validator_(&addr, &borrow_global<ValidatorSet>(0x1D8).validators)
    }

    // Get the ValidatorInfo for the ith validator
    public fun get_ith_validator_info(i: u64): ValidatorInfo acquires ValidatorSet {
      let validators_vec_ref = &borrow_global<ValidatorSet>(0x1D8).validators;
      Transaction::assert(i < Vector::length(validators_vec_ref), 3);
      *Vector::borrow(validators_vec_ref, i)
    }

    // Get the address of the i'th validator.
    public fun get_ith_validator_address(i: u64): address acquires ValidatorSet {
      let validator_set = borrow_global<ValidatorSet>(0x1D8);
      let len = Vector::length(&validator_set.validators);
      Transaction::assert(i < len, 3);
      Vector::borrow(&validator_set.validators, i).addr
    }

    // Get the DiscoveryInfo for the ith validator
    public fun get_ith_discovery_info(i: u64): DiscoveryInfo acquires DiscoverySet {
        let discovery_vec_ref = &borrow_global<DiscoverySet>(0xD15C0).discovery_set;
        Transaction::assert(i < Vector::length(discovery_vec_ref), 4);
        *Vector::borrow(discovery_vec_ref, i)
    }

    // Get the index of the validator with address `addr` in `validators`.
    // Aborts if `addr` is not the address of any validator
    public fun get_validator_index(validators: &vector<ValidatorInfo>, addr: address): u64 {
        let len = Vector::length(validators);
        let i = 0;
        loop {
            if (get_validator_address(Vector::borrow(validators, i)) == &addr) {
                return i
            };

            i = i + 1;
            if (i >= len) break;
        };

        abort 99
    }

    // Adds a validator to the addition buffer, which will cause it to be added to the validator
    // set in the next epoch.
    // Fails if `account_address` is already a validator or has already been added to the addition
    // buffer.
    // Only callable by the Association address
    public fun add_validator(account_address: address) acquires ValidatorSet {
        // Only the Association can add new validators
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        // A prospective validator must have a validator config resource
        Transaction::assert(ValidatorConfig::has(account_address), 17);

        let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);
        let validator_set_additions = &mut validator_set_ref.additions;
        // Ensure that this address is not already a validator
        Transaction::assert(
            !is_validator_(&account_address, &validator_set_ref.validators),
            18
        );
        // Ensure that this is not already an addition
        Transaction::assert(
            !Vector::contains(
                validator_set_additions,
                &account_address,
            ),
            19
        );

        // Add to candidates
        Vector::push_back(
            validator_set_additions,
            account_address,
        );
    }

    public fun remove_validator(account_address: address) acquires ValidatorSet {
        // Only the Association can remove validators
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);

        let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);
        // Ensure that this address is already a validator
        Transaction::assert(
            is_validator_(&account_address, &validator_set_ref.validators),
            21
        );
        // Ensure that this is not already a removal
        Transaction::assert(
            !Vector::contains(
                &validator_set_ref.removals,
                &account_address,
            ),
            22
        );

        // Add to removals
        Vector::push_back(
            &mut validator_set_ref.removals,
            account_address,
        );
    }

    // Return true if the ValidatorInfo given as input is different than the one
    // derived from the ValidatorConfig published at validator_info.addr + copies
    // the differing fields. Aborts if there is no ValidatorConfig at
    // validator_info.addr
    public fun copy_validator_info(validator_info: &mut ValidatorInfo): bool {

        let config = ValidatorConfig::config(validator_info.addr);
        let consensus_pubkey = ValidatorConfig::consensus_pubkey(&config);
        let network_signing_pubkey = ValidatorConfig::validator_network_signing_pubkey(&config);
        let network_identity_pubkey = ValidatorConfig::validator_network_identity_pubkey(&config);

        let changed = false;
        if (&consensus_pubkey != &validator_info.consensus_pubkey) {
            *&mut validator_info.consensus_pubkey = consensus_pubkey;
            changed = true;
        };
        if (&network_signing_pubkey != &validator_info.network_signing_pubkey) {
            *&mut validator_info.network_signing_pubkey = network_signing_pubkey;
            changed = true;
        };
        if (&network_identity_pubkey != &validator_info.network_identity_pubkey) {
            *&mut validator_info.network_identity_pubkey = network_identity_pubkey;
            changed = true;
        };
        changed
    }

    // Create a ValidatorInfo from the ValidatorConfig stored at addr.
    // Aborts if addr does not have a ValidatorConfig
    fun make_validator_info(addr: address): ValidatorInfo {
        let config = ValidatorConfig::config(addr);

       ValidatorInfo {
           addr: addr,
           consensus_pubkey: ValidatorConfig::consensus_pubkey(&config),
           consensus_voting_power: 1,
           network_signing_pubkey: ValidatorConfig::validator_network_signing_pubkey(&config),
           network_identity_pubkey: ValidatorConfig::validator_network_identity_pubkey(&config),
       }
    }

    // Return true if the DiscoveryInfo given as input is different than the one
    // derived from the ValidatorConfig published at discovery_info.addr + copies
    // the differing fields. Aborts if there is no ValidatorConfig at
    // discovery_info.addr
    public fun copy_discovery_info(discovery_info: &mut DiscoveryInfo): bool {
        let config = ValidatorConfig::config(*&discovery_info.addr);
        let validator_network_identity_pubkey = ValidatorConfig::validator_network_identity_pubkey(&config);
        let validator_network_address = ValidatorConfig::validator_network_address(&config);
        let fullnodes_network_identity_pubkey = ValidatorConfig::fullnodes_network_identity_pubkey(&config);
        let fullnodes_network_address = ValidatorConfig::fullnodes_network_address(&config);

        let changed = false;
        if (&validator_network_identity_pubkey != &discovery_info.validator_network_identity_pubkey) {
            *&mut discovery_info.validator_network_identity_pubkey = validator_network_identity_pubkey;
            changed = true;
        };
        if (&validator_network_address != &discovery_info.validator_network_address) {
            *&mut discovery_info.validator_network_address = validator_network_address;
            changed = true;
        };
        if (&fullnodes_network_identity_pubkey != &discovery_info.fullnodes_network_identity_pubkey) {
            *&mut discovery_info.fullnodes_network_identity_pubkey = fullnodes_network_identity_pubkey;
            changed = true;
        };
        if (&fullnodes_network_address != &discovery_info.fullnodes_network_address) {
            *&mut discovery_info.fullnodes_network_address = fullnodes_network_address;
            changed = true;
        };

        changed
    }

    // Create a DiscoveryInfo from the ValidatorConfig stored at addr.
    // Aborts if addr does not have a ValidatorConfig
    fun make_discovery_info(addr: address): DiscoveryInfo {
        let config = ValidatorConfig::config(addr);

       DiscoveryInfo {
           addr: addr,
           validator_network_identity_pubkey:
               ValidatorConfig::validator_network_identity_pubkey(&config),
           validator_network_address:
               ValidatorConfig::validator_network_address(&config),
           fullnodes_network_identity_pubkey:
               ValidatorConfig::fullnodes_network_identity_pubkey(&config),
           fullnodes_network_address:
               ValidatorConfig::fullnodes_network_address(&config),
       }
    }

    // Trigger a reconfiguation the Libra system by:
    // (1) Checking if there have been any additions since the last reconfiguration (and adding
    //     the validator's config if so)
    // (2) Checking if there have been any removals since the last reconfiguration (and removing
    //     the validator if so)
    // (3) Checking if there have been any key rotations since the last reconfiguration (and
    //     updating the key info if so)
    // (4) Emitting an event containing new validator set or discovery set, which will be
    //     passed to the executor
    public fun reconfigure() acquires ValidatorSet, DiscoverySet {
        // Only callable by the VM
        Transaction::assert(Transaction::sender() == 0x0, 1);

        // For now, this only supports a simple form of reconfiguration: allowing a fixed set of
        // validators to rotate their keys.
        // TODO: support adding and removing validators. Eventually, we will do this by computing
        // the new validator set from a larger list of candidate validators sorted by stake.
        let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);
        let additions_ref = &mut validator_set_ref.additions;
        let removals_ref = &mut validator_set_ref.removals;
        let validators_vec_ref = &mut validator_set_ref.validators;
        let discovery_set_ref = borrow_global_mut<DiscoverySet>(0xD15C0);
        let discovery_vec_ref = &mut discovery_set_ref.discovery_set;
        let validator_set_changed = false;
        let discovery_set_changed = false;

        // (1) Check for additions
        let i = 0;
        let len = Vector::length(additions_ref);
        // if additions is nonempty, we have an addition to process
        if (len > 0) {
            loop {
                // remove validator address from additions and add corresponding ValidatorInfo to
                // the validator set and DiscoveryInfo to discovery set
                let addr = Vector::pop_back(additions_ref);
                Vector::push_back(
                    validators_vec_ref,
                    make_validator_info(addr)
                );
                Vector::push_back(
                    discovery_vec_ref,
                    make_discovery_info(addr)
                );
                i = i + 1;
                if (i >= len) break;
            };
            // ensures additions.length == 0
            // ensures validators.length == old(validators).length + old(additions).length
            // ensures discovery_set.length == old(discovery_set).length + old(additions).length
            validator_set_changed = true;
            discovery_set_changed = true;
        };

        // (2) Check for removals
        i = 0;
        len = Vector::length(removals_ref);
        if (len > 0) {
            loop {
                // remove validator address from removals
                let to_remove_index = get_validator_index(
                    validators_vec_ref,
                    Vector::pop_back(removals_ref)
                );
                // remove corresponding ValidatorInfo from the validator set
                _  = Vector::swap_remove(
                    validators_vec_ref,
                    to_remove_index
                );
                // remove corresponding DiscoveryInfo from the discovery set
                _ = Vector::swap_remove(
                    discovery_vec_ref,
                    to_remove_index,
                );
                i = i + 1;
                if (i >= len) break;
            };
            // ensures removals.length == 0
            // ensures validators.length == old(validators).length - old(removals).length
            // ensures discovery_set.length == old(discovery_set).length - old(removals).length
            validator_set_changed = true;
            discovery_set_changed = true;
        };

        // (3) Check for key rotations in the validator set
        i = 0;
        len = Vector::length(validators_vec_ref);
        // assume(len > 0), since an empty validator set is nonsensical
        let validator_info_ref = Vector::borrow_mut(validators_vec_ref, i);
        // check if each validator has rotated their keys, copy their new info and note the change
        // if so.
        loop {
            if (copy_validator_info(validator_info_ref)) {
                validator_set_changed = true;
            };

            i = i + 1;
            if (i >= len) break;
            validator_info_ref = Vector::borrow_mut(validators_vec_ref, i);
        };

        // (4) Check for changes in the discovery set
        i = 0;
        len = Vector::length(discovery_vec_ref);
        // assume(len > 0), since an empty validator set is nonsensical
        let discovery_info_ref = Vector::borrow_mut(discovery_vec_ref, i);
        // check if each validator has changed their discovery info, copy their new info, and
        // note the change if so.
        loop {
            if (copy_discovery_info(discovery_info_ref)) {
                discovery_set_changed = true;
            };

            i = i + 1;
            if (i >= len) break;
            discovery_info_ref = Vector::borrow_mut(discovery_vec_ref, i);
        };

        // (5) Emit validator set changed event if appropriate
        if (validator_set_changed) {
            LibraAccount::emit_event<ValidatorSetChangeEvent>(
                &mut validator_set_ref.change_events,
                ValidatorSetChangeEvent {
                    new_validator_set: *validators_vec_ref,
                },
            );
        };

        // (6) Emit discovery set changed event if appropriate
        if (discovery_set_changed) {
            LibraAccount::emit_event<DiscoverySetChangeEvent>(
                &mut discovery_set_ref.change_events,
                DiscoverySetChangeEvent {
                    new_discovery_set: *discovery_vec_ref,
                },
            );
        };
    }
}
