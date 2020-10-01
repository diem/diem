//! account: bob, 1000000

script {
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
use 0x1::Signer;

fun main(sender: &signer) {
    let sender_addr = Signer::address_of(sender);
    let recipient_addr = {{bob}};
    let sender_original_balance = LibraAccount::balance<Coin1>(sender_addr);
    let recipient_original_balance = LibraAccount::balance<Coin1>(recipient_addr);
    let with_cap = LibraAccount::extract_withdraw_capability(sender);
    LibraAccount::pay_from<Coin1>(&with_cap, recipient_addr, 5, x"", x"");
    LibraAccount::restore_withdraw_capability(with_cap);

    let sender_new_balance = LibraAccount::balance<Coin1>(sender_addr);
    let recipient_new_balance = LibraAccount::balance<Coin1>(recipient_addr);
    assert(sender_new_balance == sender_original_balance - 5, 77);
    assert(recipient_new_balance == recipient_original_balance + 5, 77);
}
}
