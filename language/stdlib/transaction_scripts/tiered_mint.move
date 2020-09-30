script {
use 0x1::LibraAccount;
use 0x1::SlidingNonce;

/// # Summary
/// Mints a specified number of coins in a currency to a Designated Dealer. The sending account
/// must be the Treasury Compliance account, and coins can only be minted to a Designated Dealer
/// account.
///
/// # Technical Description
/// Mints `mint_amount` of coins in the `CoinType` currency to Designated Dealer account at
/// `designated_dealer_address`. The `tier_index` parameter specifies which tier should be used to
/// check verify the off-chain approval policy, and is based in part on the on-chain tier values
/// for the specific Designated Dealer, and the number of `CoinType` coins that have been minted to
/// the dealer over the past 24 hours. Every Designated Dealer has 4 tiers for each currency that
/// they support. The sending `tc_account` must be the Treasury Compliance account, and the
/// receiver an authorized Designated Dealer account.
///
/// ## Events
/// Successful execution of the transaction will emit two events:
/// * A `Libra::MintEvent` with the amount and currency code minted is emitted on the
/// `mint_event_handle` in the stored `Libra::CurrencyInfo<CoinType>` resource stored under
/// `0xA550C18`; and
/// * A `DesignatedDealer::ReceivedMintEvent` with the amount, currency code, and Designated
/// Dealer's address is emitted on the `mint_event_handle` in the stored `DesignatedDealer::Dealer`
/// resource published under the `designated_dealer_address`.
///
/// # Parameters
/// | Name                        | Type      | Description                                                                                                |
/// | ------                      | ------    | -------------                                                                                              |
/// | `CoinType`                  | Type      | The Move type for the `CoinType` being minted. `CoinType` must be an already-registered currency on-chain. |
/// | `tc_account`                | `&signer` | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.  |
/// | `sliding_nonce`             | `u64`     | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                 |
/// | `designated_dealer_address` | `address` | The address of the Designated Dealer account being minted to.                                              |
/// | `mint_amount`               | `u64`     | The number of coins to be minted.                                                                          |
/// | `tier_index`                | `u64`     | The mint tier index to use for the Designated Dealer account.                                              |
///
/// # Common Abort Conditions
/// | Error Category                | Error Reason                                 | Description                                                                                                                  |
/// | ----------------              | --------------                               | -------------                                                                                                                |
/// | `Errors::INVALID_ARGUMENT`    | `SlidingNonce::ENONCE_TOO_OLD`               | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not.                                   |
/// | `Errors::INVALID_ARGUMENT`    | `SlidingNonce::ENONCE_TOO_NEW`               | The `sliding_nonce` is too far in the future.                                                                                |
/// | `Errors::INVALID_ARGUMENT`    | `SlidingNonce::ENONCE_ALREADY_RECORDED`      | The `sliding_nonce` has been previously recorded.                                                                            |
/// | `Errors::REQUIRES_ADDRESS`    | `CoreAddresses::ETREASURY_COMPLIANCE`        | `tc_account` is not the Treasury Compliance account.                                                                         |
/// | `Errors::INVALID_ARGUMENT`    | `DesignatedDealer::EINVALID_MINT_AMOUNT`     | `mint_amount` is zero.                                                                                                       |
/// | `Errors::NOT_PUBLISHED`       | `DesignatedDealer::EDEALER`                  | `DesignatedDealer::Dealer` or `DesignatedDealer::TierInfo<CoinType>` resource does not exist at `designated_dealer_address`. |
/// | `Errors::INVALID_ARGUMENT`    | `DesignatedDealer::EINVALID_TIER_INDEX`      | The `tier_index` is out of bounds.                                                                                           |
/// | `Errors::INVALID_ARGUMENT`    | `DesignatedDealer::EINVALID_AMOUNT_FOR_TIER` | `mint_amount` exceeds the maximum allowed amount for `tier_index`.                                                           |
/// | `Errors::REQUIRES_CAPABILITY` | `Libra::EMINT_CAPABILITY`                    | `tc_account` does not have a `Libra::MintCapability<CoinType>` resource published under it.                                  |
/// | `Errors::INVALID_STATE`       | `Libra::EMINTING_NOT_ALLOWED`                | Minting is not currently allowed for `CoinType` coins.                                                                       |
///
/// # Related Scripts
/// * `Script::create_designated_dealer`
/// * `Script::peer_to_peer_with_metadata`
/// * `Script::rotate_dual_attestation_info`

fun tiered_mint<CoinType>(
    tc_account: &signer,
    sliding_nonce: u64,
    designated_dealer_address: address,
    mint_amount: u64,
    tier_index: u64
) {
    SlidingNonce::record_nonce_or_abort(tc_account, sliding_nonce);
    LibraAccount::tiered_mint<CoinType>(
        tc_account, designated_dealer_address, mint_amount, tier_index
    );
}

spec fun tiered_mint {
    use 0x1::Errors;

    include SlidingNonce::RecordNonceAbortsIf{account: tc_account, seq_nonce: sliding_nonce};
    include LibraAccount::TieredMintAbortsIf<CoinType>;
    include LibraAccount::TieredMintEnsures<CoinType>;

    aborts_with [check]
        Errors::INVALID_ARGUMENT,
        Errors::REQUIRES_ADDRESS,
        Errors::NOT_PUBLISHED,
        Errors::REQUIRES_CAPABILITY,
        Errors::INVALID_STATE,
        Errors::LIMIT_EXCEEDED, // TODO: Undocumented error code. Possibly raised in LibraAccount::deposit, Libra::deposit, and AccountLimits::can_receive.
        Errors::REQUIRES_ROLE; // TODO: Undocumented error code. Added due to Roles::assert_treasury_compliance in DesginatedDealer::tiered_mint
}
}
