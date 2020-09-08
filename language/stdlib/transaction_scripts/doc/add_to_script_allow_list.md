
<a name="SCRIPT"></a>

# Script `add_to_script_allow_list.move`

### Table of Contents

-  [Function `add_to_script_allow_list`](#SCRIPT_add_to_script_allow_list)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)



<a name="SCRIPT_add_to_script_allow_list"></a>

## Function `add_to_script_allow_list`


<a name="SCRIPT_@Summary"></a>

### Summary

Adds a script hash to the transaction allowlist. This transaction
can only be sent by the Libra Root account. Scripts with this hash can be
sent afterward the successful execution of this script.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description


The sending account (<code>lr_account</code>) must be the Libra Root account. The script allow
list must not already hold the script <code>hash</code> being added. The <code>sliding_nonce</code> must be
a valid nonce for the Libra Root account. After this transaction has executed
successfully a reconfiguration will be initiated, and the on-chain config
<code><a href="../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_LibraTransactionPublishingOption">LibraTransactionPublishingOption::LibraTransactionPublishingOption</a></code>'s
<code>script_allow_list</code> field will contain the new script <code>hash</code> and transactions
with this <code>hash</code> can be successfully sent to the network.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name            | Type         | Description                                                                                     |
| ------          | ------       | -------------                                                                                   |
| <code>lr_account</code>    | <code>&signer</code>    | The signer reference of the sending account of this transaction. Must be the Libra Root signer. |
| <code>hash</code>          | <code>vector&lt;u8&gt;</code> | The hash of the script to be added to the script allowlist.                                     |
| <code>sliding_nonce</code> | <code>u64</code>        | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                      |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                                           | Description                                                                                |
| ----------------           | --------------                                                         | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">CoreAddresses::ELIBRA_ROOT</a></code>                                           | The sending account is not the Libra Root account.                                         |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>                                         | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>                                         | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>                                | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EINVALID_SCRIPT_HASH">LibraTransactionPublishingOption::EINVALID_SCRIPT_HASH</a></code>               | The script <code>hash</code> is an invalid length.                                                    |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">LibraTransactionPublishingOption::EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a></code> | The on-chain allowlist already contains the script <code>hash</code>.                                 |


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, hash: vector&lt;u8&gt;, sliding_nonce: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, hash: vector&lt;u8&gt;, sliding_nonce: u64,) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(lr_account, sliding_nonce);
    <a href="../../modules/doc/LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_add_to_script_allow_list">LibraTransactionPublishingOption::add_to_script_allow_list</a>(lr_account, hash)
}
</code></pre>



</details>
