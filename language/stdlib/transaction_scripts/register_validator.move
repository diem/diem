script {
use 0x0::ValidatorConfig;
use 0x0::LibraAccount;
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
use 0x0::Signer;

// Register the sender as a candidate validator by publishing a ValidatorConfig.T resource with the
// given keys under their account

fun main(
    account: &signer,
    consensus_pubkey: vector<u8>,
    validator_network_signing_pubkey: vector<u8>,
    validator_network_identity_pubkey: vector<u8>,
    validator_network_address: vector<u8>,
    fullnodes_network_identity_pubkey: vector<u8>,
    fullnodes_network_address: vector<u8>,
) {
  ValidatorConfig::register_candidate_validator(
      consensus_pubkey,
      validator_network_signing_pubkey,
      validator_network_identity_pubkey,
      validator_network_address,
      fullnodes_network_identity_pubkey,
      fullnodes_network_address
  );

  let sender = Signer::address_of(account);
  // Validating nodes need to accept all currencies in order to receive txn fees
  if (!LibraAccount::accepts_currency<Coin1::T>(sender)) {
      LibraAccount::add_currency<Coin1::T>(account)
  };
  if (!LibraAccount::accepts_currency<Coin2::T>(sender)) {
      LibraAccount::add_currency<Coin2::T>(account)
  };
  if (!LibraAccount::accepts_currency<LBR::T>(sender)) {
      LibraAccount::add_currency<LBR::T>(account)
  };
}
}
