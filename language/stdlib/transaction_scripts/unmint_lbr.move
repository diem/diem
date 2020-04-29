script {
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
fun main(amount_lbr: u64) {
    let sender = Transaction::sender();
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(amount_lbr);
    let (coin1, coin2) = LBR::unpack(lbr);
    LibraAccount::deposit(sender, coin1);
    LibraAccount::deposit(sender, coin2);
}
}
