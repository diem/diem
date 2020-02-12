fun main (new_key: bytearray) {
  0x0::ValidatorConfig::rotate_consensus_pubkey(new_key)
}
