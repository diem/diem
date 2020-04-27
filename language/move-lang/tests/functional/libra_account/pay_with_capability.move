//! account: alice
//! account: bob
//! account: carol

//! sender: alice
module AlicePays {
    use 0x0::LBR;
    use 0x0::LibraAccount;

    resource struct T {
        cap: LibraAccount::WithdrawalCapability,
    }

    public fun create() {
        move_to_sender(T {
            cap: LibraAccount::extract_sender_withdrawal_capability(),
        })
    }

    public fun pay(payee: address, amount: u64) acquires T {
        let t = borrow_global<T>({{alice}});
        LibraAccount::pay_from_capability<LBR::T>(
            payee,
            &t.cap,
            amount,
            x"0A11CE",
            x""
        )
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
use {{alice}}::AlicePays;
fun main() {
    AlicePays::create()
}

// check: EXECUTED

//! new-transaction
//! sender: bob
use {{alice}}::AlicePays;
use 0x0::LBR;
use 0x0::LibraAccount;
use 0x0::Transaction;

fun main() {
    let carol_prev_balance = LibraAccount::balance<LBR::T>({{carol}});
    let alice_prev_balance = LibraAccount::balance<LBR::T>({{alice}});
    AlicePays::pay({{carol}}, 10);
    Transaction::assert(carol_prev_balance + 10 == LibraAccount::balance<LBR::T>({{carol}}), 0);
    Transaction::assert(alice_prev_balance - 10 == LibraAccount::balance<LBR::T>({{alice}}), 1);
}

// check: EXECUTED
