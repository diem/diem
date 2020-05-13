//! account: alice, 0, 0, empty
//! account: bob, 0, 0, unhosted

module X {
    struct T {}
    public fun x(): T {
        T{}
    }
}

//! new-transaction
//! sender: association
script {
    use {{default}}::X;
    use 0x0::AccountType;
    fun main() {
        AccountType::register<X::T>();
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use {{default}}::X;
    use 0x0::AccountType;
    fun main() {
        AccountType::apply_for_granting_capability<X::T>();
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use {{default}}::X;
    use 0x0::AccountType;

    fun main() {
        AccountType::certify_granting_capability<X::T>({{bob}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use {{default}}::X;
    use 0x0::AccountType;
    fun main() {
        AccountType::apply_for_transition_capability<X::T>({{bob}});
        AccountType::grant_transition_capability<X::T>({{bob}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use {{default}}::X;
    use 0x0::AccountType;
    fun main() {
        AccountType::apply_for(X::x(), {{bob}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use {{default}}::X;
use 0x0::AccountType;
use 0x0::Transaction;
fun main() {
    AccountType::transition<X::T>({{alice}});
    Transaction::assert(AccountType::is_a<X::T>({{alice}}), 1);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{bob}}, 1);
    }
}
// check: ABORTED
// check: 3001
