
<a name="SCRIPT"></a>

# Script `unfreeze_account.move`

### Table of Contents

-  [Function `unfreeze_account`](#SCRIPT_unfreeze_account)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
        -  [Events](#SCRIPT_@Events)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_unfreeze_account"></a>

## Function `unfreeze_account`


<a name="SCRIPT_@Summary"></a>

### Summary

Unfreezes the account at <code>address</code>. The sending account of this transaction must be the
Treasury Compliance account. After the successful execution of this transaction transactions
may be sent from the previously frozen account, and coins may be sent and received.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Sets the <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_FreezingBit">AccountFreezing::FreezingBit</a></code> to <code><b>false</b></code> and emits a
<code>AccountFreezing::UnFreezeAccountEvent</code>. The transaction sender must be the Treasury Compliance
account. Note that this is a per-account property so unfreezing a Parent VASP will not effect
the status any of its child accounts and vice versa.


<a name="SCRIPT_@Events"></a>

#### Events

Successful execution of this script will emit a <code>AccountFreezing::UnFreezeAccountEvent</code> with
the <code>unfrozen_address</code> set the <code>to_unfreeze_account</code>'s address.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name                  | Type      | Description                                                                                               |
| ------                | ------    | -------------                                                                                             |
| <code>tc_account</code>          | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account. |
| <code>sliding_nonce</code>       | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                |
| <code>to_unfreeze_account</code> | <code>address</code> | The account address to be frozen.                                                                         |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                            | Description                                                                                |
| ----------------           | --------------                          | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>          | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>          | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code> | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>   | The sending account is not the Treasury Compliance account.                                |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Scripts::freeze_account</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_unfreeze_account">unfreeze_account</a>(account: &signer, sliding_nonce: u64, to_unfreeze_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_unfreeze_account">unfreeze_account</a>(account: &signer, sliding_nonce: u64, to_unfreeze_account: address) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_unfreeze_account">AccountFreezing::unfreeze_account</a>(account, to_unfreeze_account);
}
</code></pre>



</details>
