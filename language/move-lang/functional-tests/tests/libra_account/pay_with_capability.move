//! account: alice
//! account: bob
//! account: carol

//! sender: alice
module AlicePays {
    use 0x0::LBR::LBR;
    use 0x0::LibraAccount;

    resource struct T {
        cap: LibraAccount::WithdrawCapability,
    }

    public fun create(sender: &signer) {
        move_to(sender, T {
            cap: LibraAccount::extract_withdraw_capability(sender),
        })
    }

    public fun pay(payee: address, amount: u64) acquires T {
        let t = borrow_global<T>({{alice}});
        LibraAccount::pay_from_with_metadata<LBR>(
            &t.cap,
            payee,
            amount,
            x"0A11CE",
            x""
        )
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{alice}}::AlicePays;
fun main(sender: &signer) {
    AlicePays::create(sender)
}
}

// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use {{alice}}::AlicePays;
use 0x0::LBR::LBR;
use 0x0::LibraAccount;

fun main() {
    let carol_prev_balance = LibraAccount::balance<LBR>({{carol}});
    let alice_prev_balance = LibraAccount::balance<LBR>({{alice}});
    AlicePays::pay({{carol}}, 10);
    assert(carol_prev_balance + 10 == LibraAccount::balance<LBR>({{carol}}), 0);
    assert(alice_prev_balance - 10 == LibraAccount::balance<LBR>({{alice}}), 1);
}
}

// check: EXECUTED
