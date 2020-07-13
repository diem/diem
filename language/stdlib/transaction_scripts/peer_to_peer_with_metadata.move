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

/// Transfer `amount` coins to `recipient_address` with (optional)
/// associated metadata `metadata` and (optional) `signature` on the metadata, amount, and
/// sender address. The `metadata` and `signature` parameters are only required if
/// `amount` >= 1_000_000 micro LBR and the sender and recipient of the funds are two distinct VASPs.
/// Fails if there is no account at the recipient address or if the sender's balance is lower
/// than `amount`.
fun peer_to_peer_with_metadata<Token>(
    payer: &signer,
    payee: address,
    amount: u64,
    metadata: vector<u8>,
    metadata_signature: vector<u8>
) {
    let payer_withdrawal_cap = LibraAccount::extract_withdraw_capability(payer);
    LibraAccount::pay_from<Token>(&payer_withdrawal_cap, payee, amount, metadata, metadata_signature);
    LibraAccount::restore_withdraw_capability(payer_withdrawal_cap);
}

spec fun peer_to_peer_with_metadata {

    /// > TODO(emmazzz): This pre-condition is supposed to be a global property in LibraAccount:
    ///  The LibraAccount under addr holds either no withdraw capability
    ///  or the withdraw capability for addr itself.
    requires spec_get_withdraw_cap(Signer::spec_address_of(payer)).account_address
        == Signer::spec_address_of(payer);

    include AbortsIfPayerInvalid<Token>;
    include AbortsIfPayeeInvalid<Token>;
    include AbortsIfAmountInvalid<Token>;
    include DualAttestation::TravelRuleAppliesAbortsIf<Token>;
    include AbortsIfAmountExceedsLimit<Token>;

    /// Post condition: the balances of payer and payee are changed correctly.
    ensures Signer::spec_address_of(payer) != payee
                ==> spec_balance_of<Token>(payee) == old(spec_balance_of<Token>(payee)) + amount;
    ensures Signer::spec_address_of(payer) != payee
                ==> spec_balance_of<Token>(Signer::spec_address_of(payer))
                    == old(spec_balance_of<Token>(Signer::spec_address_of(payer))) - amount;
    /// If payer and payee are the same, the balance does not change.
    ensures Signer::spec_address_of(payer) == payee
                ==> spec_balance_of<Token>(payee) == old(spec_balance_of<Token>(payee));
}

spec module {
    pragma verify = true, aborts_if_is_strict = true;

    /// Returns the `withdrawal_capability` of LibraAccount under `addr`.
    define spec_get_withdraw_cap(addr: address): WithdrawCapability {
        Option::spec_value_inside(global<LibraAccount>(addr).withdrawal_capability)
    }

    /// Returns the value of balance under addr.
    define spec_balance_of<Token>(addr: address): u64 {
        global<Balance<Token>>(addr).coin.value
    }
}

spec schema AbortsIfPayerInvalid<Token> {
    payer: signer;
    aborts_if !exists<LibraAccount>(Signer::spec_address_of(payer));
    aborts_if AccountFreezing::spec_account_is_frozen(Signer::spec_address_of(payer));
    aborts_if !exists<Balance<Token>>(Signer::spec_address_of(payer));
    /// Aborts if payer's withdrawal_capability has been delegated.
    aborts_if Option::spec_is_none(
        global<LibraAccount>(
            Signer::spec_address_of(payer)
        ).withdrawal_capability);
}

spec schema AbortsIfPayeeInvalid<Token> {
    payee: address;
    aborts_if !exists<LibraAccount>(payee);
    aborts_if AccountFreezing::spec_account_is_frozen(payee);
    aborts_if !exists<Balance<Token>>(payee);
}

spec schema AbortsIfAmountInvalid<Token> {
    payer: &signer;
    payee: address;
    amount: u64;
    aborts_if amount == 0;
    /// Aborts if arithmetic overflow happens.
    aborts_if global<Balance<Token>>(Signer::spec_address_of(payer)).coin.value < amount;
    aborts_if Signer::spec_address_of(payer) != payee
            && global<Balance<Token>>(payee).coin.value + amount > max_u64();
}

spec schema AbortsIfAmountExceedsLimit<Token> {
    payer: &signer;
    payee: address;
    amount: u64;
    /// Aborts if the amount exceeds payee's deposit limit.
    aborts_if LibraAccount::spec_should_track_limits_for_account(Signer::spec_address_of(payer), payee, false)
                && (!LibraAccount::spec_has_account_operations_cap()
                    || !AccountLimits::spec_update_deposit_limits<Token>(
                            amount,
                            VASP::spec_parent_address(payee)
                        )
                    );
    /// Aborts if the amount exceeds payer's withdraw limit.
    aborts_if LibraAccount::spec_should_track_limits_for_account(Signer::spec_address_of(payer), payee, true)
                && (!LibraAccount::spec_has_account_operations_cap()
                    || !AccountLimits::spec_update_withdrawal_limits<Token>(
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
