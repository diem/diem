script {
use 0x1::LibraAccount;

/// Unmints `amount_lbr` LBR from the sending account into the constituent coins and deposits
/// the resulting coins into the sending account."
fun unmint_lbr(account: &signer, amount_lbr: u64) {
    let withdraw_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::unstaple_lbr(&withdraw_cap, amount_lbr);
    LibraAccount::restore_withdraw_capability(withdraw_cap);
}
}
