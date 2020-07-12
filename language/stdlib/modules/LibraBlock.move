address 0x1 {

module LibraBlock {
    use 0x1::CoreAddresses;
    use 0x1::Event;
    use 0x1::LibraSystem;
    use 0x1::LibraConfig;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;

    resource struct BlockMetadata {
        /// Height of the current block
        /// TODO: should we keep the height?
        height: u64,
        /// Handle where events with the time of new blocks are emitted
        new_block_events: Event::EventHandle<Self::NewBlockEvent>,
    }

    struct NewBlockEvent {
        round: u64,
        proposer: address,
        previous_block_votes: vector<address>,

        /// On-chain time during  he block at the given height
        time_microseconds: u64,
    }

    const ENOT_GENESIS: u64 = 0;
    const EINVALID_SINGLETON_ADDRESS: u64 = 1;
    const ESENDER_NOT_VM: u64 = 2;
    const EPROPOSER_NOT_A_VALIDATOR: u64 = 3;

    /// This can only be invoked by the Association address, and only a single time.
    /// Currently, it is invoked in the genesis transaction
    public fun initialize_block_metadata(account: &signer) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        // Operational constraint, only callable by the Association address
        assert(Signer::address_of(account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);

        move_to<BlockMetadata>(
            account,
            BlockMetadata {
                height: 0,
                new_block_events: Event::new_event_handle<Self::NewBlockEvent>(account),
            }
        );
    }

    spec fun initialize_block_metadata {
        aborts_if !LibraTimestamp::spec_is_genesis();
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS();
        aborts_if spec_is_initialized();
        ensures spec_is_initialized();
        ensures spec_block_height() == 0;
    }

    spec module {
        define spec_is_initialized(): bool {
            exists<BlockMetadata>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS())
        }
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
        // Can only be invoked by LibraVM privilege.
        assert(Signer::address_of(vm) == CoreAddresses::VM_RESERVED_ADDRESS(), ESENDER_NOT_VM);

        process_block_prologue(vm,  round, timestamp, previous_block_votes, proposer);

        // TODO(valerini): call regular reconfiguration here LibraSystem2::update_all_validator_info()
    }

    spec fun block_prologue {
        aborts_if Signer::spec_address_of(vm) != CoreAddresses::SPEC_VM_RESERVED_ADDRESS();
        ensures LibraTimestamp::spec_now_microseconds() == timestamp;
        ensures spec_block_height() == old(spec_block_height()) + 10;
    }

    spec module {
        define spec_block_height(): u64 {
            global<BlockMetadata>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS()).height
        }
    }

    /// Update the BlockMetadata resource with the new blockmetada coming from the consensus.
    fun process_block_prologue(
        vm: &signer,
        round: u64,
        timestamp: u64,
        previous_block_votes: vector<address>,
        proposer: address
    ) acquires BlockMetadata {
        let block_metadata_ref = borrow_global_mut<BlockMetadata>(CoreAddresses::LIBRA_ROOT_ADDRESS());

        if(proposer != CoreAddresses::VM_RESERVED_ADDRESS()) assert(LibraSystem::is_validator(proposer), EPROPOSER_NOT_A_VALIDATOR);
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
    }

    spec fun process_block_prologue {
        pragma assume_no_abort_from_here = true, opaque = true;
        aborts_if !spec_is_initialized();
        aborts_if proposer != CoreAddresses::SPEC_VM_RESERVED_ADDRESS()
            && !LibraConfig::spec_is_published<LibraSystem::LibraSystem>();
        aborts_if proposer != CoreAddresses::SPEC_VM_RESERVED_ADDRESS()
            && !LibraSystem::spec_is_validator(proposer);
        aborts_if spec_block_height() + 1  > max_u64();
        ensures LibraTimestamp::spec_now_microseconds() == timestamp;
        ensures spec_block_height() == old(spec_block_height()) + 1;
    }

    /// Get the current block height
    public fun get_current_block_height(): u64 acquires BlockMetadata {
        borrow_global<BlockMetadata>(CoreAddresses::LIBRA_ROOT_ADDRESS()).height
    }

    spec fun get_current_block_height {
        aborts_if !spec_is_initialized();
        ensures result == spec_block_height();
    }

    // **************** FUNCTION SPECIFICATIONS ****************

    spec module {
        pragma verify = true;
    }
}

}
