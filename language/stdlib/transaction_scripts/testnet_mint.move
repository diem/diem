script {
use 0x1::LibraAccount;
use 0x1::Signer;

/// Send `amount` coins of type `Token` to `payee`.
fun testnet_mint<Token>(payer: &signer, payee: address, amount: u64) {
  assert(LibraAccount::exists_at(payee), 8000971);
  assert(Signer::address_of(payer) == 0xDD, 8000972);
  // Limit minting to 1B Libra at a time on testnet.
  // This is to prevent the market cap's total value from hitting u64_max due to excessive
  // minting. This will not be a problem in the production Libra system because coins will
  // be backed with real-world assets, and thus minting will be correspondingly rarer.
  // * 1000000 here because the unit is microlibra
  assert(amount <= 1000000000 * 1000000, 8000973);
  let payer_withdrawal_cap = LibraAccount::extract_withdraw_capability(payer);
  LibraAccount::pay_from<Token>(&payer_withdrawal_cap, payee, amount, x"", x"");
  LibraAccount::restore_withdraw_capability(payer_withdrawal_cap);
}
}
