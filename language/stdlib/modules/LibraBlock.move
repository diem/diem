address 0x1 {

module LibraBlock {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::Event;
    use 0x1::LibraSystem;
    use 0x1::LibraTimestamp;

    resource struct BlockMetadata {
        /// Height of the current block
        /// TODO: should we keep the height?
        height: u64,
        /// Handle where events with the time of new blocks are emitted
        new_block_events: Event::EventHandle<Self::NewBlockEvent>,
    }

    spec module {
        invariant [global] LibraTimestamp::is_operating() ==> is_initialized();
    }

    struct NewBlockEvent {
        round: u64,
        proposer: address,
        previous_block_votes: vector<address>,

        /// On-chain time during  he block at the given height
        time_microseconds: u64,
    }

    const EBLOCK_METADATA: u64 = 0;
    const ESENDER_NOT_VM: u64 = 2;
    const EVM_OR_VALIDATOR: u64 = 3;

    /// This can only be invoked by the Association address, and only a single time.
    /// Currently, it is invoked in the genesis transaction
    public fun initialize_block_metadata(account: &signer) {
        LibraTimestamp::assert_genesis();
        // Operational constraint, only callable by the Association address
        CoreAddresses::assert_libra_root(account);

        assert(!is_initialized(), Errors::already_published(EBLOCK_METADATA));
        move_to<BlockMetadata>(
            account,
            BlockMetadata {
                height: 0,
                new_block_events: Event::new_event_handle<Self::NewBlockEvent>(account),
            }
        );
    }
    spec fun initialize_block_metadata {
        include LibraTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot;
        aborts_if is_initialized() with Errors::ALREADY_PUBLISHED;
        ensures is_initialized();
        ensures get_current_block_height() == 0;
    }

    /// Helper function to determine whether this module has been initialized.
    fun is_initialized(): bool {
        exists<BlockMetadata>(CoreAddresses::LIBRA_ROOT_ADDRESS())
    }

    /// Set the metadata for the current block.
    /// The runtime always runs this before executing the transactions in a block.
    /// TODO: 1. Make this private, support other metadata
    ///       2. Should the previous block votes be provided from BlockMetadata or should it come from the ValidatorSet
    ///          Resource?
    public fun block_prologue(
        vm: &signer,
        round: u64,
        timestamp: u64,
        previous_block_votes: vector<address>,
        proposer: address
    ) acquires BlockMetadata {
        LibraTimestamp::assert_operating();
        // Operational constraint: can only be invoked by the VM.
        CoreAddresses::assert_vm(vm);

        // Authorization
        assert(
            proposer == CoreAddresses::VM_RESERVED_ADDRESS() || LibraSystem::is_validator(proposer),
            Errors::requires_address(EVM_OR_VALIDATOR)
        );

        let block_metadata_ref = borrow_global_mut<BlockMetadata>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        LibraTimestamp::update_global_time(vm, proposer, timestamp);
        block_metadata_ref.height = block_metadata_ref.height + 1;
        Event::emit_event<NewBlockEvent>(
            &mut block_metadata_ref.new_block_events,
            NewBlockEvent {
                round: round,
                proposer: proposer,
                previous_block_votes: previous_block_votes,
                time_microseconds: timestamp,
            }
        );
        // TODO(valerini): call regular reconfiguration here LibraSystem2::update_all_validator_info()
    }
    spec fun block_prologue {
        include LibraTimestamp::AbortsIfNotOperating;
        include CoreAddresses::AbortsIfNotVM{account: vm};
        aborts_if proposer != CoreAddresses::VM_RESERVED_ADDRESS() && !LibraSystem::spec_is_validator(proposer)
            with Errors::REQUIRES_ADDRESS;
        ensures LibraTimestamp::spec_now_microseconds() == timestamp;
        ensures get_current_block_height() == old(get_current_block_height()) + 1;

        /// The below counter overflow is assumed to be excluded from verification of callers.
        aborts_if [assume] get_current_block_height() + 1 > MAX_U64 with EXECUTION_FAILURE;
    }

    /// Get the current block height
    public fun get_current_block_height(): u64 acquires BlockMetadata {
        borrow_global<BlockMetadata>(CoreAddresses::LIBRA_ROOT_ADDRESS()).height
    }

    // **************** FUNCTION SPECIFICATIONS ****************

    spec module {
        pragma verify;
    }
}

}
