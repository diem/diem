script {
use 0x1::LibraAccount;
// Prover deps:
use 0x1::LibraAccount::{Balance, LibraAccount};
use 0x1::AccountFreezing;
use 0x1::AccountLimits;
use 0x1::DualAttestation;
use 0x1::Signer;
use 0x1::VASP;

/// # Summary
/// Transfers a given number of coins in a specified currency from one account to another.
/// Transfers over a specified amount defined on-chain that are between two different VASPs, or
/// other accounts that have opted-in will be subject to on-chain checks to ensure the receiver has
/// agreed to receive the coins.  This transaction can be sent by any account that can hold a
/// balance, and to any account that can hold a balance. Both accounts must hold balances in the
/// currency being transacted.
///
/// # Technical Description
///
/// Transfers `amount` coins of type `Currency` from `payer` to `payee` with (optional) associated
/// `metadata` and an (optional) `metadata_signature` on the message
/// `metadata` | `Signer::address_of(payer)` | `amount` | `DualAttestation::DOMAIN_SEPARATOR`.
/// The `metadata` and `metadata_signature` parameters are only required if `amount` >=
/// `DualAttestation::get_cur_microlibra_limit` LBR and `payer` and `payee` are distinct VASPs.
/// However, a transaction sender can opt in to dual attestation even when it is not required
/// (e.g., a DesignatedDealer -> VASP payment) by providing a non-empty `metadata_signature`.
/// Standardized `metadata` LCS format can be found in `libra_types::transaction::metadata::Metadata`.
///
/// ## Events
/// Successful execution of this script emits two events:
/// * A `LibraAccount::SentPaymentEvent` on `payer`'s `LibraAccount::LibraAccount` `sent_events` handle; and
/// * A `LibraAccount::ReceivedPaymentEvent` on `payee`'s `LibraAccount::LibraAccount` `received_events` handle.
///
/// # Parameters
/// | Name                 | Type         | Description                                                                                                                  |
/// | ------               | ------       | -------------                                                                                                                |
/// | `Currency`           | Type         | The Move type for the `Currency` being sent in this transaction. `Currency` must be an already-registered currency on-chain. |
/// | `payer`              | `&signer`    | The signer reference of the sending account that coins are being transferred from.                                           |
/// | `payee`              | `address`    | The address of the account the coins are being transferred to.                                                               |
/// | `metadata`           | `vector<u8>` | Optional metadata about this payment.                                                                                        |
/// | `metadata_signature` | `vector<u8>` | Optional signature over `metadata` and payment information. See                                                              |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                     | Description                                                                                                                         |
/// | ----------------           | --------------                                   | -------------                                                                                                                       |
/// | `Errors::NOT_PUBLISHED`    | `LibraAccount::EPAYER_DOESNT_HOLD_CURRENCY`      | `payer` doesn't hold a balance in `Currency`.                                                                                       |
/// | `Errors::LIMIT_EXCEEDED`   | `LibraAccount::EINSUFFICIENT_BALANCE`            | `amount` is greater than `payer`'s balance in `Currency`.                                                                           |
/// | `Errors::INVALID_ARGUMENT` | `LibraAccount::ECOIN_DEPOSIT_IS_ZERO`            | `amount` is zero.                                                                                                                   |
/// | `Errors::NOT_PUBLISHED`    | `LibraAccount::EPAYEE_DOES_NOT_EXIST`            | No account exists at the `payee` address.                                                                                           |
/// | `Errors::INVALID_ARGUMENT` | `LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE` | An account exists at `payee`, but it does not accept payments in `Currency`.                                                        |
/// | `Errors::INVALID_STATE`    | `AccountFreezing::EACCOUNT_FROZEN`               | The `payee` account is frozen.                                                                                                      |
/// | `Errors::INVALID_ARGUMENT` | `DualAttestation::EMALFORMED_METADATA_SIGNATURE` | `metadata_signature` is not 64 bytes.                                                                                               |
/// | `Errors::INVALID_ARGUMENT` | `DualAttestation::EINVALID_METADATA_SIGNATURE`   | `metadata_signature` does not verify on the against the `payee'`s `DualAttestation::Credential` `compliance_public_key` public key. |
/// | `Errors::LIMIT_EXCEEDED`   | `LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS`       | `payer` has exceeded its daily withdrawal limits for the backing coins of LBR.                                                      |
/// | `Errors::LIMIT_EXCEEDED`   | `LibraAccount::EDEPOSIT_EXCEEDS_LIMITS`          | `payee` has exceeded its daily deposit limits for LBR.                                                                              |
///
/// # Related Scripts
/// * `Script::create_child_vasp_account`
/// * `Script::create_parent_vasp_account`
/// * `Script::add_currency_to_account`

fun peer_to_peer_with_metadata<Currency>(
    payer: &signer,
    payee: address,
    amount: u64,
    metadata: vector<u8>,
    metadata_signature: vector<u8>
) {
    let payer_withdrawal_cap = LibraAccount::extract_withdraw_capability(payer);
    LibraAccount::pay_from<Currency>(
        &payer_withdrawal_cap, payee, amount, metadata, metadata_signature
    );
    LibraAccount::restore_withdraw_capability(payer_withdrawal_cap);
}

spec fun peer_to_peer_with_metadata {
	pragma verify = false; // TODO: times out
    let payer_addr = Signer::spec_address_of(payer);

    /// ## Post conditions
    /// The balances of payer and payee are changed correctly if payer and payee are different.
    ensures payer_addr != payee
                ==> spec_balance_of<Currency>(payee) == old(spec_balance_of<Currency>(payee)) + amount;
    ensures payer_addr != payee
                ==> spec_balance_of<Currency>(payer_addr) == old(spec_balance_of<Currency>(payer_addr)) - amount;

    /// If payer and payee are the same, the balance does not change.
    ensures payer_addr == payee ==> spec_balance_of<Currency>(payee) == old(spec_balance_of<Currency>(payee));


    /// ## Abort conditions
    include AbortsIfPayerInvalid<Currency>{payer: payer_addr};
    include AbortsIfPayeeInvalid<Currency>;
    include AbortsIfAmountInvalid<Currency>{payer: payer_addr};
    include DualAttestation::AssertPaymentOkAbortsIf<Currency>{payer: payer_addr, value: amount};
    include AbortsIfAmountExceedsLimit<Currency>{payer: payer_addr};
    include LibraAccount::spec_should_track_limits_for_account<Currency>(payer_addr, payee, false) ==>
                AccountLimits::UpdateDepositLimitsAbortsIf<Currency> {
                    addr: VASP::spec_parent_address(payee),
                };
    include LibraAccount::spec_should_track_limits_for_account<Currency>(payer_addr, payee, true) ==>
                AccountLimits::UpdateWithdrawalLimitsAbortsIf<Currency> {
                    addr: VASP::spec_parent_address(payer_addr),
                };
}

spec module {
    pragma verify = true, aborts_if_is_strict = true;

    /// Returns the value of balance under addr.
    define spec_balance_of<Currency>(addr: address): u64 {
        global<Balance<Currency>>(addr).coin.value
    }
}

spec schema AbortsIfPayerInvalid<Currency> {
    payer: address;
    aborts_if !exists<LibraAccount>(payer);
    aborts_if AccountFreezing::account_is_frozen(payer);
    aborts_if !exists<Balance<Currency>>(payer);
    /// Aborts if payer's withdrawal_capability has been delegated.
    aborts_if LibraAccount::delegated_withdraw_capability(payer);
}

spec schema AbortsIfPayeeInvalid<Currency> {
    payee: address;
    aborts_if !exists<LibraAccount>(payee);
    aborts_if AccountFreezing::account_is_frozen(payee);
    aborts_if !exists<Balance<Currency>>(payee);
}

spec schema AbortsIfAmountInvalid<Currency> {
    payer: address;
    payee: address;
    amount: u64;
    aborts_if amount == 0;
    /// Aborts if arithmetic overflow happens.
    aborts_if global<Balance<Currency>>(payer).coin.value < amount;
    aborts_if payer != payee
            && global<Balance<Currency>>(payee).coin.value + amount > max_u64();
}

spec schema AbortsIfAmountExceedsLimit<Currency> {
    payer: address;
    payee: address;
    amount: u64;
    /// Aborts if the amount exceeds payee's deposit limit.
    aborts_if LibraAccount::spec_should_track_limits_for_account<Currency>(payer, payee, false)
                && (!LibraAccount::spec_has_account_operations_cap()
                    || !AccountLimits::spec_update_deposit_limits<Currency>(
                            amount,
                            VASP::spec_parent_address(payee)
                        )
                    );
    /// Aborts if the amount exceeds payer's withdraw limit.
    aborts_if LibraAccount::spec_should_track_limits_for_account<Currency>(payer, payee, true)
                && (!LibraAccount::spec_has_account_operations_cap()
                    || !AccountLimits::spec_update_withdrawal_limits<Currency>(
                            amount,
                            VASP::spec_parent_address(payer)
                        )
                    );
}

spec module {
    pragma verify = true;
}
}
