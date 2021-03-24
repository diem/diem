//! account: bob, 1000000

script {
use 0x1::XUS::XUS;
use 0x1::DiemAccount;
use 0x1::Signer;

fun main(sender: signer) {
    let sender = &sender;
    let sender_addr = Signer::address_of(sender);
    let recipient_addr = {{bob}};
    let sender_original_balance = DiemAccount::balance<XUS>(sender_addr);
    let recipient_original_balance = DiemAccount::balance<XUS>(recipient_addr);
    let with_cap = DiemAccount::extract_withdraw_capability(sender);
    DiemAccount::pay_from<XUS>(&with_cap, recipient_addr, 5, x"", x"");
    DiemAccount::restore_withdraw_capability(with_cap);

    let sender_new_balance = DiemAccount::balance<XUS>(sender_addr);
    let recipient_new_balance = DiemAccount::balance<XUS>(recipient_addr);
    assert(sender_new_balance == sender_original_balance - 5, 77);
    assert(recipient_new_balance == recipient_original_balance + 5, 77);
}
}
