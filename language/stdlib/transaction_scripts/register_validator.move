// Register the sender as a candidate validator by publishing a ValidatorConfig.T resource with the
// given keys under their account

fun main(
    consensus_pubkey: vector<u8>,
    validator_network_signing_pubkey: vector<u8>,
    validator_network_identity_pubkey: vector<u8>,
    validator_network_address: vector<u8>,
    fullnodes_network_identity_pubkey: vector<u8>,
    fullnodes_network_address: vector<u8>,
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
