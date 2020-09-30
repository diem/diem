script {
use 0x1::TransactionFee;

/// # Summary
/// Burns the transaction fees collected in the `CoinType` currency so that the
/// Libra association may reclaim the backing coins off-chain. May only be sent
/// by the Treasury Compliance account.
///
/// # Technical Description
/// Burns the transaction fees collected in `CoinType` so that the
/// association may reclaim the backing coins. Once this transaction has executed
/// successfully all transaction fees that will have been collected in
/// `CoinType` since the last time this script was called with that specific
/// currency. Both `balance` and `preburn` fields in the
/// `TransactionFee::TransactionFee<CoinType>` resource published under the `0xB1E55ED`
/// account address will have a value of 0 after the successful execution of this script.
///
/// ## Events
/// The successful execution of this transaction will emit a `Libra::BurnEvent` on the event handle
/// held in the `Libra::CurrencyInfo<CoinType>` resource's `burn_events` published under
/// `0xA550C18`.
///
/// # Parameters
/// | Name         | Type      | Description                                                                                                                                         |
/// | ------       | ------    | -------------                                                                                                                                       |
/// | `CoinType`   | Type      | The Move type for the `CoinType` being added to the sending account of the transaction. `CoinType` must be an already-registered currency on-chain. |
/// | `tc_account` | `&signer` | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.                                           |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                          | Description                                                 |
/// | ----------------           | --------------                        | -------------                                               |
/// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::ETREASURY_COMPLIANCE` | The sending account is not the Treasury Compliance account. |
/// | `Errors::NOT_PUBLISHED`    | `TransactionFee::ETRANSACTION_FEE`    | `CoinType` is not an accepted transaction fee currency.     |
/// | `Errors::INVALID_ARGUMENT` | `Libra::ECOIN`                        | The collected fees in `CoinType` are zero.                  |
///
/// # Related Scripts
/// * `Script::burn`
/// * `Script::cancel_burn`

fun burn_txn_fees<CoinType>(tc_account: &signer) {
    TransactionFee::burn_fees<CoinType>(tc_account);
}
}
