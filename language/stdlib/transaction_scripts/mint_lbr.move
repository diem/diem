script {
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
fun main(amount_lbr: u64) {
    let sender = Transaction::sender();
    let coin1_balance = LibraAccount::balance<Coin1::T>(sender);
    let coin2_balance = LibraAccount::balance<Coin2::T>(sender);
    let coin1 = LibraAccount::withdraw_from_sender<Coin1::T>(coin1_balance);
    let coin2 = LibraAccount::withdraw_from_sender<Coin2::T>(coin2_balance);
    let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
    LibraAccount::deposit(sender, lbr);
    LibraAccount::deposit(sender, coin1);
    LibraAccount::deposit(sender, coin2);
}
}
