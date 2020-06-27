script {
use 0x1::LibraAccount;

/// Mint `amount_lbr` LBR from the sending account's constituent coins and deposits the
/// resulting LBR into the sending account.
fun mint_lbr(account: &signer, amount_lbr: u64) {
    let withdraw_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::staple_lbr(&withdraw_cap, amount_lbr);
    LibraAccount::restore_withdraw_capability(withdraw_cap)
}
}
