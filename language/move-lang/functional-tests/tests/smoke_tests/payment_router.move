//! account: bob, 100000LBR
//! account: alice, 100000Coin1
//! account: gary, 100000Coin1
//! account: vivian, 100000Coin2
//! account: nope, 100000Coin2

// Errors:
// 0 -> ADDRESS_NOT_ALLOWED
// 1 -> CURRENCY_NOT_HELD
// 2 -> NON_EXCLUSIVE_WITHDRAWALS_NOT_PERMITTED
// 3 -> ADDRESS_IS_NOT_ROUTED
module PaymentRouter {
    use 0x1::LibraAccount;
    use 0x1::Libra::Libra;
    use 0x1::Vector;
    use 0x1::Signer;

    // A resource that specifies the addresses that are allowed to be added
    // to this multi-currency holding.
    resource struct PaymentRouterInfo {
        // Whether the router account has exclusive withdrawal privileges to
        // its routed accounts. If false withdrawal privileges are shared
        // between router and child.
        exclusive_withdrawals_only: bool,
        allowed_addresses: vector<address>,
    }

    // Withdraw capabilities for the added accounts, partitioned by their
    // currency type.
    resource struct AccountInfo<Token> {
        child_accounts: vector<address>,
    }

    // A delegated withdraw capability that is published under the child
    // account and that allows the routing account to withdraw from it (and
    // the sender, depending on flags set in `PaymentRouterInfo`).
    resource struct RoutedAccount<Token> {
        router_account_addr: address,
        withdrawal_cap: LibraAccount::WithdrawCapability,
    }

    // Initialize the sending account with `exclusive_withdrawals_only`
    // either turned on (true) or off (false).
    public fun initialize(account:  &signer, exclusive_withdrawals_only: bool) {
        move_to(account, PaymentRouterInfo {
            exclusive_withdrawals_only,
            allowed_addresses: Vector::empty(),
        })
    }

    // Set whether exclusive access is held to the routed accounts
    public fun set_exclusive_withdrawals(account: &signer, exclusive_withdrawals_only: bool)
    acquires PaymentRouterInfo {
        borrow_global_mut<PaymentRouterInfo>(Signer::address_of(account))
            .exclusive_withdrawals_only = exclusive_withdrawals_only
    }

    // Return whether the router at `router_account_addr` only allows
    // exclusive withdrawals
    public fun exclusive_withdrawals_only(router_account_addr: address): bool
    acquires PaymentRouterInfo {
        borrow_global_mut<PaymentRouterInfo>(router_account_addr).exclusive_withdrawals_only
    }

    // Allow the account at `addr` to delegate its withdrawal_capability to us.
    public fun allow_account_address(account: &signer, addr: address)
    acquires PaymentRouterInfo {
        let router_info = borrow_global_mut<PaymentRouterInfo>(Signer::address_of(account));
        if (!Vector::contains(&router_info.allowed_addresses, &addr))
            Vector::push_back(&mut router_info.allowed_addresses, addr);
    }

    // Allow routing of currencies of `Token` type.
    public fun allow_currency<Token>(account: &signer) {
        move_to(account, AccountInfo<Token>{ child_accounts: Vector::empty() })
    }

    // Add the sending account of currency type `Token` to the router at
    // `router_account_addr`.  The sending address must be in the
    // `allowed_addresses` held under the router account.
    public fun add_account_to<Token>(sender: &signer, router_account_addr: address)
    acquires PaymentRouterInfo, AccountInfo {
        let sender_addr = Signer::address_of(sender);
        let router_info = borrow_global_mut<PaymentRouterInfo>(router_account_addr);
        let (has, index) = Vector::index_of(&router_info.allowed_addresses, &sender_addr);
        assert(has, 0);
        let account_info = borrow_global_mut<AccountInfo<Token>>(router_account_addr);
        Vector::swap_remove(&mut router_info.allowed_addresses, index);
        Vector::push_back(
            &mut account_info.child_accounts,
            sender_addr,
        );
        move_to(sender, RoutedAccount<Token> {
            router_account_addr,
            withdrawal_cap: LibraAccount::extract_withdraw_capability(sender)
        })
    }

    // Routes deposits to a sub-account that holds currencies of type `Token`.
    public fun deposit<Token>(sender: &signer, router_account_addr: address, coin: Libra<Token>)
    acquires AccountInfo {
        let addrs = &borrow_global<AccountInfo<Token>>(router_account_addr).child_accounts;
        assert(!Vector::is_empty(addrs), 1);
        // TODO: policy around how to rotate through different accounts
        let index = 0;
        LibraAccount::deposit(
            sender,
            *Vector::borrow(addrs, index),
            coin
        );
    }

    // Withdraws `amount` of `Token` currency from the sending account
    // using the delegated withdrawal capability in the PaymentRouterInfo
    // at `router_account_addr`.
    public fun withdraw_through<Token>(account: &signer, amount: u64): Libra<Token>
    acquires PaymentRouterInfo, RoutedAccount {
        let routed_info = borrow_global<RoutedAccount<Token>>(Signer::address_of(account));
        let router_info = borrow_global<PaymentRouterInfo>(*&routed_info.router_account_addr);
        assert(!router_info.exclusive_withdrawals_only, 2);
        LibraAccount::withdraw_from(
            &routed_info.withdrawal_cap,
            amount
        )
    }

    // Routes withdrawal requests from the sending account to a sub-account
    // that holds currencies of type `Token`.
    public fun withdraw<Token>(account: &signer, amount: u64): Libra<Token>
    acquires AccountInfo, RoutedAccount {
        let addrs = &borrow_global<AccountInfo<Token>>(Signer::address_of(account)).child_accounts;
        assert(!Vector::is_empty(addrs), 1);
        // TODO: policy around how to rotate through different accounts
        let index = 0;
        let addr = Vector::borrow(addrs, index);
        LibraAccount::withdraw_from(
            &borrow_global<RoutedAccount<Token>>(*addr).withdrawal_cap,
            amount
        )
    }

    // Return the account addresses in this multi-currency holding that can
    // hold currency of `Token` type. The set of addresses returned can both be
    // empty, or non-empty.
    public fun addresses_for_currency<Token>(router_account_addr: address): vector<address>
    acquires AccountInfo {
        *&borrow_global<AccountInfo<Token>>(router_account_addr).child_accounts
    }

    // Return whether the account at `addr` is a routed address or not.
    public fun is_routed<Token>(addr: address): bool {
        exists<RoutedAccount<Token>>(addr)
    }

    // Return the router address for the account.
    public fun router_address<Token>(addr: address): address
    acquires RoutedAccount {
        assert(is_routed<Token>(addr), 3);
        *&borrow_global<RoutedAccount<Token>>(addr).router_account_addr
    }
}

//! new-transaction
//! sender: bob
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::LBR::LBR;
fun main(account: &signer) {
    PaymentRouter::initialize(account, true);
    PaymentRouter::allow_account_address(account, {{bob}});
    PaymentRouter::allow_account_address(account, {{alice}});
    PaymentRouter::allow_account_address(account, {{gary}});
    PaymentRouter::allow_account_address(account, {{vivian}});
    PaymentRouter::allow_currency<Coin1>(account);
    PaymentRouter::allow_currency<Coin2>(account);
    PaymentRouter::allow_currency<LBR>(account);
}
}

//! new-transaction
//! sender: bob
//! gas-price: 0
//! max-gas: 100000
script {
use {{default}}::PaymentRouter;
fun main(account: &signer) {
    PaymentRouter::allow_account_address(account, {{bob}});
    assert(PaymentRouter::exclusive_withdrawals_only({{bob}}), 0);
}
}

//! new-transaction
//! sender: alice
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin1>(sender, {{bob}});
}
}

//! new-transaction
//! sender: gary
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin1>(sender, {{bob}});
}
}

//! new-transaction
//! sender: vivian
//! gas-currency: Coin2
script {
use {{default}}::PaymentRouter;
use 0x1::Coin2::Coin2;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin2>(sender, {{bob}});
}
}

//! new-transaction
//! sender: bob
script {
use {{default}}::PaymentRouter;
use 0x1::LBR::LBR;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<LBR>(sender, {{bob}});
}
}

//! new-transaction
//! sender: nope
//! gas-currency: Coin2
script {
use {{default}}::PaymentRouter;
use 0x1::Coin2::Coin2;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin2>(sender, {{bob}});
}
}
// check: ABORTED
// check: 0

//! new-transaction
//! sender: bob
script {
use 0x1::Vector;
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::LBR::LBR;
use 0x1::Signer;
fun main(account: &signer) {
    let sender = Signer::address_of(account);
    let addrs_coin1 = PaymentRouter::addresses_for_currency<Coin1>(sender);
    let addrs_coin2 = PaymentRouter::addresses_for_currency<Coin2>(sender);
    let addrs_lbr = PaymentRouter::addresses_for_currency<LBR>(sender);

    assert(Vector::length(&addrs_coin1) == 2, 0);
    assert(Vector::length(&addrs_coin2) == 1, 1);
    assert(Vector::length(&addrs_lbr) == 1, 2);
}
}

//! new-transaction
//! sender: bob
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
// Withdraw from the router "root" account.
fun main(account: &signer) {
    let prev_balance = LibraAccount::balance<Coin1>({{alice}});
    let x_coins = PaymentRouter::withdraw<Coin1>(account, 10);
    let new_balance = LibraAccount::balance<Coin1>({{alice}});
    assert(prev_balance - new_balance == 10, 0);
    PaymentRouter::deposit<Coin1>(account, {{bob}}, x_coins);
    new_balance = LibraAccount::balance<Coin1>({{alice}});
    assert(prev_balance - new_balance == 0, 1);
}
}

//! new-transaction
//! sender: alice
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
// Try to have alice withdraw through the payment router. But this doesn't
// work since `exclusive_withdrawals_only` is set to true.
fun main(account: &signer) {
    let x_coins = PaymentRouter::withdraw_through<Coin1>(account, 10);
    PaymentRouter::deposit<Coin1>(account, {{bob}}, x_coins);
}
}
// check: ABORTED
// check: 2

//! new-transaction
//! sender: bob
script {
use {{default}}::PaymentRouter;
use 0x1::LBR::LBR;
// Try to have bob withdraw through the payment router owned by bob. But this doesn't
// work since `exclusive_withdrawals_only` is set to true.
fun main(account: &signer) {
    let x_coins = PaymentRouter::withdraw_through<LBR>(account, 10);
    PaymentRouter::deposit<LBR>(account, {{bob}}, x_coins);
}
}
// check: ABORTED
// check: 2

//! new-transaction
//! sender: bob
script {
use {{default}}::PaymentRouter;
fun main(account: &signer) {
    PaymentRouter::set_exclusive_withdrawals(account, false);
}
}

//! new-transaction
//! sender: alice
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
use 0x1::LibraAccount;
// Try to have alice withdraw through the payment router. This now succeeds
// since we set `exclusive_withdrawals_only` to false.
fun main(account: &signer) {
    let prev_balance = LibraAccount::balance<Coin1>({{alice}});
    let x_coins = PaymentRouter::withdraw_through<Coin1>(account, 10);
    let new_balance = LibraAccount::balance<Coin1>({{alice}});
    assert(prev_balance - new_balance == 10, 0);
    PaymentRouter::deposit<Coin1>(account, {{bob}}, x_coins);
    new_balance = LibraAccount::balance<Coin1>({{alice}});
    assert(prev_balance - new_balance == 0, 1);
}
}

// ==== Multiple routers test ===

//! account: bob1, 100000Coin1
//! account: alice1, 100000Coin1
//! account: gary1, 100000Coin1
//! account: vivian1, 100000Coin1
//! account: nope1, 100000Coin1

//! new-transaction
//! sender: bob1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    PaymentRouter::initialize(account, true);
    PaymentRouter::allow_account_address(account, {{bob1}});
    PaymentRouter::allow_account_address(account, {{gary1}});
    PaymentRouter::allow_account_address(account, {{nope1}});
    PaymentRouter::allow_currency<Coin1>(account);
}
}

//! new-transaction
//! sender: alice1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main(account: &signer) {
    PaymentRouter::initialize(account, false);
    PaymentRouter::allow_account_address(account, {{alice1}});
    PaymentRouter::allow_account_address(account, {{vivian1}});
    // nope1 could be added to both.
    PaymentRouter::allow_account_address(account, {{nope1}});
    PaymentRouter::allow_currency<Coin1>(account);
}
}

//! new-transaction
//! sender: gary1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin1>(sender, {{bob1}});
}
}

//! new-transaction
//! sender: bob1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin1>(sender, {{bob1}});
}
}

//! new-transaction
//! sender: alice1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin1>(sender, {{alice1}});
}
}

//! new-transaction
//! sender: vivian1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin1>(sender, {{alice1}});
}
}

//! new-transaction
//! sender: gary1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
// Try to have gary1 withdraw through bob1's payment router. But this doesn't
// work since bob1 has set the exclusive_withdrawal_flag to true.
fun main(account: &signer) {
    let x_coins = PaymentRouter::withdraw_through<Coin1>(account, 10);
    PaymentRouter::deposit<Coin1>(account, {{bob1}}, x_coins);
}
}
// check: ABORTED
// check: 2

//! new-transaction
//! sender: vivian1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
// vivan can withdraw through alice1's payment router since alice1 has set
// the exclusive_withdrawal_flag to false.
fun main(account: &signer) {
    let x_coins = PaymentRouter::withdraw_through<Coin1>(account, 10);
    PaymentRouter::deposit<Coin1>(account, {{alice1}}, x_coins);
}
}

//! new-transaction
//! sender: vivian1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main() {
    assert(PaymentRouter::is_routed<Coin1>({{bob1}}), 0);
    assert(PaymentRouter::is_routed<Coin1>({{gary1}}), 0);
    assert(PaymentRouter::is_routed<Coin1>({{alice1}}), 0);
    assert(PaymentRouter::is_routed<Coin1>({{vivian1}}), 0);
    assert(!PaymentRouter::is_routed<Coin1>({{nope1}}), 1);

    assert(PaymentRouter::router_address<Coin1>({{bob1}}) == {{bob1}}, 2);
    assert(PaymentRouter::router_address<Coin1>({{gary1}}) == {{bob1}}, 2);
    assert(PaymentRouter::router_address<Coin1>({{alice1}}) == {{alice1}}, 2);
    assert(PaymentRouter::router_address<Coin1>({{vivian1}}) == {{alice1}}, 2);
}
}

//! new-transaction
//! sender: nope1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin1>(sender, {{alice1}});
}
}

//! new-transaction
//! sender: nope1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
// an account can't belong to multiple routers. This fails since nope1 is
// already routed by the payment router at alice1.
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin1>(sender, {{bob1}});
}
}
// check: ABORTED
// check: 11

//! new-transaction
//! sender: nope1
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
use 0x1::Vector;
fun main() {
    let bob1s = PaymentRouter::addresses_for_currency<Coin1>({{bob1}});
    let alice1s = PaymentRouter::addresses_for_currency<Coin1>({{alice1}});

    // alice1s addresses for Coin1 should have nope1, but bob1's should not
    assert(!Vector::contains(&bob1s, &{{nope1}}), 3);
    assert(Vector::contains(&alice1s, &{{nope1}}), 4);
}
}

// === Withdraw no currency tests ===

//! account: bob2, 100000LBR
//! account: alice2, 100000Coin1
//! account: vivian2, 100000Coin2

//! new-transaction
//! sender: bob2
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
use 0x1::Coin2::Coin2;
use 0x1::LBR::LBR;
fun main(account: &signer) {
    PaymentRouter::initialize(account, true);
    PaymentRouter::allow_account_address(account, {{bob2}});
    PaymentRouter::allow_account_address(account, {{alice2}});
    PaymentRouter::allow_currency<Coin1>(account);
    PaymentRouter::allow_currency<Coin2>(account);
    PaymentRouter::allow_currency<LBR>(account);
}
}

//! new-transaction
//! sender: alice2
//! gas-currency: Coin1
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<Coin1>(sender, {{bob2}});
}
}

//! new-transaction
//! sender: bob2
script {
use {{default}}::PaymentRouter;
use 0x1::LBR::LBR;
fun main(sender: &signer) {
    PaymentRouter::add_account_to<LBR>(sender, {{bob2}});
}
}

//! new-transaction
//! sender: bob2
script {
use {{default}}::PaymentRouter;
use 0x1::Coin2::Coin2;
fun main(account: &signer) {
    let x_coins = PaymentRouter::withdraw<Coin2>(account, 10);
    PaymentRouter::deposit<Coin2>(account, {{bob2}}, x_coins);
}
}
// check: ABORTED
// check: 1

//! new-transaction
//! sender: vivian2
//! gas-currency: Coin2
script {
use {{default}}::PaymentRouter;
use 0x1::Coin2::Coin2;
use 0x1::LibraAccount;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    let x_coins = LibraAccount::withdraw_from<Coin2>(&with_cap, 10);
    LibraAccount::restore_withdraw_capability(with_cap);
    PaymentRouter::deposit<Coin2>(account, {{bob2}}, x_coins);
}
}
// check: ABORTED
// check: 1

//! new-transaction
script {
use {{default}}::PaymentRouter;
use 0x1::Coin1::Coin1;
fun main() {
    let _addr = PaymentRouter::router_address<Coin1>({{default}})
}
}
// check: ABORTED
// check: 3
