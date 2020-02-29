fun main (new_key: vector<u8>) {
  0x0::ValidatorConfig::rotate_consensus_pubkey(new_key)
}
