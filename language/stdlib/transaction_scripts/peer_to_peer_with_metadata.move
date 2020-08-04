script {
use 0x1::LibraAccount;
// Prover deps:
use 0x1::LibraAccount::{Balance, LibraAccount, WithdrawCapability};
use 0x1::AccountFreezing;
use 0x1::AccountLimits;
use 0x1::DualAttestation;
use 0x1::Option;
use 0x1::Signer;
use 0x1::VASP;

/// Transfer `amount` coins of type `Currency` from `payer` to `payee` with (optional) associated
/// `metadata` and an (optional) `metadata_signature` on the message
/// `metadata` | `Signer::address_of(payer)` | `amount` | `DualAttestation::DOMAIN_SEPARATOR`.
/// The `metadata` and `metadata_signature` parameters are only required if `amount` >=
/// `DualAttestation::get_cur_microlibra_limit` LBR and `payer` and `payee` are distinct VASPs.
/// However, a transaction sender can opt in to dual attestation even when it is not required (e.g., a DesignatedDealer -> VASP payment) by providing a non-empty `metadata_signature`.
/// Standardized `metadata` LCS format can be found in `libra_types::transaction::metadata::Metadata`.
///
/// ## Events
/// When this script executes without aborting, it emits two events:
/// `SentPaymentEvent { amount, currency_code = Currency, payee, metadata }`
/// on `payer`'s `LibraAccount::sent_events` handle, and
///  `ReceivedPaymentEvent { amount, currency_code = Currency, payer, metadata }`
/// on `payee`'s `LibraAccount::received_events` handle.
///
/// ## Common Aborts
/// These aborts can in occur in any payment.
/// * Aborts with `LibraAccount::EINSUFFICIENT_BALANCE` if `amount` is greater than `payer`'s balance in `Currency`.
/// * Aborts with `LibraAccount::ECOIN_DEPOSIT_IS_ZERO` if `amount` is zero.
/// * Aborts with `LibraAccount::EPAYEE_DOES_NOT_EXIST` if no account exists at the address `payee`.
/// * Aborts with `LibraAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE` if an account exists at `payee`, but it does not accept payments in `Currency`.
///
/// ## Dual Attestation Aborts
/// These aborts can occur in any payment subject to dual attestation.
/// * Aborts with `DualAttestation::EMALFORMED_METADATA_SIGNATURE` if `metadata_signature`'s is not 64 bytes.
/// * Aborts with `DualAttestation:EINVALID_METADATA_SIGNATURE` if `metadata_signature` does not verify on the message `metadata` | `payer` | `value` | `DOMAIN_SEPARATOR` using the `compliance_public_key` published in the `payee`'s `DualAttestation::Credential` resource.
///
/// ## Other Aborts
/// These aborts should only happen when `payer` or `payee` have account limit restrictions or
/// have been frozen by Libra administrators.
/// * Aborts with `LibraAccount::EWITHDRAWAL_EXCEEDS_LIMITS` if `payer` has exceeded their daily
/// withdrawal limits.
/// * Aborts with `LibraAccount::EDEPOSIT_EXCEEDS_LIMITS` if `payee` has exceeded their daily deposit limits.
/// * Aborts with `LibraAccount::EACCOUNT_FROZEN` if `payer`'s account is frozen.

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

    /// > TODO(emmazzz): This pre-condition is supposed to be a global property in LibraAccount:
    ///  The LibraAccount under addr holds either no withdraw capability
    ///  or the withdraw capability for addr itself.
    requires spec_get_withdraw_cap(Signer::spec_address_of(payer)).account_address
        == Signer::spec_address_of(payer);

    include AbortsIfPayerInvalid<Currency>;
    include AbortsIfPayeeInvalid<Currency>;
    include AbortsIfAmountInvalid<Currency>;
    include DualAttestation::AssertPaymentOkAbortsIf<Currency>{payer: Signer::spec_address_of(payer), value: amount};
    include AbortsIfAmountExceedsLimit<Currency>;

    /// Post condition: the balances of payer and payee are changed correctly.
    ensures Signer::spec_address_of(payer) != payee
                ==> spec_balance_of<Currency>(payee) == old(spec_balance_of<Currency>(payee)) + amount;
    ensures Signer::spec_address_of(payer) != payee
                ==> spec_balance_of<Currency>(Signer::spec_address_of(payer))
                    == old(spec_balance_of<Currency>(Signer::spec_address_of(payer))) - amount;
    /// If payer and payee are the same, the balance does not change.
    ensures Signer::spec_address_of(payer) == payee
                ==> spec_balance_of<Currency>(payee) == old(spec_balance_of<Currency>(payee));
}

spec module {
    pragma verify = true, aborts_if_is_strict = true;

    /// Returns the `withdrawal_capability` of LibraAccount under `addr`.
    define spec_get_withdraw_cap(addr: address): WithdrawCapability {
        Option::spec_get(global<LibraAccount>(addr).withdrawal_capability)
    }

    /// Returns the value of balance under addr.
    define spec_balance_of<Currency>(addr: address): u64 {
        global<Balance<Currency>>(addr).coin.value
    }
}

spec schema AbortsIfPayerInvalid<Currency> {
    payer: signer;
    aborts_if !exists<LibraAccount>(Signer::spec_address_of(payer));
    aborts_if AccountFreezing::spec_account_is_frozen(Signer::spec_address_of(payer));
    aborts_if !exists<Balance<Currency>>(Signer::spec_address_of(payer));
    /// Aborts if payer's withdrawal_capability has been delegated.
    aborts_if Option::spec_is_none(
        global<LibraAccount>(
            Signer::spec_address_of(payer)
        ).withdrawal_capability);
}

spec schema AbortsIfPayeeInvalid<Currency> {
    payee: address;
    aborts_if !exists<LibraAccount>(payee);
    aborts_if AccountFreezing::spec_account_is_frozen(payee);
    aborts_if !exists<Balance<Currency>>(payee);
}

spec schema AbortsIfAmountInvalid<Currency> {
    payer: &signer;
    payee: address;
    amount: u64;
    aborts_if amount == 0;
    /// Aborts if arithmetic overflow happens.
    aborts_if global<Balance<Currency>>(Signer::spec_address_of(payer)).coin.value < amount;
    aborts_if Signer::spec_address_of(payer) != payee
            && global<Balance<Currency>>(payee).coin.value + amount > max_u64();
}

spec schema AbortsIfAmountExceedsLimit<Currency> {
    payer: &signer;
    payee: address;
    amount: u64;
    /// Aborts if the amount exceeds payee's deposit limit.
    aborts_if LibraAccount::spec_should_track_limits_for_account(Signer::spec_address_of(payer), payee, false)
                && (!LibraAccount::spec_has_account_operations_cap()
                    || !AccountLimits::spec_update_deposit_limits<Currency>(
                            amount,
                            VASP::spec_parent_address(payee)
                        )
                    );
    /// Aborts if the amount exceeds payer's withdraw limit.
    aborts_if LibraAccount::spec_should_track_limits_for_account(Signer::spec_address_of(payer), payee, true)
                && (!LibraAccount::spec_has_account_operations_cap()
                    || !AccountLimits::spec_update_withdrawal_limits<Currency>(
                            amount,
                            VASP::spec_parent_address(Signer::spec_address_of(payer))
                        )
                    );
}

spec module {
    /// > TODO(emmazzz): turn verify on when non-termination issue is resolved.
    pragma verify = false;
}
}
