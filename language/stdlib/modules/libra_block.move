address 0x0:

module LibraBlock {
    use 0x0::LBR;
    use 0x0::LibraAccount;
    use 0x0::LibraSystem;
    use 0x0::LibraTimestamp;
    use 0x0::Transaction;
    use 0x0::TransactionFee;

    resource struct BlockMetadata {
      // Height of the current block
      // TODO: should we keep the height?
      height: u64,
      // Handle where events with the time of new blocks are emitted
      new_block_events: LibraAccount::EventHandle<Self::NewBlockEvent>,
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
    public fun initialize_block_metadata() {
      // Only callable by the Association address
      Transaction::assert(Transaction::sender() == 0xA550C18, 1);

      move_to_sender<BlockMetadata>(BlockMetadata {
        height: 0,
        new_block_events: LibraAccount::new_event_handle<Self::NewBlockEvent>(),
      });
    }

    // Set the metadata for the current block.
    // The runtime always runs this before executing the transactions in a block.
    // TODO: 1. Make this private, support other metadata
    //       2. Should the previous block votes be provided from BlockMetadata or should it come from the ValidatorSet
    //          Resource?
    public fun block_prologue(
        round: u64,
        timestamp: u64,
        previous_block_votes: vector<address>,
        proposer: address
    ) acquires BlockMetadata {
        // Can only be invoked by LibraVM privilege.
        Transaction::assert(Transaction::sender() == 0x0, 33);

        process_block_prologue(round, timestamp, previous_block_votes, proposer);

        // Currently distribute once per-block.
        // TODO: Once we have a better on-chain representation of epochs we will make this per-epoch.
        TransactionFee::distribute_transaction_fees<LBR::T>();

        // TODO(valerini): call regular reconfiguration here LibraSystem2::update_all_validator_info()
    }

    // Update the BlockMetadata resource with the new blockmetada coming from the consensus.
    fun process_block_prologue(
        round: u64,
        timestamp: u64,
        previous_block_votes: vector<address>,
        proposer: address
    ) acquires BlockMetadata {
        let block_metadata_ref = borrow_global_mut<BlockMetadata>(0xA550C18);

        // TODO: Figure out a story for errors in the system transactions.
        if(proposer != 0x0) Transaction::assert(LibraSystem::is_validator(proposer), 5002);
        LibraTimestamp::update_global_time(proposer, timestamp);
        block_metadata_ref.height = block_metadata_ref.height + 1;
        LibraAccount::emit_event<NewBlockEvent>(
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
      borrow_global<BlockMetadata>(0xA550C18).height
    }
}
