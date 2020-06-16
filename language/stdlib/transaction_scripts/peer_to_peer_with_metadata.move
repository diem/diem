script {
use 0x1::LibraAccount;

fun peer_to_peer_with_metadata<Token>(
    payer: &signer,
    payee: address,
    amount: u64,
    metadata: vector<u8>,
    metadata_signature: vector<u8>
) {
  let payer_withdrawal_cap = LibraAccount::extract_withdraw_capability(payer);
  LibraAccount::pay_from_with_metadata<Token>(&payer_withdrawal_cap, payee, amount, metadata, metadata_signature);
  LibraAccount::restore_withdraw_capability(payer_withdrawal_cap);
}
}
