script {
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Signer;
fun main(account: &signer, amount_lbr: u64) {
    let sender = Signer::address_of(account);
    let coin1_balance = LibraAccount::balance<Coin1::T>(sender);
    let coin2_balance = LibraAccount::balance<Coin2::T>(sender);
    let coin1 = LibraAccount::withdraw_from<Coin1::T>(account, coin1_balance);
    let coin2 = LibraAccount::withdraw_from<Coin2::T>(account, coin2_balance);
    let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
    LibraAccount::deposit_to(account, lbr);
    LibraAccount::deposit_to(account, coin1);
    LibraAccount::deposit_to(account, coin2);
}
}
