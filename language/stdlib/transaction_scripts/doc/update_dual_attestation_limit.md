
<a name="update_dual_attestation_limit"></a>

# Script `update_dual_attestation_limit`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
-  [Parameters](#@Parameters_2)
-  [Common Abort Conditions](#@Common_Abort_Conditions_3)
-  [Related Scripts](#@Related_Scripts_4)


<a name="@Summary_0"></a>

## Summary

Update the dual attestation limit on-chain. Defined in terms of micro-LBR.  The transaction can
only be sent by the Treasury Compliance account.  After this transaction all inter-VASP
payments over this limit must be checked for dual attestation.


<a name="@Technical_Description_1"></a>

## Technical Description

Updates the <code>micro_lbr_limit</code> field of the <code><a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_Limit">DualAttestation::Limit</a></code> resource published under
<code>0xA550C18</code>. The amount is set in micro-LBR.


<a name="@Parameters_2"></a>

## Parameters

| Name                  | Type      | Description                                                                                               |
| ------                | ------    | -------------                                                                                             |
| <code>tc_account</code>          | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account. |
| <code>sliding_nonce</code>       | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                |
| <code>new_micro_lbr_limit</code> | <code>u64</code>     | The new dual attestation limit to be used on-chain.                                                       |


<a name="@Common_Abort_Conditions_3"></a>

## Common Abort Conditions

| Error Category             | Error Reason                            | Description                                                                                |
| ----------------           | --------------                          | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>   | <code>tc_account</code> is not the Treasury Compliance account.                                       |


<a name="@Related_Scripts_4"></a>

## Related Scripts

* <code>Scripts::update_exchange_rate</code>
* <code>Scripts::update_minting_ability</code>


<pre><code><b>public</b> <b>fun</b> <a href="update_dual_attestation_limit.md#update_dual_attestation_limit">update_dual_attestation_limit</a>(tc_account: &signer, sliding_nonce: u64, new_micro_lbr_limit: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_dual_attestation_limit.md#update_dual_attestation_limit">update_dual_attestation_limit</a>(
    tc_account: &signer,
    sliding_nonce: u64,
    new_micro_lbr_limit: u64
) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/DualAttestation.md#0x1_DualAttestation_set_microlibra_limit">DualAttestation::set_microlibra_limit</a>(tc_account, new_micro_lbr_limit);
}
</code></pre>



</details>
