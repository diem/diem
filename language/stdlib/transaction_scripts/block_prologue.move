// Script for running block prologue.
// Will disappear once we run the block prologue from a BlockMetadataTransaction/make the block
// prologue  public

fun main(
    timestamp: u64,
    new_block_hash: vector<u8>,
    previous_block_votes: vector<u8>,
    proposer: address
) {
  0x0::LibraSystem::block_prologue(
      timestamp,
      new_block_hash,
      previous_block_votes,
      proposer
  )
}
