//! account: bob, 100000000XUS, 0
//! account: alice, 100000000XUS, 0

//! new-transaction
module Holder {
    resource struct Hold<T> { x: T }
    public fun hold<T>(account: &signer, x: T) {
        move_to(account, Hold<T>{ x })
    }
}

//! new-transaction
script {
use 0x1::AccountLimits;
use {{default}}::Holder;
fun main(account: &signer) {
    Holder::hold(
        account,
        AccountLimits::grant_mutation_capability(account)
    );
}
}

//! new-transaction
//! sender: diemroot
//! execute-as: bob
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(dr: &signer, bob_account: &signer) {
    AccountLimits::publish_unrestricted_limits<XUS>(bob_account);
    AccountLimits::publish_window<XUS>(dr, bob_account, {{bob}});
}
}

//! new-transaction
//! sender: diemroot
//! execute-as: bob
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(dr: &signer, bob_account: &signer) {
    AccountLimits::publish_window<XUS>(dr, bob_account, {{bob}});
}
}

//! new-transaction
//! sender: bob
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(bob_account: &signer) {
    AccountLimits::publish_window<XUS>(bob_account, bob_account, {{bob}});
}
}

//! new-transaction
//! sender: bob
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(bob_account: &signer) {
    AccountLimits::publish_unrestricted_limits<XUS>(bob_account);
}
}

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(tc: &signer) {
    AccountLimits::update_limits_definition<XUS>(
        tc,
        {{bob}},
        100, /* new_max_inflow */
        200, /* new_max_outflow */
        150, /* new_max_holding_balance */
        10000, /* new_time_period */
    )
}
}

//! new-transaction
//! sender: diemroot
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(tc: &signer) {
    AccountLimits::update_limits_definition<XUS>(
        tc,
        {{bob}},
        100, /* new_max_inflow */
        200, /* new_max_outflow */
        150, /* new_max_holding_balance */
        10000, /* new_time_period */
    )
}
}

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(tc: &signer) {
    AccountLimits::update_limits_definition<XUS>(
        tc,
        {{bob}},
        0, /* new_max_inflow */
        0, /* new_max_outflow */
        150, /* new_max_holding_balance */
        10000, /* new_time_period */
    )
}
}

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(tc: &signer) {
    AccountLimits::update_limits_definition<XUS>(
        tc,
        {{default}},
        0, /* new_max_inflow */
        0, /* new_max_outflow */
        150, /* new_max_holding_balance */
        10000, /* new_time_period */
    )
}
}

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(tc: &signer) {
    AccountLimits::update_limits_definition<XUS>(
        tc,
        {{bob}},
        0, /* new_max_inflow */
        0, /* new_max_outflow */
        150, /* new_max_holding_balance */
        10000, /* new_time_period */
    )
}
}

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(tc: &signer) {
    AccountLimits::update_window_info<XUS>(
        tc,
        {{bob}},
        120,
        {{bob}},
    )
}
}

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(tc: &signer) {
    AccountLimits::update_window_info<XUS>(
        tc,
        {{bob}},
        0,
        {{bob}},
    )
}
}

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(tc: &signer) {
    AccountLimits::update_window_info<XUS>(
        tc,
        {{bob}},
        120,
        {{alice}},
    )
}
}

//! new-transaction
//! sender: diemroot
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(dr: &signer) {
    AccountLimits::update_window_info<XUS>(
        dr,
        {{bob}},
        120,
        {{bob}},
    )
}
}

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main() {
    assert(AccountLimits::limits_definition_address<XUS>({{bob}}) == {{bob}}, 0);
}
}

//! new-transaction
//! sender: blessed
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main() {
    assert(AccountLimits::has_limits_published<XUS>({{bob}}), 1);

    assert(!AccountLimits::has_limits_published<XUS>({{alice}}), 3);
}
}

//! new-transaction
//! sender: diemroot
//! execute-as: bob
script {
use 0x1::AccountLimits;
use 0x1::XUS::XUS;
fun main(dr: &signer, bob_account: &signer) {
    AccountLimits::publish_window<XUS>(dr, bob_account, {{default}});
}
}
