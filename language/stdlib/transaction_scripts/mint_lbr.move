script {
use 0x1::LibraAccount;

/// Mint `amount_lbr` LBR from the sending account's constituent coins and deposits the
/// resulting LBR into the sending account.
///
/// # Events
/// When this script executes without aborting, it emits three events:
/// `SentPaymentEvent { amount_coin1, currency_code = Coin1, address_of(account), metadata = x"" }`
/// `SentPaymentEvent { amount_coin2, currency_code = Coin2, address_of(account), metadata = x"" }`
/// on `account`'s `LibraAccount::sent_events` handle where `amount_coin1` and `amount_coin2`
/// are the components amounts of `amount_lbr` LBR, and
/// `ReceivedPaymentEvent { amount_lbr, currency_code = LBR, address_of(account), metadata = x"" }`
/// on `account`'s `LibraAccount::received_events` handle.
///
/// # Abort Conditions
/// > TODO(emmazzz): the documentation below documents the reasons of abort, instead of the categories.
/// > We might want to discuss about what the best approach is here.
/// The following abort conditions have been formally specified and verified. See spec schema
/// `LibraAccount::StapleLBRAbortsIf` for the formal specifications.
///
/// ## Aborts Caused by Invalid Account State
/// * Aborts with `LibraAccount::EINSUFFICIENT_BALANCE` if `amount_coin1` is greater than sending
/// `account`'s balance in `Coin1` or if `amount_coin2` is greater than sending `account`'s balance in `Coin2`.
/// * Aborts with `LibraAccount::ECOIN_DEPOSIT_IS_ZERO` if `amount_lbr` is zero.
/// * Aborts with `LibraAccount::EPAYEE_DOES_NOT_EXIST` if no LibraAccount exists at the address of `account`.
/// * Aborts with `LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE` if LibraAccount exists at `account`,
/// but it does not accept payments in LBR.
///
/// ## Aborts Caused by Invalid LBR Minting State
/// * Aborts with `Libra::EMINTING_NOT_ALLOWED` if minting LBR is not allowed according to the CurrencyInfo<LBR>
/// stored at `CURRENCY_INFO_ADDRESS`.
/// * Aborts with `Libra::ECURRENCY_INFO` if the total value of LBR would reach `MAX_U128` after `amount_lbr`
/// LBR is minted.
///
/// ## Other Aborts
/// These aborts should only happen when `account` has account limit restrictions or
/// has been frozen by Libra administrators.
/// * Aborts with `LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS` if `account` has exceeded their daily
/// withdrawal limits for Coin1 or Coin2.
/// * Aborts with `LibraAccount::EDEPOSIT_EXCEEDS_LIMITS` if `account` has exceeded their daily
/// deposit limits for LBR.
/// * Aborts with `LibraAccount::EACCOUNT_FROZEN` if `account` is frozen.
///
/// # Post Conditions
/// The following post conditions have been formally specified and verified. See spec schema
/// `LibraAccount::StapleLBREnsures` for the formal specifications.
///
/// ## Changed States
/// * The reserve backing for Coin1 and Coin2 increase by the right amounts as specified by the component ratio.
/// * Coin1 and Coin2 balances at the address of sending `account` decrease by the right amounts as specified by
/// the component ratio.
/// * The total value of LBR increases by `amount_lbr`.
/// * LBR balance at the address of sending `account` increases by `amount_lbr`.
///
/// ## Unchanged States
/// * The total values of Coin1 and Coin2 stay the same.
fun mint_lbr(account: &signer, amount_lbr: u64) {
    let withdraw_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::staple_lbr(&withdraw_cap, amount_lbr);
    LibraAccount::restore_withdraw_capability(withdraw_cap)
}

spec fun mint_lbr {
    use 0x1::Signer;
    use 0x1::Option;
    /// > TODO: timeout due to FixedPoint32 flakiness.
    pragma verify = false;
    let account_addr = Signer::spec_address_of(account);
    let cap = Option::borrow(LibraAccount::spec_get_withdraw_cap(account_addr));
    include LibraAccount::ExtractWithdrawCapAbortsIf{sender_addr: account_addr};
    include LibraAccount::StapleLBRAbortsIf{cap: cap};
    include LibraAccount::StapleLBREnsures{cap: cap};
}
}
