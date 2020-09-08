
<a name="SCRIPT"></a>

# Script `freeze_account.move`

### Table of Contents

-  [Function `freeze_account`](#SCRIPT_freeze_account)
    -  [Summary](#SCRIPT_@Summary)
    -  [Technical Description](#SCRIPT_@Technical_Description)
        -  [Events](#SCRIPT_@Events)
    -  [Parameters](#SCRIPT_@Parameters)
    -  [Common Abort Conditions](#SCRIPT_@Common_Abort_Conditions)
    -  [Related Scripts](#SCRIPT_@Related_Scripts)



<a name="SCRIPT_freeze_account"></a>

## Function `freeze_account`


<a name="SCRIPT_@Summary"></a>

### Summary

Freezes the account at <code>address</code>. The sending account of this transaction
must be the Treasury Compliance account. The account being frozen cannot be
the Libra Root or Treasury Compliance account. After the successful
execution of this transaction no transactions may be sent from the frozen
account, and the frozen account may not send or receive coins.


<a name="SCRIPT_@Technical_Description"></a>

### Technical Description

Sets the <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_FreezingBit">AccountFreezing::FreezingBit</a></code> to <code><b>true</b></code> and emits a
<code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_FreezeAccountEvent">AccountFreezing::FreezeAccountEvent</a></code>. The transaction sender must be the
Treasury Compliance account, but the account at <code>to_freeze_account</code> must
not be either <code>0xA550C18</code> (the Libra Root address), or <code>0xB1E55ED</code> (the
Treasury Compliance address). Note that this is a per-account property
e.g., freezing a Parent VASP will not effect the status any of its child
accounts and vice versa.


<a name="SCRIPT_@Events"></a>

#### Events

Successful execution of this transaction will emit a <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_FreezeAccountEvent">AccountFreezing::FreezeAccountEvent</a></code> on
the <code>freeze_event_handle</code> held in the <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">AccountFreezing::FreezeEventsHolder</a></code> resource published
under <code>0xA550C18</code> with the <code>frozen_address</code> being the <code>to_freeze_account</code>.


<a name="SCRIPT_@Parameters"></a>

### Parameters

| Name                | Type      | Description                                                                                               |
| ------              | ------    | -------------                                                                                             |
| <code>tc_account</code>        | <code>&signer</code> | The signer reference of the sending account of this transaction. Must be the Treasury Compliance account. |
| <code>sliding_nonce</code>     | <code>u64</code>     | The <code>sliding_nonce</code> (see: <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>) to be used for this transaction.                                |
| <code>to_freeze_account</code> | <code>address</code> | The account address to be frozen.                                                                         |


<a name="SCRIPT_@Common_Abort_Conditions"></a>

### Common Abort Conditions

| Error Category             | Error Reason                                 | Description                                                                                |
| ----------------           | --------------                               | -------------                                                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">SlidingNonce::ENONCE_TOO_OLD</a></code>               | The <code>sliding_nonce</code> is too old and it's impossible to determine if it's duplicated or not. |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">SlidingNonce::ENONCE_TOO_NEW</a></code>               | The <code>sliding_nonce</code> is too far in the future.                                              |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">SlidingNonce::ENONCE_ALREADY_RECORDED</a></code>      | The <code>sliding_nonce</code> has been previously recorded.                                          |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a></code> | <code><a href="../../modules/doc/CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">CoreAddresses::ETREASURY_COMPLIANCE</a></code>        | The sending account is not the Treasury Compliance account.                                |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_ECANNOT_FREEZE_TC">AccountFreezing::ECANNOT_FREEZE_TC</a></code>         | <code>to_freeze_account</code> was the Treasury Compliance account (<code>0xB1E55ED</code>).                     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_ECANNOT_FREEZE_LIBRA_ROOT">AccountFreezing::ECANNOT_FREEZE_LIBRA_ROOT</a></code> | <code>to_freeze_account</code> was the Libra Root account (<code>0xA550C18</code>).                              |


<a name="SCRIPT_@Related_Scripts"></a>

### Related Scripts

* <code>Scripts::unfreeze_account</code>


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_freeze_account">freeze_account</a>(tc_account: &signer, sliding_nonce: u64, to_freeze_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_freeze_account">freeze_account</a>(tc_account: &signer, sliding_nonce: u64, to_freeze_account: address) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(tc_account, sliding_nonce);
    <a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_freeze_account">AccountFreezing::freeze_account</a>(tc_account, to_freeze_account);
}
</code></pre>



</details>
