
<a name="update_exchange_rate"></a>

# Script `update_exchange_rate`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
-  [Parameters](#@Parameters_2)
-  [Common Abort Conditions](#@Common_Abort_Conditions_3)
-  [Related Scripts](#@Related_Scripts_4)


<a name="@Summary_0"></a>

## Summary

Update the rough on-chain exchange rate between a specified currency and LBR (as a conversion
to micro-LBR). The transaction can only be sent by the Treasury Compliance account. After this
transaction the updated exchange rate will be used for normalization of gas prices, and for
dual attestation checking.


<a name="@Technical_Description_1"></a>

## Technical Description

Updates the on-chain exchange rate from the given <code>Currency</code> to micro-LBR.  The exchange rate
is given by <code>new_exchange_rate_numerator/new_exchange_rate_denominator</code>.


<a name="@Parameters_2"></a>

## Parameters

| Name                            | Type      | Description                                                                                                                        |
| ------                          | ------    | -------------                                                                                                                      |
| <code>Currency</code>                      | Type      | The Move type for the <code>Currency</code> whose exchange rate is being updated. <code>Currency</code> must be an already-registered currency on-chain. |
| <code>tc_account</code>                    | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account.                          |
| <code>sliding_nonce</code>                 | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for the transaction.                                                          |
| <code>new_exchange_rate_numerator</code>   | <code>u64</code>     | The numerator for the new to micro-LBR exchange rate for <code>Currency</code>.                                                               |
| <code>new_exchange_rate_denominator</code> | <code>u64</code>     | The denominator for the new to micro-LBR exchange rate for <code>Currency</code>.                                                             |


<a name="@Common_Abort_Conditions_3"></a>

## Common Abort Conditions

| Error Category             | Error Reason                            | Description                                                                                |
| ----------------           | --------------                          | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>   | <code>tc_account</code> is not the Treasury Compliance account.                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_EDENOMINATOR">FixedPoint32::EDENOMINATOR</a></code>            | <code>new_exchange_rate_denominator</code> is zero.                                                   |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">FixedPoint32::ERATIO_OUT_OF_RANGE</a></code>     | The quotient is unrepresentable as a <code><a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a></code>.                                       |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a></code>   | <code><a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">FixedPoint32::ERATIO_OUT_OF_RANGE</a></code>     | The quotient is unrepresentable as a <code><a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a></code>.                                       |


<a name="@Related_Scripts_4"></a>

## Related Scripts

* <code>Scripts::update_dual_attestation_limit</code>
* <code>Scripts::update_minting_ability</code>


<pre><code><b>public</b> <b>fun</b> <a href="update_exchange_rate.md#update_exchange_rate">update_exchange_rate</a>&lt;Currency&gt;(tc_account: &signer, sliding_nonce: u64, new_exchange_rate_numerator: u64, new_exchange_rate_denominator: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_exchange_rate.md#update_exchange_rate">update_exchange_rate</a>&lt;Currency&gt;(
    tc_account: &signer,
    sliding_nonce: u64,
    new_exchange_rate_numerator: u64,
    new_exchange_rate_denominator: u64,
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <b>let</b> rate = <a href="../../modules/doc/FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(
        new_exchange_rate_numerator,
        new_exchange_rate_denominator,
    );
    <a href="../../modules/doc/Libra.md#0x1_Libra_update_lbr_exchange_rate">Libra::update_lbr_exchange_rate</a>&lt;Currency&gt;(tc_account, rate);
}
</code></pre>



</details>
