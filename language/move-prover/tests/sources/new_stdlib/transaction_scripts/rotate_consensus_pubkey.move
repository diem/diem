use 0x0::LibraSystem;
fun main (new_key: vector<u8>) {
  LibraSystem::rotate_consensus_pubkey(new_key)
}
