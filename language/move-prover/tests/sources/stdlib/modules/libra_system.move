// dep: tests/sources/stdlib/modules/libra_account.move
// dep: tests/sources/stdlib/modules/hash.move
// dep: tests/sources/stdlib/modules/lbr.move
// dep: tests/sources/stdlib/modules/lcs.move
// dep: tests/sources/stdlib/modules/libra.move
// dep: tests/sources/stdlib/modules/libra_transaction_timeout.move
// dep: tests/sources/stdlib/modules/transaction.move
// dep: tests/sources/stdlib/modules/vector.move
// dep: tests/sources/stdlib/modules/libra_time.move
// dep: tests/sources/stdlib/modules/validator_config.move
// no-verify

address 0x0:

module LibraSystem {
    use 0x0::LibraAccount;
    use 0x0::ValidatorConfig;
    use 0x0::Vector;
    use 0x0::Transaction;
    use 0x0::LibraTimestamp;

    struct ValidatorInfo {
        addr: address,
        consensus_pubkey: vector<u8>,
        consensus_voting_power: u64,
        network_signing_pubkey: vector<u8>,
        network_identity_pubkey: vector<u8>,
    }

    struct ValidatorSetChangeEvent {
        scheme: u8,
        new_validator_set: vector<ValidatorInfo>,
    }

    resource struct ValidatorSet {
        // The current consensus crypto scheme.
        scheme: u8,
        // The current validator set. Updated only at epoch boundaries via reconfiguration.
        validators: vector<ValidatorInfo>,
        // Last time when a reconfiguration happened.
        last_reconfiguration_time: u64,
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
          scheme: 0,
          validators: Vector::empty(),
          last_reconfiguration_time: 0,
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

    public fun reconfigure() acquires ValidatorSet {
        // TODO: Transform this method to user capability pattern.
        // Only the Association can emit reconfiguration event for now
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);

        reconfigure_()
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
            if (i >= size) { break };
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
            if (i >= len) { break };
        };

        abort 99
    }

    // Get the index of the discovery info with address `addr` in `discovery_set`.
    // Aborts if `addr` is not in discovery set.
    public fun get_discovery_index(discovery_set: &vector<DiscoveryInfo>, addr: address): u64 {
        let len = Vector::length(discovery_set);
        let i = 0;
        loop {
            if (get_discovery_address(Vector::borrow(discovery_set, i)) == &addr) {
                return i
            };

            i = i + 1;
            if (i >= len) { break };
        };

        abort 99
    }

    // Adds a validator to the addition buffer, which will cause it to be added to the validator
    // set in the next epoch.
    // Fails if `account_address` is already a validator or has already been added to the addition
    // buffer.
    // Only callable by the Association address
    public fun add_validator(account_address: address) acquires ValidatorSet, DiscoverySet {
        add_validator_(account_address);
        reconfigure_();
        emit_discovery_set_change();
    }

   fun add_validator_(account_address: address) acquires ValidatorSet, DiscoverySet {
       // Only the Association can add new validators
       Transaction::assert(Transaction::sender() == 0xA550C18, 1);
       // A prospective validator must have a validator config resource
       Transaction::assert(ValidatorConfig::has(account_address), 17);

       let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);
       let discovery_set_ref = borrow_global_mut<DiscoverySet>(0xD15C0);

       // Ensure that this address is not already a validator
       Transaction::assert(
           !is_validator_(&account_address, &validator_set_ref.validators),
           18
       );

       Vector::push_back(
           &mut validator_set_ref.validators,
           make_validator_info(account_address)
       );
       Vector::push_back(
           &mut discovery_set_ref.discovery_set,
           make_discovery_info(account_address)
       );
   }

   public fun remove_validator(account_address: address) acquires ValidatorSet, DiscoverySet {
       // Only the Association can remove validators
       Transaction::assert(Transaction::sender() == 0xA550C18, 1);

       let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);
       let discovery_set_ref = borrow_global_mut<DiscoverySet>(0xD15C0);
       // Ensure that this address is already a validator
       Transaction::assert(
           is_validator_(&account_address, &validator_set_ref.validators),
           21
       );

       let to_remove_index = get_validator_index(
           &validator_set_ref.validators,
           account_address
       );

       // remove corresponding ValidatorInfo from the validator set
       _  = Vector::swap_remove(
            &mut validator_set_ref.validators,
           to_remove_index
       );
       _  = Vector::swap_remove(
           &mut discovery_set_ref.discovery_set,
           to_remove_index
       );

       reconfigure_();
       emit_discovery_set_change();
   }

   public fun rotate_consensus_pubkey(consensus_pubkey: vector<u8>) acquires ValidatorSet {
       let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);
       let account_address = Transaction::sender();

       // Ensure that this address is already a validator
       Transaction::assert(
           is_validator_(&account_address, &validator_set_ref.validators),
           21
       );

       ValidatorConfig::rotate_consensus_pubkey(consensus_pubkey);

       let validator_index = get_validator_index(
           &validator_set_ref.validators,
           account_address
       );

       if(copy_validator_info(Vector::borrow_mut(&mut validator_set_ref.validators, validator_index))) {
           reconfigure_();
       }
   }

   public fun rotate_validator_network_identity_pubkey(
       validator_network_identity_pubkey: vector<u8>
   ) acquires DiscoverySet {
       let discovery_set_ref = borrow_global_mut<DiscoverySet>(0xD15C0);
       let account_address = Transaction::sender();

       ValidatorConfig::rotate_validator_network_identity_pubkey(validator_network_identity_pubkey);

       let validator_index = get_discovery_index(
           &discovery_set_ref.discovery_set,
           account_address
       );

       if(copy_discovery_info(Vector::borrow_mut(&mut discovery_set_ref.discovery_set, validator_index))) {
           emit_discovery_set_change();
       }
   }

   public fun rotate_validator_network_address(
       validator_network_address: vector<u8>
   ) acquires DiscoverySet {
       let discovery_set_ref = borrow_global_mut<DiscoverySet>(0xD15C0);
       let account_address = Transaction::sender();

       ValidatorConfig::rotate_validator_network_address(validator_network_address);

       let validator_index = get_discovery_index(
           &discovery_set_ref.discovery_set,
           account_address
       );

       if(copy_discovery_info(Vector::borrow_mut(&mut discovery_set_ref.discovery_set, validator_index))) {
           emit_discovery_set_change();
       }
   }

   fun reconfigure_() acquires ValidatorSet {
       // Do not do anything if time is not set up yet.
       if(LibraTimestamp::is_genesis()) {
           return ()
       };

       let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);

       // Ensure that there is at most one reconfiguration per transaction. This ensures that there is a 1-1
       // correspondence between system reconfigurations and emitted ReconfigurationEvents.

       let current_block_time = LibraTimestamp::now_microseconds();
       Transaction::assert(current_block_time > validator_set_ref.last_reconfiguration_time, 23);
       validator_set_ref.last_reconfiguration_time = current_block_time;

       emit_reconfiguration_event();
   }

   // Emit a reconfiguration event. This function will be invoked by the genesis to generate the very first
   // reconfiguration event.
   fun emit_reconfiguration_event() acquires ValidatorSet {
       let validator_set_ref = borrow_global_mut<ValidatorSet>(0x1D8);

       LibraAccount::emit_event<ValidatorSetChangeEvent>(
           &mut validator_set_ref.change_events,
           ValidatorSetChangeEvent {
               scheme: validator_set_ref.scheme,
               new_validator_set: *&validator_set_ref.validators,
           },
       );
   }

   fun emit_discovery_set_change() acquires DiscoverySet {
       let discovery_set_ref = borrow_global_mut<DiscoverySet>(0xD15C0);
       LibraAccount::emit_event<DiscoverySetChangeEvent>(
           &mut discovery_set_ref.change_events,
           DiscoverySetChangeEvent {
               new_discovery_set: *&discovery_set_ref.discovery_set,
           },
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
}
