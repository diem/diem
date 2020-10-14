script {
use 0x1::LibraAccount;

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
    use 0x1::Signer;
    use 0x1::Errors;

    include LibraAccount::TransactionChecks{sender: payer}; // properties checked by the prologue.
    let payer_addr = Signer::spec_address_of(payer);
    let cap = LibraAccount::spec_get_withdraw_cap(payer_addr);
    include LibraAccount::ExtractWithdrawCapAbortsIf{sender_addr: payer_addr};
    include LibraAccount::PayFromAbortsIf<Currency>{cap: cap};

    /// The balances of payer and payee change by the correct amount.
    ensures payer_addr != payee
        ==> LibraAccount::balance<Currency>(payer_addr)
        == old(LibraAccount::balance<Currency>(payer_addr)) - amount;
    ensures payer_addr != payee
        ==> LibraAccount::balance<Currency>(payee)
        == old(LibraAccount::balance<Currency>(payee)) + amount;
    ensures payer_addr == payee
        ==> LibraAccount::balance<Currency>(payee)
        == old(LibraAccount::balance<Currency>(payee));

    aborts_with [check]
        Errors::NOT_PUBLISHED,
        Errors::INVALID_STATE,
        Errors::INVALID_ARGUMENT,
        Errors::LIMIT_EXCEEDED;

    /// **Access Control:**
    /// Both the payer and the payee must hold the balances of the Currency. Only Designated Dealers,
    /// Parent VASPs, and Child VASPs can hold balances [[D1]][ROLE][[D2]][ROLE][[D3]][ROLE][[D4]][ROLE][[D5]][ROLE][[D6]][ROLE][[D7]][ROLE].
    aborts_if !exists<LibraAccount::Balance<Currency>>(payer_addr) with Errors::NOT_PUBLISHED;
    aborts_if !exists<LibraAccount::Balance<Currency>>(payee) with Errors::INVALID_ARGUMENT;
}
}
