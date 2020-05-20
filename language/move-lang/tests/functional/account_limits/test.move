//! account: validatorvivian, 10000000, 0, validator
//! account: bob, 100000000, 0, unhosted
//! account: alice, 100000000, 0, unhosted

module Holder {
    resource struct Hold<T> { x: T }
    public fun hold<T>(x: T) {
        move_to_sender(Hold<T>{x})
    }
}

//! new-transaction
//! sender: association
script {
    use 0x0::Testnet;
    // Unset testnet
    fun main() {
        Testnet::remove_testnet()
    }
}
// check: EXECUTED

//! new-transaction
script {
    use {{default}}::Holder;
    use 0x0::AccountLimits;
    fun main() {
        Holder::hold(
            AccountLimits::grant_account_tracking()
        )
    }
}
// check: EXECUTED


//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{alice}}, 1);
    }
}
// check: ABORTED
// check: 10044

//! new-transaction
//! sender: association
script {
    use 0x0::LibraAccount;
    fun main() {
        LibraAccount::mint_lbr_to_address({{bob}}, 1);
    }
}
// check: ABORTED
// check: 10043

//! new-transaction
//! sender: association
script {
    use 0x0::Testnet;
    // Reset testnet
    fun main() {
        Testnet::initialize()
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x0::AccountLimits;
    fun main() {
        AccountLimits::publish_unrestricted_limits();
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::AccountLimits;
    fun main() {
        AccountLimits::certify_limits_definition({{bob}});
    }
}
// check: EXECUTED

//! new-transaction
script {
    use 0x0::AccountLimits;
    fun main() {
        AccountLimits::decertify_limits_definition({{bob}});
    }
}
// check: ABORTED
// check: 1002

//! new-transaction
//! sender: association
script {
    use 0x0::AccountLimits;
    fun main() {
        AccountLimits::decertify_limits_definition({{bob}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x0::AccountLimits;
    fun main() {
        AccountLimits::unpublish_limits_definition();
    }
}
// check: EXECUTED

//! new-transaction
script {
    use 0x0::AccountLimits;
    use 0x0::Transaction;
    fun main() {
        Transaction::assert(AccountLimits::default_limits_addr() == {{association}}, 0);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    // Since we directly wrote into this account using fake data store, we
    // don't actually know that the balance is greater than 0 in the
    // account limits code, but it is.
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{alice}}, 1);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::LibraAccount;
    fun main() {
        LibraAccount::mint_lbr_to_address({{bob}}, 2);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{alice}}, 1);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::AccountLimits;
    // Publish our own limits definition for testing! Make sure we are
    // exercising the unrestricted limits check.
    fun main() {
        AccountLimits::unpublish_limits_definition();
        AccountLimits::publish_unrestricted_limits();
        AccountLimits::certify_limits_definition({{association}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::LibraAccount;
    fun main() {
        LibraAccount::mint_lbr_to_address({{bob}}, 2);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{alice}}, 1);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::AccountLimits;
    // Publish our own limits definition for testing! Make sure we are
    // exercising the unrestricted limits check.
    fun main() {
        AccountLimits::decertify_limits_definition({{association}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{alice}}, 1);
    }
}
// check: ABORTED
// check: 1

//! new-transaction
//! sender: association
script {
    use 0x0::AccountLimits;
    // Publish our own limits definition for testing!
    fun main() {
        AccountLimits::unpublish_limits_definition();
        AccountLimits::publish_limits_definition(
            100,
            100,
            200,
            40000,
        );
        AccountLimits::certify_limits_definition({{association}});
    }
}
// check: EXECUTED

//! block-prologue
//! proposer: validatorvivian
//! block-time: 40001

//! new-transaction
//! sender: association
script {
    use 0x0::LibraAccount;
    fun main() {
        LibraAccount::mint_lbr_to_address({{bob}}, 100);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::LibraAccount;
    fun main() {
        LibraAccount::mint_lbr_to_address({{bob}}, 1);
    }
}
// check: ABORTED
// check: 9

//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{alice}}, 101);
    }
}
// check: ABORTED
// check: 11

//! new-transaction
//! sender: association
script {
    use 0x0::AccountLimits;
    // Publish our own limits definition for testing! Make sure we are
    // exercising the unrestricted limits check.
    fun main() {
        AccountLimits::decertify_limits_definition({{association}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x0::LibraAccount;
    use 0x0::LBR;
    fun main() {
        LibraAccount::pay_from_sender<LBR::T>({{alice}}, 1);
    }
}
// check: ABORTED
// check: 1

//! new-transaction
//! sender: association
script {
    use 0x0::LibraAccount;
    fun main() {
        LibraAccount::mint_lbr_to_address({{bob}}, 1);
    }
}
// check: ABORTED
// check: 1
