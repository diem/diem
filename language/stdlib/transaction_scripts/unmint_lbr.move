script {
use 0x0::LBR::{Self, LBR};
use 0x0::LibraAccount;
fun main(account: &signer, amount_lbr: u64) {
    let lbr = LibraAccount::withdraw_from<LBR>(account, amount_lbr);
    let (coin1, coin2) = LBR::unpack(account, lbr);
    LibraAccount::deposit_to(account, coin1);
    LibraAccount::deposit_to(account, coin2);
}
}
