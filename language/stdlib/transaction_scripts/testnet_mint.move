script {
use 0x1::DualAttestation;
use 0x1::Libra;
use 0x1::LibraAccount;
use 0x1::Signer;

/// Send `amount` coins of type `Token` to `payee`.
fun testnet_mint<Token>(payer: &signer, payee: address, amount: u64) {
  assert(LibraAccount::exists_at(payee), 8000971);
  assert(Signer::address_of(payer) == 0xDD, 8000972);
  // Each mint amount must be under the dual attestation threshold. This is because the "minter" is
  // is a DesignatedDealer account, and the recipient (at least in the current testnet) will always
  // be a DesignatedDealer or VASP.
  assert(
      Libra::approx_lbr_for_value<Token>(amount) < DualAttestation::get_cur_microlibra_limit(),
      8000973
  );
  let payer_withdrawal_cap = LibraAccount::extract_withdraw_capability(payer);
  LibraAccount::pay_from<Token>(&payer_withdrawal_cap, payee, amount, x"", x"");
  LibraAccount::restore_withdraw_capability(payer_withdrawal_cap);
}
}
