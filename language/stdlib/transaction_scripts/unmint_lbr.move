script {
use 0x1::LBR::{Self, LBR};
use 0x1::LibraAccount;
fun main(account: &signer, amount_lbr: u64) {
    let withdraw_cap = LibraAccount::extract_withdraw_capability(account);
    let lbr = LibraAccount::withdraw_from<LBR>(&withdraw_cap, amount_lbr);
    LibraAccount::restore_withdraw_capability(withdraw_cap);
    let (coin1, coin2) = LBR::unpack(account, lbr);
    LibraAccount::deposit_to(account, coin1);
    LibraAccount::deposit_to(account, coin2);
}
}
