// Script for running block prologue.
// Will disappear once we run the block prologue from a BlockMetadataTransaction/make the block
// prologue  public

fun main(
    timestamp: u64,
    new_block_hash: bytearray,
    previous_block_votes: bytearray,
    proposer: address
) {
  0x0::LibraSystem::block_prologue(
      timestamp,
      new_block_hash,
      previous_block_votes,
      proposer
  )
}
