address 0x1 {

module LibraBlock {
    use 0x1::CoreAddresses;
    use 0x1::Event;
    use 0x1::LibraSystem;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;

    resource struct BlockMetadata {
      // Height of the current block
      // TODO: should we keep the height?
      height: u64,
      // Handle where events with the time of new blocks are emitted
      new_block_events: Event::EventHandle<Self::NewBlockEvent>,
    }

    struct NewBlockEvent {
      round: u64,
      proposer: address,
      previous_block_votes: vector<address>,

      // On-chain time during  he block at the given height
      time_microseconds: u64,
    }

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize_block_metadata(account: &signer) {
      // Operational constraint, only callable by the Association address
      assert(Signer::address_of(account) == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), 1);

      move_to<BlockMetadata>(
          account,
          BlockMetadata {
              height: 0,
              new_block_events: Event::new_event_handle<Self::NewBlockEvent>(account),
          }
      );
    }

    // Set the metadata for the current block.
    // The runtime always runs this before executing the transactions in a block.
    // TODO: 1. Make this private, support other metadata
    //       2. Should the previous block votes be provided from BlockMetadata or should it come from the ValidatorSet
    //          Resource?
    public fun block_prologue(
        vm: &signer,
        round: u64,
        timestamp: u64,
        previous_block_votes: vector<address>,
        proposer: address
    ) acquires BlockMetadata {
        // Can only be invoked by LibraVM privilege.
        assert(Signer::address_of(vm) == CoreAddresses::VM_RESERVED_ADDRESS(), 33);

        process_block_prologue(vm,  round, timestamp, previous_block_votes, proposer);

        // TODO(valerini): call regular reconfiguration here LibraSystem2::update_all_validator_info()
    }

    // Update the BlockMetadata resource with the new blockmetada coming from the consensus.
    fun process_block_prologue(
        vm: &signer,
        round: u64,
        timestamp: u64,
        previous_block_votes: vector<address>,
        proposer: address
    ) acquires BlockMetadata {
        let block_metadata_ref = borrow_global_mut<BlockMetadata>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());

        // TODO: Figure out a story for errors in the system transactions.
        if(proposer != CoreAddresses::VM_RESERVED_ADDRESS()) assert(LibraSystem::is_validator(proposer), 5002);
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

    // Get the current block height
    public fun get_current_block_height(): u64 acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS()).height
    }
}

}
