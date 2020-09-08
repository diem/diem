script {
use 0x1::LibraAccount;

/// # Summary
/// Mints LBR from the sending account's constituent coins by depositing in the
/// on-chain LBR reserve. Deposits the newly-minted LBR into the sending
/// account. Can be sent by any account that can hold balances for the constituent
/// currencies for LBR and LBR.
///
/// # Technical Description
/// Mints `amount_lbr` LBR from the sending account's constituent coins and deposits the
/// resulting LBR into the sending account.
///
/// ## Events
/// Successful execution of this script emits three events:
/// * A `LibraAccount::SentPaymentEvent` with the Coin1 currency code, and a
/// `LibraAccount::SentPaymentEvent` with the Coin2 currency code on `account`'s
/// `LibraAccount::LibraAccount` `sent_events` handle with the `amounts` for each event being the
/// components amounts of `amount_lbr` LBR; and
/// * A `LibraAccount::ReceivedPaymentEvent` on `account`'s `LibraAccount::LibraAccount`
/// `received_events` handle with the LBR currency code and amount field equal to `amount_lbr`.
///
/// # Parameters
/// | Name         | Type      | Description                                      |
/// | ------       | ------    | -------------                                    |
/// | `account`    | `&signer` | The signer reference of the sending account.     |
/// | `amount_lbr` | `u64`     | The amount of LBR (in microlibra) to be created. |
///

/// # Common Abort Conditions
/// | Error Category             | Error Reason                                     | Description                                                                      |
/// | ----------------           | --------------                                   | -------------                                                                    |
/// | `Errors::NOT_PUBLISHED`    | `LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY`      | `account` doesn't hold a balance in one of the backing currencies of LBR.        |
/// | `Errors::INVALID_ARGUMENT` | `LBR::EZERO_LBR_MINT_NOT_ALLOWED`                | `amount_lbr` passed in was zero.                                                 |
/// | `Errors::LIMIT_EXCEEDED`   | `LBR::ECOIN1`                                    | The amount of `Coin1` needed for the specified LBR would exceed `LBR::MAX_U64`.  |
/// | `Errors::LIMIT_EXCEEDED`   | `LBR::ECOIN2`                                    | The amount of `Coin2` needed for the specified LBR would exceed `LBR::MAX_U64`.  |
/// | `Errors::INVALID_STATE`    | `Libra::EMINTING_NOT_ALLOWED`                    | Minting of LBR is not allowed currently.                                         |
/// | `Errors::INVALID_ARGUMENT` | `LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE` | `account` doesn't hold a balance in LBR.                                         |
/// | `Errors::LIMIT_EXCEEDED`   | `LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS`       | `account` has exceeded its daily withdrawal limits for the backing coins of LBR. |
/// | `Errors::LIMIT_EXCEEDED`   | `LibraAccount::EDEPOSIT_EXCEEDS_LIMITS`          | `account` has exceeded its daily deposit limits for LBR.                         |
///
/// # Related Scripts
/// * `Script::unmint_lbr`

fun mint_lbr(account: &signer, amount_lbr: u64) {
    let withdraw_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::staple_lbr(&withdraw_cap, amount_lbr);
    LibraAccount::restore_withdraw_capability(withdraw_cap)
}

// # Specified List of Abort Conditions
// > TODO(emmazzz): the documentation below documents the reasons of abort, instead of the categories.
// > We might want to discuss about what the best approach is here.
// The following abort conditions have been formally specified and verified. See spec schema
// `LibraAccount::StapleLBRAbortsIf` for the formal specifications.
//
// ## Aborts Caused by Invalid Account State
// * Aborts with `LibraAccount::EINSUFFICIENT_BALANCE` if `amount_coin1` is greater than sending
// `account`'s balance in `Coin1` or if `amount_coin2` is greater than sending `account`'s balance in `Coin2`.
// * Aborts with `LibraAccount::ECOIN_DEPOSIT_IS_ZERO` if `amount_lbr` is zero.
// * Aborts with `LibraAccount::EPAYEE_DOES_NOT_EXIST` if no LibraAccount exists at the address of `account`.
// * Aborts with `LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE` if LibraAccount exists at `account`,
// but it does not accept payments in LBR.
//
// ## Aborts Caused by Invalid LBR Minting State
// * Aborts with `Libra::EMINTING_NOT_ALLOWED` if minting LBR is not allowed according to the CurrencyInfo<LBR>
// stored at `CURRENCY_INFO_ADDRESS`.
// * Aborts with `Libra::ECURRENCY_INFO` if the total value of LBR would reach `MAX_U128` after `amount_lbr`
// LBR is minted.
//
// ## Other Aborts
// These aborts should only happen when `account` has account limit restrictions or
// has been frozen by Libra administrators.
// * Aborts with `LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS` if `account` has exceeded their daily
// withdrawal limits for Coin1 or Coin2.
// * Aborts with `LibraAccount::EDEPOSIT_EXCEEDS_LIMITS` if `account` has exceeded their daily
// deposit limits for LBR.
// * Aborts with `LibraAccount::EACCOUNT_FROZEN` if `account` is frozen.
//
// # Post Conditions
// The following post conditions have been formally specified and verified. See spec schema
// `LibraAccount::StapleLBREnsures` for the formal specifications.
//
// ## Changed States
// * The reserve backing for Coin1 and Coin2 increase by the right amounts as specified by the component ratio.
// * Coin1 and Coin2 balances at the address of sending `account` decrease by the right amounts as specified by
// the component ratio.
// * The total value of LBR increases by `amount_lbr`.
// * LBR balance at the address of sending `account` increases by `amount_lbr`.
//
// ## Unchanged States
// * The total values of Coin1 and Coin2 stay the same.
//
spec fun mint_lbr {
    use 0x1::Signer;
    /// > TODO: timeout due to FixedPoint32 flakiness.
    pragma verify = false;
    let account_addr = Signer::spec_address_of(account);
    let cap = LibraAccount::spec_get_withdraw_cap(account_addr);
    include LibraAccount::ExtractWithdrawCapAbortsIf{sender_addr: account_addr};
    include LibraAccount::StapleLBRAbortsIf{cap: cap};
    include LibraAccount::StapleLBREnsures{cap: cap};
}
}
