// Register the sender as a candidate validator by publishing a ValidatorConfig.T resource with the
// given keys under their account

fun main(
    consensus_pubkey: bytearray,
    validator_network_signing_pubkey: bytearray,
    validator_network_identity_pubkey: bytearray,
    validator_network_address: bytearray,
    fullnodes_network_identity_pubkey: bytearray,
    fullnodes_network_address: bytearray,
) {
  0x0::ValidatorConfig::register_candidate_validator(
      consensus_pubkey,
      validator_network_signing_pubkey,
      validator_network_identity_pubkey,
      validator_network_address,
      fullnodes_network_identity_pubkey,
      fullnodes_network_address
  )
}
