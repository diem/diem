
<a name="burn"></a>

# Script `burn`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
    -  [Events](#@Events_2)
-  [Parameters](#@Parameters_3)
-  [Common Abort Conditions](#@Common_Abort_Conditions_4)
-  [Related Scripts](#@Related_Scripts_5)


<a name="@Summary_0"></a>

## Summary

Burns all coins held in the preburn resource at the specified
preburn address and removes them from the system. The sending account must
be the Treasury Compliance account.
The account that holds the preburn resource will normally be a Designated
Dealer, but there are no enforced requirements that it be one.


<a name="@Technical_Description_1"></a>

## Technical Description

This transaction permanently destroys all the coins of <code>Token</code> type
stored in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource published under the
<code>preburn_address</code> account address.

This transaction will only succeed if the sending <code>account</code> has a
<code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;</code>, and a <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource
exists under <code>preburn_address</code>, with a non-zero <code>to_burn</code> field. After the successful execution
of this transaction the <code>total_value</code> field in the
<code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token&gt;</code> resource published under <code>0xA550C18</code> will be
decremented by the value of the <code>to_burn</code> field of the preburn resource
under <code>preburn_address</code> immediately before this transaction, and the
<code>to_burn</code> field of the preburn resource will have a zero value.


<a name="@Events_2"></a>

### Events

The successful execution of this transaction will emit a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnEvent">Libra::BurnEvent</a></code> on the event handle
held in the <code><a href="../../modules/doc/Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token&gt;</code> resource's <code>burn_events</code> published under
<code>0xA550C18</code>.


<a name="@Parameters_3"></a>

## Parameters

| Name              | Type      | Description                                                                                                                  |
| ------            | ------    | -------------                                                                                                                |
| <code>Token</code>           | Type      | The Move type for the <code>Token</code> currency being burned. <code>Token</code> must be an already-registered currency on-chain.                |
| <code>tc_account</code>      | <code>&signer</code> | The signer reference of the sending account of this transaction, must have a burn capability for <code>Token</code> published under it. |
| <code>sliding_nonce</code>   | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                                   |
| <code>preburn_address</code> | <code>address</code> | The address where the coins to-be-burned are currently held.                                                                 |


<a name="@Common_Abort_Conditions_4"></a>

## Common Abort Conditions

| Error Category                | Error Reason                            | Description                                                                                           |
| ----------------              | --------------                          | -------------                                                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not.            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code>    | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a></code> | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EBURN_CAPABILITY">Libra::EBURN_CAPABILITY</a></code>               | The sending <code>account</code> does not have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;</code> published under it.              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EPREBURN">Libra::EPREBURN</a></code>                       | The account at <code>preburn_address</code> does not have a <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource published under it. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_EPREBURN_EMPTY">Libra::EPREBURN_EMPTY</a></code>                 | The <code><a href="../../modules/doc/Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;</code> resource is empty (has a value of 0).                                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a></code>       | <code><a href="../../modules/doc/Libra.md#0x1_Libra_ECURRENCY_INFO">Libra::ECURRENCY_INFO</a></code>                 | The specified <code>Token</code> is not a registered currency on-chain.                                          |


<a name="@Related_Scripts_5"></a>

## Related Scripts

* <code><a href="burn_txn_fees.md#burn_txn_fees">Script::burn_txn_fees</a></code>
* <code><a href="cancel_burn.md#cancel_burn">Script::cancel_burn</a></code>
* <code><a href="preburn.md#preburn">Script::preburn</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="burn.md#burn">burn</a>&lt;Token&gt;(account: &signer, sliding_nonce: u64, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="burn.md#burn">burn</a>&lt;Token&gt;(account: &signer, sliding_nonce: u64, preburn_address: address) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <a href="../../modules/doc/Libra.md#0x1_Libra_burn">Libra::burn</a>&lt;Token&gt;(account, preburn_address)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">SlidingNonce::RecordNonceAbortsIf</a>{ seq_nonce: sliding_nonce };
<b>include</b> <a href="../../modules/doc/Libra.md#0x1_Libra_BurnAbortsIf">Libra::BurnAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="../../modules/doc/Libra.md#0x1_Libra_BurnEnsures">Libra::BurnEnsures</a>&lt;Token&gt;;
</code></pre>



</details>
