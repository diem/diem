//! account: bob, 10000000LBR
//! account: alice, 10000000Coin1
//! account: cody, 10000000Coin2


module MultiCurrencyAccount {
    use 0x0::Transaction;
    use 0x0::LibraAccount;
    use 0x0::Vector;
    use 0x0::Libra;

    resource struct T {
        withdrawal_caps: vector<LibraAccount::WithdrawalCapability>,
    }

    public fun init() {
        move_to_sender(T{ withdrawal_caps: Vector::empty() });
    }

    public fun add_cap(owner_addr: address)
    acquires T {
        let withdrawal_cap = LibraAccount::extract_sender_withdrawal_capability();
        let x = borrow_global_mut<T>(owner_addr);
        Vector::push_back(&mut x.withdrawal_caps, withdrawal_cap)
    }

    public fun withdraw<Token>(amount: u64, cap_index: u64): Libra::T<Token>
    acquires T {
        let x = borrow_global<T>(Transaction::sender());
        LibraAccount::withdraw_with_capability<Token>(
            Vector::borrow(&x.withdrawal_caps, cap_index),
            amount
        )
    }
}

//! new-transaction
//! sender: bob
script {
use {{default}}::MultiCurrencyAccount;
// Setup bob's account as a multi-currency account.
fun main() {
    MultiCurrencyAccount::init();
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//! gas-price: 0
script {
use {{default}}::MultiCurrencyAccount;
// Now add coin1 (alice's) account to the multi currency account in bob's
fun main() {
    MultiCurrencyAccount::add_cap({{bob}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: cody
//! gas-price: 0
script {
use {{default}}::MultiCurrencyAccount;
// Now add coin1 (alice's) account to the multi currency account in bob's
fun main() {
    MultiCurrencyAccount::add_cap({{bob}});
}
}
// check: EXECUTED

// Now mint LBR to bob's account
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
use {{default}}::MultiCurrencyAccount;
fun main() {
    let sender = Transaction::sender();
    let amount_lbr = 10;
    let coin1_balance = LibraAccount::balance<Coin1::T>({{alice}});
    let coin2_balance = LibraAccount::balance<Coin2::T>({{cody}});
    let coin1 = MultiCurrencyAccount::withdraw<Coin1::T>(coin1_balance, 0);
    let coin2 = MultiCurrencyAccount::withdraw<Coin2::T>(coin2_balance, 1);
    let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 10, 0);
    Transaction::assert(Libra::value(&coin1) == coin1_balance - 6, 1);
    Transaction::assert(Libra::value(&coin2) == coin2_balance - 6, 2);
    LibraAccount::deposit(sender, lbr);
    LibraAccount::deposit({{alice}}, coin1);
    LibraAccount::deposit({{cody}}, coin2);
}
}
// check: EXECUTED

// Now unpack from the LBR into the constituent coins
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(10);
    Transaction::assert(Libra::value(&lbr) == 10, 3);
    let (coin1, coin2) = LBR::unpack(lbr);
    Transaction::assert(Libra::value(&coin1) == 5, 4);
    Transaction::assert(Libra::value(&coin2) == 5, 5);
    LibraAccount::deposit({{alice}}, coin1);
    LibraAccount::deposit({{cody}}, coin2);
}
}
// not: PreburnEvent
// not: BurnEvent
// check: EXECUTED

// Now mint zero LBR to bob's account
//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use 0x0::Coin1;
use 0x0::Coin2;
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Libra;
use {{default}}::MultiCurrencyAccount;
fun main() {
    let amount_lbr = 0;
    let coin1_balance = LibraAccount::balance<Coin1::T>({{alice}});
    let coin2_balance = LibraAccount::balance<Coin2::T>({{cody}});
    let coin1 = MultiCurrencyAccount::withdraw<Coin1::T>(coin1_balance, 0);
    let coin2 = MultiCurrencyAccount::withdraw<Coin2::T>(coin2_balance, 1);
    let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
    Transaction::assert(Libra::value(&lbr) == 0, 6);
    Transaction::assert(Libra::value(&coin1) == coin1_balance, 7);
    Transaction::assert(Libra::value(&coin2) == coin2_balance, 8);
    Libra::destroy_zero(lbr);
    LibraAccount::deposit({{alice}}, coin1);
    LibraAccount::deposit({{cody}}, coin2);
}
}
// not: MintEvent
// check: EXECUTED
