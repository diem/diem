
<a name="tiered_mint"></a>

# Script `tiered_mint`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
    -  [Events](#@Events_2)
-  [Parameters](#@Parameters_3)
-  [Common Abort Conditions](#@Common_Abort_Conditions_4)
-  [Related Scripts](#@Related_Scripts_5)


<a name="@Summary_0"></a>

## Summary

Mints a specified number of coins in a currency to a Designated Dealer. The sending account
must be the Treasury Compliance account, and coins can only be minted to a Designated Dealer
account.


<a name="@Technical_Description_1"></a>

## Technical Description

Mints <code>mint_amount</code> of coins in the <code>CoinType</code> currency to Designated Dealer account at
<code>designated_dealer_address</code>. The <code>tier_index</code> parameter specifies which tier should be used to
check verify the off-chain approval policy, and is based in part on the on-chain tier values
for the specific Designated Dealer, and the number of <code>CoinType</code> coins that have been minted to
the dealer over the past 24 hours. Every Designated Dealer has 4 tiers for each currency that
they support. The sending <code>tc_account</code> must be the Treasury Compliance account, and the
receiver an authorized Designated Dealer account.


<a name="@Events_2"></a>

### Events

Successful execution of the transaction will emit two events:
* A <code><a href="../../modules/doc/Libra.md#0x1_Libra_MintEvent">Libra::MintEvent</a></code> with the amount and currency code minted is emitted on the
<code>mint_event_handle</code> in the stored <code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;CoinType&gt;</code> resource stored under
<code>0xA550C18</code>; and
* A <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_ReceivedMintEvent">DesignatedDealer::ReceivedMintEvent</a></code> with the amount, currency code, and Designated
Dealer's address is emitted on the <code>mint_event_handle</code> in the stored <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a></code>
resource published under the <code>designated_dealer_address</code>.


<a name="@Parameters_3"></a>

## Parameters

| Name                        | Type      | Description                                                                                                |
| ------                      | ------    | -------------                                                                                              |
| <code>CoinType</code>                  | Type      | The Move type for the <code>CoinType</code> being minted. <code>CoinType</code> must be an already-registered currency on-chain. |
| <code>tc_account</code>                | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.  |
| <code>sliding_nonce</code>             | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                 |
| <code>designated_dealer_address</code> | <code>address</code> | The address of the Designated Dealer account being minted to.                                              |
| <code>mint_amount</code>               | <code>u64</code>     | The number of coins to be minted.                                                                          |
| <code>tier_index</code>                | <code>u64</code>     | The mint tier index to use for the Designated Dealer account.                                              |


<a name="@Common_Abort_Conditions_4"></a>

## Common Abort Conditions

| Error Category                | Error Reason                                 | Description                                                                                                                  |
| ----------------              | --------------                               | -------------                                                                                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>               | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not.                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>               | The <code>sliding_nonce</code> is too far in the future.                                                                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>      | The <code>sliding_nonce</code> has been previously recorded.                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code>    | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>        | <code>tc_account</code> is not the Treasury Compliance account.                                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_MINT_AMOUNT">DesignatedDealer::EINVALID_MINT_AMOUNT</a></code>     | <code>mint_amount</code> is zero.                                                                                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_EDEALER">DesignatedDealer::EDEALER</a></code>                  | <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a></code> or <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_TierInfo">DesignatedDealer::TierInfo</a>&lt;CoinType&gt;</code> resource does not exist at <code>designated_dealer_address</code>. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_TIER_INDEX">DesignatedDealer::EINVALID_TIER_INDEX</a></code>      | The <code>tier_index</code> is out of bounds.                                                                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/DesignatedDealer.md#0x1_DesignatedDealer_EINVALID_AMOUNT_FOR_TIER">DesignatedDealer::EINVALID_AMOUNT_FOR_TIER</a></code> | <code>mint_amount</code> exceeds the maximum allowed amount for <code>tier_index</code>.                                                           |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EMINT_CAPABILITY">Libra::EMINT_CAPABILITY</a></code>                    | <code>tc_account</code> does not have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;CoinType&gt;</code> resource published under it.                                  |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EMINTING_NOT_ALLOWED">Libra::EMINTING_NOT_ALLOWED</a></code>                | Minting is not currently allowed for <code>CoinType</code> coins.                                                                       |


<a name="@Related_Scripts_5"></a>

## Related Scripts

* <code><a href="create_designated_dealer.md#create_designated_dealer">Script::create_designated_dealer</a></code>
* <code><a href="overview.md#peer_to_peer_with_metadata">Script::peer_to_peer_with_metadata</a></code>
* <code><a href="rotate_dual_attestation_info.md#rotate_dual_attestation_info">Script::rotate_dual_attestation_info</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="tiered_mint.md#tiered_mint">tiered_mint</a>&lt;CoinType&gt;(tc_account: &signer, sliding_nonce: u64, designated_dealer_address: address, mint_amount: u64, tier_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="tiered_mint.md#tiered_mint">tiered_mint</a>&lt;CoinType&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    designated_dealer_address: address,
    mint_amount: u64,
    tier_index: u64
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_tiered_mint">LibraAccount::tiered_mint</a>&lt;CoinType&gt;(
        tc_account, designated_dealer_address, mint_amount, tier_index
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{account: tc_account, seq_nonce: sliding_nonce};
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TieredMintAbortsIf">LibraAccount::TieredMintAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_TieredMintEnsures">LibraAccount::TieredMintEnsures</a>&lt;CoinType&gt;;
</code></pre>



</details>
