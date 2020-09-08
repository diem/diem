script {
use 0x1::LibraAccount;

/// # Summary
/// Withdraws a specified amount of LBR from the transaction sender's account, and unstaples the
/// withdrawn LBR into its constituent coins. Deposits each of the constituent coins to the
/// transaction sender's balances. Any account that can hold balances that has the correct balances
/// may send this transaction.
///
/// # Technical Description
/// Withdraws `amount_lbr` LBR coins from the `LibraAccount::Balance<LBR::LBR>` balance held under
/// `account`. Withdraws the backing coins for the LBR coins from the on-chain reserve in the
/// `LBR::Reserve` resource published under `0xA550C18`. It then deposits each of the backing coins
/// into balance resources published under `account`.
///
/// ## Events
/// Successful execution of this transaction will emit two `LibraAccount::SentPaymentEvent`s. One
/// for each constituent currency that is unstapled and returned to the sending `account`'s
/// balances.
///
/// # Parameters
/// | Name         | Type      | Description                                                     |
/// | ------       | ------    | -------------                                                   |
/// | `account`    | `&signer` | The signer reference of the sending account of the transaction. |
/// | `amount_lbr` | `u64`     | The amount of microlibra to unstaple.                           |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                             | Description                                                                               |
/// | ----------------           | --------------                                           | -------------                                                                             |
/// | `Errors::INVALID_STATE`    | `LibraAccount::EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED` | The `LibraAccount::WithdrawCapability` for `account` has previously been extracted.       |
/// | `Errors::NOT_PUBLISHED`    | `LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY`              | `account` doesn't have a balance in LBR.                                                  |
/// | `Errors::LIMIT_EXCEEDED`   | `LibraAccount::EINSUFFICIENT_BALANCE`                    | `amount_lbr` is greater than the balance of LBR in `account`.                             |
/// | `Errors::INVALID_ARGUMENT` | `Libra::ECOIN`                                           | `amount_lbr` is zero.                                                                     |
/// | `Errors::LIMIT_EXCEEDED`   | `LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS`               | `account` has exceeded its daily withdrawal limits for LBR.                               |
/// | `Errors::LIMIT_EXCEEDED`   | `LibraAccount::EDEPOSIT_EXCEEDS_LIMITS`                  | `account` has exceeded its daily deposit limits for one of the backing currencies of LBR. |
/// | `Errors::INVALID_ARGUMENT` | `LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE`         | `account` doesn't hold a balance in one or both of the backing currencies of LBR.         |
///
/// # Related Scripts
/// * `Script::mint_lbr`

fun unmint_lbr(account: &signer, amount_lbr: u64) {
    let withdraw_cap = LibraAccount::extract_withdraw_capability(account);
    LibraAccount::unstaple_lbr(&withdraw_cap, amount_lbr);
    LibraAccount::restore_withdraw_capability(withdraw_cap);
}
}
