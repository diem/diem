address 0x0:

module LibraSystem {
    use 0x0::LibraAccount;
    use 0x0::ValidatorConfig;
    use 0x0::Vector;
    use 0x0::U64Util;
    use 0x0::AddressUtil;
    use 0x0::LibraTimestamp;
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

    resource struct BlockMetadata {
      // Height of the current block
      // TODO: Should we keep the height?
      height: u64,

      // Hash of the current block of transactions.
      id: vector<u8>,

      // Proposer of the current block.
      proposer: address,
    }

    resource struct TransactionFees {
        fee_withdrawal_capability: LibraAccount::WithdrawalCapability,
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

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize_block_metadata() {
      // Only callable by the Association address
      Transaction::assert(Transaction::sender() == 0xA550C18, 1);

      // TODO: How should we get the default block metadata? Should it be set in the first block prologue transaction or
      //       in the genesis?
      move_to_sender<BlockMetadata>(BlockMetadata {
        height: 0,
        // FIXME: Update this once we have vector<u8> literals
        id: U64Util::u64_to_bytes(0),
        proposer: 0xA550C18,
      });
      LibraTimestamp::initialize_timer();
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

    // Set the metadata for the current block.
    // The runtime always runs this before executing the transactions in a block.
    // TODO: 1. Make this private, support other metadata
    //       2. Should the previous block votes be provided from BlockMetadata or should it come from the ValidatorSet
    //          Resource?
    public fun block_prologue(
        timestamp: u64,
        new_block_hash: vector<u8>,
        previous_block_votes: vector<u8>,
        proposer: address
    ) acquires BlockMetadata, ValidatorSet, DiscoverySet, TransactionFees {
      process_block_prologue(timestamp, new_block_hash, previous_block_votes, proposer);

      // Currently distribute once per-block.
      // TODO: Once we have a better on-chain representation of epochs we will make this per-epoch.
      distribute_transaction_fees();

      // triggers a reconfiguration if the validator keys or validator set has changed
      reconfigure();
    }

    // Update the BlockMetadata resource with the new blockmetada coming from the consensus.
    fun process_block_prologue(
        timestamp: u64,
        new_block_hash: vector<u8>,
        previous_block_votes: vector<u8>,
        proposer: address
    ) acquires BlockMetadata, ValidatorSet {
        let block_metadata_ref = borrow_global_mut<BlockMetadata>(0xA550C18);

        // TODO: Figure out a story for errors in the system transactions.
        if(proposer != 0x0) Transaction::assert(is_validator(proposer), 5002);
        LibraTimestamp::update_global_time(proposer, timestamp);

        block_metadata_ref.id = new_block_hash;
        block_metadata_ref.proposer = proposer;
        block_metadata_ref.height = block_metadata_ref.height + 1;
    }

    // Get the current block height
    public fun get_current_block_height(): u64 acquires BlockMetadata {
      borrow_global<BlockMetadata>(0xA550C18).height
    }

    // Get the current block id
    public fun get_current_block_id(): vector<u8> acquires BlockMetadata {
      *&borrow_global<BlockMetadata>(0xA550C18).id
    }

    // Gets the address of the proposer of the current block
    public fun get_current_proposer(): address acquires BlockMetadata {
      borrow_global<BlockMetadata>(0xA550C18).proposer
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
    fun reconfigure() acquires ValidatorSet, DiscoverySet {
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

    ///////////////////////////////////////////////////////////////////////////
    // Transaction Fee Distribution
    ///////////////////////////////////////////////////////////////////////////
    // Implements a basic transaction fee distribution logic.
    //
    // We have made a couple design decisions here that are worth noting:
    //  * We pay out once per-block for now.
    //    TODO: Once we have a better on-chain representation of
    //          epochs this should be changed over to be once per-epoch.
    //  * Sometimes the number of validators does not evenly divide the transaction fees to be
    //    distributed. In such cases the remainder ("dust") is left in the transaction fees pot and
    //    these remaining fees will be included in the calculations for the transaction fee
    //    distribution in the next epoch. This distribution strategy is meant to in part minimize the
    //    benefit of being the first validator in the validator set.

     // Initialize the transaction fee distribution module. We keep track of the last paid block
     // height in order to ensure that we don't try to pay more than once per-block. We also
     // encapsulate the withdrawal capability to the transaction fee account so that we can withdraw
     // the fees from this account from block metadata transactions.
     fun initialize_transaction_fees() {
         Transaction::assert(Transaction::sender() == 0xFEE, 0);
         move_to_sender<TransactionFees>(TransactionFees {
             fee_withdrawal_capability: LibraAccount::extract_sender_withdrawal_capability(),
         });
     }

     fun distribute_transaction_fees() acquires TransactionFees, ValidatorSet {
       let num_validators = validator_set_size();
       let amount_collected = LibraAccount::balance(0xFEE);

       // If amount_collected == 0, this will also return early
       if (amount_collected < num_validators) return ();

       // Calculate the amount of money to be dispursed, along with the remainder.
       let amount_to_distribute_per_validator = per_validator_distribution_amount(
           amount_collected,
           num_validators
       );

       // Iterate through the validators distributing fees equally
       distribute_transaction_fees_internal(
           amount_to_distribute_per_validator,
           num_validators,
       );
     }

     // After the book keeping has been performed, this then distributes the
     // transaction fees equally to all validators with the exception that
     // any remainder (in the case that the number of validators does not
     // evenly divide the transaction fee pot) is distributed to the first
     // validator.
     fun distribute_transaction_fees_internal(
         amount_to_distribute_per_validator: u64,
         num_validators: u64
     ) acquires ValidatorSet, TransactionFees {
         let distribution_resource = borrow_global<TransactionFees>(0xFEE);
         let index = 0;

         while (index < num_validators) {

             let addr = get_ith_validator_address(index);
             // Increment the index into the validator set.
             index = index + 1;

             LibraAccount::pay_from_capability(
                 addr,
                 &distribution_resource.fee_withdrawal_capability,
                 amount_to_distribute_per_validator,
                 // FIXME: Update this once we have vector<u8> literals
                 AddressUtil::address_to_bytes(0xFEE),
             );
         }
     }

     // This calculates the amount to be distributed to each validator equally. We do this by calculating
     // the integer division of the transaction fees collected by the number of validators. In
     // particular, this means that if the number of validators does not evenly divide the
     // transaction fees collected, then there will be a remainder that is left in the transaction
     // fees pot to be distributed later.
     fun per_validator_distribution_amount(amount_collected: u64, num_validators: u64): u64 {
         Transaction::assert(num_validators != 0, 0);
         let validator_payout = amount_collected / num_validators;
         Transaction::assert(validator_payout * num_validators <= amount_collected, 1);
         validator_payout
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
}
