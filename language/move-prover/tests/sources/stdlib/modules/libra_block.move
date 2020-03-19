// dep: tests/sources/stdlib/modules/libra_time.move
// dep: tests/sources/stdlib/modules/libra_transaction_timeout.move
// dep: tests/sources/stdlib/modules/transaction.move
// dep: tests/sources/stdlib/modules/transaction_fee.move
// dep: tests/sources/stdlib/modules/u64_util.move
// dep: tests/sources/stdlib/modules/libra_system.move
// dep: tests/sources/stdlib/modules/hash.move
// dep: tests/sources/stdlib/modules/vector.move
// dep: tests/sources/stdlib/modules/libra_coin.move
// dep: tests/sources/stdlib/modules/libra_account.move
// dep: tests/sources/stdlib/modules/validator_config.move
// dep: tests/sources/stdlib/modules/address_util.move
// no-verify

address 0x0:

module LibraBlock {
    use 0x0::LibraTimestamp;
    use 0x0::Transaction;
    use 0x0::TransactionFee;
    use 0x0::U64Util;
    use 0x0::LibraSystem;

    resource struct BlockMetadata {
      // Height of the current block
      // TODO: Should we keep the height?
      height: u64,

      // Hash of the current block of transactions.
      id: vector<u8>,

      // Proposer of the current block.
      proposer: address,
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
        // FIXME: Update this once we have byte vector literals
        id: U64Util::u64_to_bytes(0),
        proposer: 0xA550C18,
      });
      LibraTimestamp::initialize_timer();
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
    ) acquires BlockMetadata {
      // Can only be invoked by LibraVM privilege.
      Transaction::assert(Transaction::sender() == 0x0, 33);

      process_block_prologue(timestamp, new_block_hash, previous_block_votes, proposer);

      // Currently distribute once per-block.
      // TODO: Once we have a better on-chain representation of epochs we will make this per-epoch.
      TransactionFee::distribute_transaction_fees();
    }

    // Update the BlockMetadata resource with the new blockmetada coming from the consensus.
    fun process_block_prologue(
        timestamp: u64,
        new_block_hash: vector<u8>,
        previous_block_votes: vector<u8>,
        proposer: address
    ) acquires BlockMetadata {
        let block_metadata_ref = borrow_global_mut<BlockMetadata>(0xA550C18);

        // TODO: Figure out a story for errors in the system transactions.
        if(proposer != 0x0) Transaction::assert(LibraSystem::is_validator(proposer), 5002);
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
}
