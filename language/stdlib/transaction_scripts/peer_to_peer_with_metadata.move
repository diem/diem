script {
use 0x1::LibraAccount;

/// Transfer `amount` coins to `recipient_address` with (optional)
/// associated metadata `metadata` and (optional) `signature` on the metadata, amount, and
/// sender address. The `metadata` and `signature` parameters are only required if
/// `amount` >= 1_000_000 micro LBR and the sender and recipient of the funds are two distinct VASPs.
/// Fails if there is no account at the recipient address or if the sender's balance is lower
/// than `amount`.
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
