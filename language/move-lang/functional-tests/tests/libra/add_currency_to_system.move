//! account: vivian, 1000000, 0, validator
//! account: dd, 0, 0, address
//! account: bob, 0Coin1, 0, vasp

//! new-transaction
//! sender: bob
//! gas-currency: Coin3
script {
fun main() {}
}
// check: LINKER_ERROR

//! new-transaction
//! sender: libraroot
//! args: b"01",
script {
use 0x1::LibraVMConfig;

fun main(config: &signer, args: vector<u8>) {
    LibraVMConfig::set_publishing_option(config, args)
}
}
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: libraroot
address 0x1 {
module Coin3 {
    use 0x1::FixedPoint32;
    use 0x1::Libra::{Self, BurnCapability, MintCapability};
    use 0x1::Roles;
    use 0x1::Offer;

    struct Coin3 { }

    public fun initialize(lr_account: &signer) {
        // Register the Coin3 currency.
        let (mint_cap, burn_cap) = Libra::register_currency<Coin3>(
            lr_account,
            FixedPoint32::create_from_rational(1, 2), // exchange rate to LBR
            false,   // is_synthetic
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"Coin3",
        );
        Offer::create<BurnCapability<Coin3>>(lr_account, burn_cap, {{blessed}});
        Offer::create<MintCapability<Coin3>>(lr_account, mint_cap, {{blessed}});
    }

    public fun finalize(tc_account: &signer) {
        assert(Roles::has_treasury_compliance_role(tc_account), 0);
        let mint_cap = Offer::redeem<MintCapability<Coin3>>(tc_account, {{libraroot}});
        let burn_cap = Offer::redeem<BurnCapability<Coin3>>(tc_account, {{libraroot}});
        Libra::publish_mint_capability<Coin3>(tc_account, mint_cap, tc_account);
        Libra::publish_burn_capability<Coin3>(tc_account, burn_cap, tc_account);
    }
}
}
// check: EXECUTED

//! new-transaction
//! sender: libraroot
script {
use 0x1::Coin3;
fun main(account: &signer) {
    Coin3::initialize(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::Coin3;
fun main(account: &signer) {
    Coin3::finalize(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
//! gas-currency: Coin3
script {
use 0x1::Libra;
use 0x1::Coin3::Coin3;
use 0x1::FixedPoint32;
fun main(account: &signer) {
    assert(Libra::approx_lbr_for_value<Coin3>(10) == 5, 1);
    assert(Libra::scaling_factor<Coin3>() == 1000000, 2);
    assert(Libra::fractional_part<Coin3>() == 100, 3);
    Libra::update_lbr_exchange_rate<Coin3>(account, FixedPoint32::create_from_rational(1, 3));
    assert(Libra::approx_lbr_for_value<Coin3>(10) == 3, 4);
}
}
// check: ToLBRExchangeRateUpdateEvent
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
use 0x1::Coin3::Coin3;
use 0x1::Libra;
use 0x1::DesignatedDealer;
fun main(account: &signer) {
    let pubkey = x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d";
    let prev_mcap3 = Libra::market_cap<Coin3>();
    LibraAccount::create_designated_dealer<Coin3>(
        account,
        {{dd}},
        {{dd::auth_key}},
        x"",
        x"",
        pubkey,
        false,
    );
    DesignatedDealer::add_tier(account, {{dd}}, 10000000);
    LibraAccount::tiered_mint<Coin3>(
        account,
        {{dd}},
        100000,
        0,
    );
    assert(Libra::market_cap<Coin3>() - prev_mcap3 == 100000, 8);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use 0x1::LibraAccount;
use 0x1::Coin3::Coin3;
fun main(account: &signer) {
    LibraAccount::add_currency<Coin3>(account);
}
}
// check: EXECUTED

//! new-transaction
//! sender: dd
script {
use 0x1::LibraAccount;
use 0x1::Coin3::Coin3;
fun main(account: &signer) {
    let with_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::pay_from<Coin3>(
        &with_cap,
        {{bob}},
        10000,
        x"",
        x""
    );
    LibraAccount::restore_withdraw_capability(with_cap);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-currency: Coin3
script {
fun main() {}
}
// check: EXECUTED
