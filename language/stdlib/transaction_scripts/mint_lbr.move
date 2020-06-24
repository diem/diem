script {
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::LBR;
use 0x1::LibraAccount;
use 0x1::Signer;

/// Mint `amount_lbr` LBR from the sending account's constituent coins and deposits the
/// resulting LBR into the sending account.
fun mint_lbr(account: &signer, amount_lbr: u64) {
    let sender = Signer::address_of(account);
    let coin1_balance = LibraAccount::balance<Coin1>(sender);
    let coin2_balance = LibraAccount::balance<Coin2>(sender);
    let withdraw_cap = LibraAccount::extract_withdraw_capability(account);
    let coin1 = LibraAccount::withdraw_from<Coin1>(&withdraw_cap, coin1_balance);
    let coin2 = LibraAccount::withdraw_from<Coin2>(&withdraw_cap, coin2_balance);
    LibraAccount::restore_withdraw_capability(withdraw_cap);
    let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
    LibraAccount::deposit_to(account, lbr);
    LibraAccount::deposit_to(account, coin1);
    LibraAccount::deposit_to(account, coin2);
}
}
