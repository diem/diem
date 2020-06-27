
<a name="0x1_LibraAccount"></a>

# Module `0x1::LibraAccount`

### Table of Contents

-  [Resource `AccountFreezing`](#0x1_LibraAccount_AccountFreezing)
-  [Resource `AccountUnfreezing`](#0x1_LibraAccount_AccountUnfreezing)
-  [Resource `PublishModule`](#0x1_LibraAccount_PublishModule)
-  [Resource `LibraAccount`](#0x1_LibraAccount_LibraAccount)
-  [Resource `Balance`](#0x1_LibraAccount_Balance)
-  [Resource `WithdrawCapability`](#0x1_LibraAccount_WithdrawCapability)
-  [Resource `KeyRotationCapability`](#0x1_LibraAccount_KeyRotationCapability)
-  [Resource `AccountOperationsCapability`](#0x1_LibraAccount_AccountOperationsCapability)
-  [Struct `SentPaymentEvent`](#0x1_LibraAccount_SentPaymentEvent)
-  [Struct `ReceivedPaymentEvent`](#0x1_LibraAccount_ReceivedPaymentEvent)
-  [Struct `FreezingPrivilege`](#0x1_LibraAccount_FreezingPrivilege)
-  [Struct `FreezeAccountEvent`](#0x1_LibraAccount_FreezeAccountEvent)
-  [Struct `UnfreezeAccountEvent`](#0x1_LibraAccount_UnfreezeAccountEvent)
-  [Function `grant_association_privileges`](#0x1_LibraAccount_grant_association_privileges)
-  [Function `grant_treasury_compliance_privileges`](#0x1_LibraAccount_grant_treasury_compliance_privileges)
-  [Function `initialize`](#0x1_LibraAccount_initialize)
-  [Function `staple_lbr`](#0x1_LibraAccount_staple_lbr)
-  [Function `unstaple_lbr`](#0x1_LibraAccount_unstaple_lbr)
-  [Function `deposit`](#0x1_LibraAccount_deposit)
-  [Function `tiered_mint`](#0x1_LibraAccount_tiered_mint)
-  [Function `cancel_burn`](#0x1_LibraAccount_cancel_burn)
-  [Function `withdraw_from_balance`](#0x1_LibraAccount_withdraw_from_balance)
-  [Function `withdraw_from`](#0x1_LibraAccount_withdraw_from)
-  [Function `preburn`](#0x1_LibraAccount_preburn)
-  [Function `extract_withdraw_capability`](#0x1_LibraAccount_extract_withdraw_capability)
-  [Function `restore_withdraw_capability`](#0x1_LibraAccount_restore_withdraw_capability)
-  [Function `pay_from`](#0x1_LibraAccount_pay_from)
-  [Function `rotate_authentication_key`](#0x1_LibraAccount_rotate_authentication_key)
-  [Function `extract_key_rotation_capability`](#0x1_LibraAccount_extract_key_rotation_capability)
-  [Function `restore_key_rotation_capability`](#0x1_LibraAccount_restore_key_rotation_capability)
-  [Function `add_currencies_for_account`](#0x1_LibraAccount_add_currencies_for_account)
-  [Function `make_account`](#0x1_LibraAccount_make_account)
-  [Function `create_root_association_account`](#0x1_LibraAccount_create_root_association_account)
-  [Function `create_treasury_compliance_account`](#0x1_LibraAccount_create_treasury_compliance_account)
-  [Function `create_designated_dealer`](#0x1_LibraAccount_create_designated_dealer)
-  [Function `create_parent_vasp_account`](#0x1_LibraAccount_create_parent_vasp_account)
-  [Function `create_child_vasp_account`](#0x1_LibraAccount_create_child_vasp_account)
-  [Function `create_unhosted_account`](#0x1_LibraAccount_create_unhosted_account)
-  [Function `create_signer`](#0x1_LibraAccount_create_signer)
-  [Function `destroy_signer`](#0x1_LibraAccount_destroy_signer)
-  [Function `balance_for`](#0x1_LibraAccount_balance_for)
-  [Function `balance`](#0x1_LibraAccount_balance)
-  [Function `add_currency`](#0x1_LibraAccount_add_currency)
-  [Function `accepts_currency`](#0x1_LibraAccount_accepts_currency)
-  [Function `sequence_number_for_account`](#0x1_LibraAccount_sequence_number_for_account)
-  [Function `sequence_number`](#0x1_LibraAccount_sequence_number)
-  [Function `authentication_key`](#0x1_LibraAccount_authentication_key)
-  [Function `delegated_key_rotation_capability`](#0x1_LibraAccount_delegated_key_rotation_capability)
-  [Function `delegated_withdraw_capability`](#0x1_LibraAccount_delegated_withdraw_capability)
-  [Function `withdraw_capability_address`](#0x1_LibraAccount_withdraw_capability_address)
-  [Function `key_rotation_capability_address`](#0x1_LibraAccount_key_rotation_capability_address)
-  [Function `exists_at`](#0x1_LibraAccount_exists_at)
-  [Function `freeze_account`](#0x1_LibraAccount_freeze_account)
-  [Function `unfreeze_account`](#0x1_LibraAccount_unfreeze_account)
-  [Function `account_is_frozen`](#0x1_LibraAccount_account_is_frozen)
-  [Function `prologue`](#0x1_LibraAccount_prologue)
-  [Function `epilogue`](#0x1_LibraAccount_epilogue)
-  [Function `success_epilogue`](#0x1_LibraAccount_success_epilogue)
-  [Function `failure_epilogue`](#0x1_LibraAccount_failure_epilogue)
-  [Function `bump_sequence_number`](#0x1_LibraAccount_bump_sequence_number)
-  [Function `create_validator_account`](#0x1_LibraAccount_create_validator_account)
-  [Function `create_validator_operator_account`](#0x1_LibraAccount_create_validator_operator_account)
-  [Specification](#0x1_LibraAccount_Specification)



<a name="0x1_LibraAccount_AccountFreezing"></a>

## Resource `AccountFreezing`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_AccountFreezing">AccountFreezing</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_AccountUnfreezing"></a>

## Resource `AccountUnfreezing`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_AccountUnfreezing">AccountUnfreezing</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_PublishModule"></a>

## Resource `PublishModule`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_PublishModule">PublishModule</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_LibraAccount"></a>

## Resource `LibraAccount`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount">LibraAccount</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>authentication_key: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>withdrawal_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>key_rotation_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>received_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>sent_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>is_frozen: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_Balance"></a>

## Resource `Balance`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>coin: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_WithdrawCapability"></a>

## Resource `WithdrawCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>account_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_KeyRotationCapability"></a>

## Resource `KeyRotationCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>account_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_AccountOperationsCapability"></a>

## Resource `AccountOperationsCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>limits_cap: <a href="AccountLimits.md#0x1_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a></code>
</dt>
<dd>

</dd>
<dt>

<code>freeze_event_handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraAccount_FreezeAccountEvent">LibraAccount::FreezeAccountEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>unfreeze_event_handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraAccount_UnfreezeAccountEvent">LibraAccount::UnfreezeAccountEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_SentPaymentEvent"></a>

## Struct `SentPaymentEvent`



<pre><code><b>struct</b> <a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>payee: address</code>
</dt>
<dd>

</dd>
<dt>

<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_ReceivedPaymentEvent"></a>

## Struct `ReceivedPaymentEvent`



<pre><code><b>struct</b> <a href="#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>payer: address</code>
</dt>
<dd>

</dd>
<dt>

<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_FreezingPrivilege"></a>

## Struct `FreezingPrivilege`



<pre><code><b>struct</b> <a href="#0x1_LibraAccount_FreezingPrivilege">FreezingPrivilege</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_FreezeAccountEvent"></a>

## Struct `FreezeAccountEvent`



<pre><code><b>struct</b> <a href="#0x1_LibraAccount_FreezeAccountEvent">FreezeAccountEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>initiator_address: address</code>
</dt>
<dd>

</dd>
<dt>

<code>frozen_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_UnfreezeAccountEvent"></a>

## Struct `UnfreezeAccountEvent`



<pre><code><b>struct</b> <a href="#0x1_LibraAccount_UnfreezeAccountEvent">UnfreezeAccountEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>initiator_address: address</code>
</dt>
<dd>

</dd>
<dt>

<code>unfrozen_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_grant_association_privileges"></a>

## Function `grant_association_privileges`

Grants
<code><a href="#0x1_LibraAccount_AccountFreezing">AccountFreezing</a></code> and
<code><a href="#0x1_LibraAccount_AccountUnfreezing">AccountUnfreezing</a></code> privileges to the calling
<code>account</code>.
Aborts if the
<code>account</code> does not have the correct role (association root).


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_grant_association_privileges">grant_association_privileges</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_grant_association_privileges">grant_association_privileges</a>(account: &signer) {
    // TODO: Need <b>to</b> also grant this <b>to</b> the core code address account.
    <a href="Roles.md#0x1_Roles_add_privilege_to_account_association_root_role">Roles::add_privilege_to_account_association_root_role</a>(account, <a href="#0x1_LibraAccount_PublishModule">PublishModule</a>{});
}
</code></pre>



</details>

<a name="0x1_LibraAccount_grant_treasury_compliance_privileges"></a>

## Function `grant_treasury_compliance_privileges`

Grants
<code><a href="#0x1_LibraAccount_AccountFreezing">AccountFreezing</a></code> and
<code><a href="#0x1_LibraAccount_AccountUnfreezing">AccountUnfreezing</a></code> privileges to the calling
<code>account</code>.
Aborts if the
<code>account</code> does not have the correct role (treasury compliance).


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_grant_treasury_compliance_privileges">grant_treasury_compliance_privileges</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_grant_treasury_compliance_privileges">grant_treasury_compliance_privileges</a>(account: &signer) {
    <a href="Roles.md#0x1_Roles_add_privilege_to_account_treasury_compliance_role">Roles::add_privilege_to_account_treasury_compliance_role</a>(account, <a href="#0x1_LibraAccount_AccountFreezing">AccountFreezing</a>{});
    <a href="Roles.md#0x1_Roles_add_privilege_to_account_treasury_compliance_role">Roles::add_privilege_to_account_treasury_compliance_role</a>(account, <a href="#0x1_LibraAccount_AccountUnfreezing">AccountUnfreezing</a>{});
}
</code></pre>



</details>

<a name="0x1_LibraAccount_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_initialize">initialize</a>(association: &signer, assoc_root_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_LibraRootRole">Roles::LibraRootRole</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_initialize">initialize</a>(
    association: &signer,
    assoc_root_capability: &Capability&lt;LibraRootRole&gt;,
) {
    // Operational constraint, not a privilege constraint.
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(association) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), 0);
    move_to(
        association,
        <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
            limits_cap: <a href="AccountLimits.md#0x1_AccountLimits_grant_calling_capability">AccountLimits::grant_calling_capability</a>(assoc_root_capability),
            freeze_event_handle: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>(association),
            unfreeze_event_handle: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>(association),
        }
    );
}
</code></pre>



</details>

<a name="0x1_LibraAccount_staple_lbr"></a>

## Function `staple_lbr`

Use
<code>cap</code> to mint
<code>amount_lbr</code> LBR by withdrawing the appropriate quantity of reserve assets
from
<code>cap.address</code>, giving them to the LBR reserve, and depositing the LBR into
<code>cap.address</code>.
The
<code>payee</code> address in the
<code><a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a></code>s emitted by this functipn be the LBR reserve
address to signify that this was a special payment that debits the
<code>cap.addr</code>'s balance and
crebits the LBR reserve.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_staple_lbr">staple_lbr</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_staple_lbr">staple_lbr</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>, amount_lbr: u64)
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> cap_address = cap.account_address;
    // withdraw all <a href="Coin1.md#0x1_Coin1">Coin1</a> and <a href="Coin2.md#0x1_Coin2">Coin2</a>
    <b>let</b> coin1_balance = <a href="#0x1_LibraAccount_balance">balance</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(cap_address);
    <b>let</b> coin2_balance = <a href="#0x1_LibraAccount_balance">balance</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(cap_address);
    // <b>use</b> the <a href="LBR.md#0x1_LBR">LBR</a> reserve address <b>as</b> `payee_address`
    <b>let</b> payee_address = <a href="LBR.md#0x1_LBR_reserve_address">LBR::reserve_address</a>();
    <b>let</b> coin1 = <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(cap, payee_address, coin1_balance, x"");
    <b>let</b> coin2 = <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(cap, payee_address, coin2_balance, x"");
    // Create `amount_lbr` <a href="LBR.md#0x1_LBR">LBR</a>
    <b>let</b> (lbr, coin1, coin2) = <a href="LBR.md#0x1_LBR_create">LBR::create</a>(amount_lbr, coin1, coin2);
    // <b>use</b> the reserved address <b>as</b> the payer for the <a href="LBR.md#0x1_LBR">LBR</a> payment because the funds did not come
    // from an existing balance
    <a href="#0x1_LibraAccount_deposit">deposit</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), cap_address, lbr, x"", x"");
    // TODO: eliminate these self-deposits by withdrawing appropriate amounts up-front
    // Deposit the <a href="Coin1.md#0x1_Coin1">Coin1</a>/<a href="Coin2.md#0x1_Coin2">Coin2</a> remainders
    <a href="#0x1_LibraAccount_deposit">deposit</a>(cap_address, cap_address, coin1, x"", x"");
    <a href="#0x1_LibraAccount_deposit">deposit</a>(cap_address, cap_address, coin2, x"", x"")
}
</code></pre>



</details>

<a name="0x1_LibraAccount_unstaple_lbr"></a>

## Function `unstaple_lbr`

Use
<code>cap</code> to withdraw
<code>amount_lbr</code>, burn the LBR, withdraw the corresponding assets from the
LBR reserve, and deposit them to
<code>cap.address</code>.
The
<code>payer</code> address in the
<code> RecievedPaymentEvent</code>s emitted by this function will be the LBR
reserve address to signify that this was a special payment that credits
<code>cap.address</code>'s balance and credits the LBR reserve.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_unstaple_lbr">unstaple_lbr</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_unstaple_lbr">unstaple_lbr</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>, amount_lbr: u64)
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    // <b>use</b> the reserved address <b>as</b> the payee because the funds will be burned
    <b>let</b> lbr = <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(cap, <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), amount_lbr, x"");
    <b>let</b> (coin1, coin2) = <a href="LBR.md#0x1_LBR_unpack">LBR::unpack</a>(lbr);
    // These funds come from the <a href="LBR.md#0x1_LBR">LBR</a> reserve, so <b>use</b> the <a href="LBR.md#0x1_LBR">LBR</a> reserve address <b>as</b> the payer
    <b>let</b> payer_address = <a href="LBR.md#0x1_LBR_reserve_address">LBR::reserve_address</a>();
    <b>let</b> payee_address = cap.account_address;
    <a href="#0x1_LibraAccount_deposit">deposit</a>(payer_address, payee_address, coin1, x"", x"");
    <a href="#0x1_LibraAccount_deposit">deposit</a>(payer_address, payee_address, coin2, x"", x"")
}
</code></pre>



</details>

<a name="0x1_LibraAccount_deposit"></a>

## Function `deposit`

Record a payment of
<code>to_deposit</code> from
<code>payer</code> to
<code>payee</code> with the attached
<code>metadata</code>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_deposit">deposit</a>&lt;Token&gt;(payer: address, payee: address, to_deposit: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_deposit">deposit</a>&lt;Token&gt;(
    payer: address,
    payee: address,
    to_deposit: <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt;,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    // Check that the `to_deposit` coin is non-zero
    <b>let</b> deposit_value = <a href="Libra.md#0x1_Libra_value">Libra::value</a>(&to_deposit);
    <b>assert</b>(deposit_value &gt; 0, 7);
    <b>let</b> travel_rule_limit_microlibra = <a href="DualAttestationLimit.md#0x1_DualAttestationLimit_get_cur_microlibra_limit">DualAttestationLimit::get_cur_microlibra_limit</a>();
    // travel rule only applies for payments over a threshold
    <b>let</b> approx_lbr_microlibra_value = <a href="Libra.md#0x1_Libra_approx_lbr_for_value">Libra::approx_lbr_for_value</a>&lt;Token&gt;(deposit_value);
    <b>let</b> above_threshold = approx_lbr_microlibra_value &gt;= travel_rule_limit_microlibra;
    // travel rule only applies <b>if</b> the sender and recipient are both VASPs
    <b>let</b> both_vasps = <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payer) && <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payee);
    <b>if</b> (above_threshold &&
        both_vasps &&
        // travel rule does not <b>apply</b> for intra-<a href="VASP.md#0x1_VASP">VASP</a> transactions
        <a href="VASP.md#0x1_VASP_parent_address">VASP::parent_address</a>(payer) != <a href="VASP.md#0x1_VASP_parent_address">VASP::parent_address</a>(payee)
    ) {
        // sanity check of signature validity
        <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&metadata_signature) == 64, 9001);
        // message should be metadata | payer | amount | domain_separator
        <b>let</b> domain_separator = b"@@$$LIBRA_ATTEST$$@@";
        <b>let</b> message = <b>copy</b> metadata;
        <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> message, <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&payer));
        <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> message, <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&deposit_value));
        <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> message, domain_separator);
        // cryptographic check of signature validity
        <b>assert</b>(
            <a href="Signature.md#0x1_Signature_ed25519_verify">Signature::ed25519_verify</a>(
                metadata_signature,
                <a href="VASP.md#0x1_VASP_compliance_public_key">VASP::compliance_public_key</a>(payee),
                message
            ),
            9002, // TODO: proper error code
        );
    };

    // Ensure that this deposit is compliant with the account limits on
    // this account.
    <b>let</b> _ = borrow_global&lt;<a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    /*<b>assert</b>(
        <a href="AccountLimits.md#0x1_AccountLimits_update_deposit_limits">AccountLimits::update_deposit_limits</a>&lt;Token&gt;(
            deposit_value,
            payee,
            &borrow_global&lt;<a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).limits_cap
        ),
        9
    );*/

    // Deposit the `to_deposit` coin
    <a href="Libra.md#0x1_Libra_deposit">Libra::deposit</a>(&<b>mut</b> borrow_global_mut&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee).coin, to_deposit);
    // Log a received event
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(payee).received_events,
        <a href="#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a> {
            amount: deposit_value,
            currency_code: <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;Token&gt;(),
            payer,
            metadata: metadata
        }
    );
}
</code></pre>



</details>

<a name="0x1_LibraAccount_tiered_mint"></a>

## Function `tiered_mint`

Mint 'mint_amount' to 'designated_dealer_address' for 'tier_index' tier.
Max valid tier index is 3 since there are max 4 tiers per DD.
Sender should be treasury compliance account and receiver authorized DD.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_tiered_mint">tiered_mint</a>&lt;Token&gt;(tc_account: &signer, tc_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;, designated_dealer_address: address, mint_amount: u64, tier_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_tiered_mint">tiered_mint</a>&lt;Token&gt;(
    tc_account: &signer,
    tc_capability: &Capability&lt;TreasuryComplianceRole&gt;,
    designated_dealer_address: address,
    mint_amount: u64,
    tier_index: u64,
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> coin = <a href="DesignatedDealer.md#0x1_DesignatedDealer_tiered_mint">DesignatedDealer::tiered_mint</a>&lt;Token&gt;(
        tc_account, tc_capability, mint_amount, designated_dealer_address, tier_index
    );
    // <b>use</b> the reserved address <b>as</b> the payer because the funds did not come from an existing
    // balance
    <a href="#0x1_LibraAccount_deposit">deposit</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), designated_dealer_address, coin, x"", x"")
}
</code></pre>



</details>

<a name="0x1_LibraAccount_cancel_burn"></a>

## Function `cancel_burn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_cancel_burn">cancel_burn</a>&lt;Token&gt;(account: &signer, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_cancel_burn">cancel_burn</a>&lt;Token&gt;(
    account: &signer,
    preburn_address: address,
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> coin = <a href="Libra.md#0x1_Libra_cancel_burn">Libra::cancel_burn</a>&lt;Token&gt;(account, preburn_address);
    // record both sender and recipient <b>as</b> `preburn_address`: the coins are moving from
    // `preburn_address`'s `Preburn` <b>resource</b> <b>to</b> its balance
    <a href="#0x1_LibraAccount_deposit">deposit</a>(preburn_address, preburn_address, coin, x"", x"")
}
</code></pre>



</details>

<a name="0x1_LibraAccount_withdraw_from_balance"></a>

## Function `withdraw_from_balance`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(_addr: address, balance: &<b>mut</b> <a href="#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;, amount: u64): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(
    _addr: address,
    balance: &<b>mut</b> <a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;,
    amount: u64
): <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt; <b>acquires</b> <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    // Make sure that this withdrawal is compliant with the limits on
    // the account.
    <b>let</b> _  = borrow_global&lt;<a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    /*<b>let</b> can_withdraw = <a href="AccountLimits.md#0x1_AccountLimits_update_withdrawal_limits">AccountLimits::update_withdrawal_limits</a>&lt;Token&gt;(
        amount,
        addr,
        &borrow_global&lt;<a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).limits_cap
    );
    <b>assert</b>(can_withdraw, 11);*/
    <a href="Libra.md#0x1_Libra_withdraw">Libra::withdraw</a>(&<b>mut</b> balance.coin, amount)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_withdraw_from"></a>

## Function `withdraw_from`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;Token&gt;(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, payee: address, amount: u64, metadata: vector&lt;u8&gt;): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;Token&gt;(
    cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>,
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
): <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt; <b>acquires</b> <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> payer = cap.account_address;
    <b>let</b> account_balance = borrow_global_mut&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer);
    // Load the payer's account and emit an event <b>to</b> record the withdrawal
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(payer).sent_events,
        <a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a> {
            amount,
            currency_code: <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;Token&gt;(),
            payee,
            metadata
        },
    );
    <a href="#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(payer, account_balance, amount)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_preburn"></a>

## Function `preburn`

Withdraw
<code>amount</code>
<code><a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt;</code>'s from
<code>cap.address</code> and send them to the
<code>Preburn</code>
resource under
<code>dd</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_preburn">preburn</a>&lt;Token&gt;(dd: &signer, cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_preburn">preburn</a>&lt;Token&gt;(
    dd: &signer, cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>, amount: u64
) <b>acquires</b> <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x1_LibraAccount">LibraAccount</a> {
    <a href="Libra.md#0x1_Libra_preburn_to">Libra::preburn_to</a>&lt;Token&gt;(dd, <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>(cap, <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(dd), amount, x""))
}
</code></pre>



</details>

<a name="0x1_LibraAccount_extract_withdraw_capability"></a>

## Function `extract_withdraw_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_extract_withdraw_capability">extract_withdraw_capability</a>(sender: &signer): <a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_extract_withdraw_capability">extract_withdraw_capability</a>(
    sender: &signer
): <a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a> <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
    // Abort <b>if</b> we already extracted the unique withdraw capability for this account.
    <b>assert</b>(!<a href="#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr), 11);
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(sender_addr);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_restore_withdraw_capability"></a>

## Function `restore_withdraw_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>)
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.withdrawal_capability, cap)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_pay_from"></a>

## Function `pay_from`

Withdraw
<code>amount</code> Libra<Token> from the address embedded in
<code><a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a></code> and
deposits it into the
<code>payee</code>'s account balance.
The included
<code>metadata</code> will appear in the
<code><a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a></code> and
<code><a href="#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a></code>.
The
<code>metadata_signature</code> will only be checked if this payment is subject to the dual
attestation protocol


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_pay_from">pay_from</a>&lt;Token&gt;(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_pay_from">pay_from</a>&lt;Token&gt;(
    cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>,
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="#0x1_LibraAccount_deposit">deposit</a>&lt;Token&gt;(
        *&cap.account_address,
        payee,
        <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>(cap, payee, amount, <b>copy</b> metadata),
        metadata,
        metadata_signature
    );
}
</code></pre>



</details>

<a name="0x1_LibraAccount_rotate_authentication_key"></a>

## Function `rotate_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_rotate_authentication_key">rotate_authentication_key</a>(cap: &<a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_rotate_authentication_key">rotate_authentication_key</a>(
    cap: &<a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>,
    new_authentication_key: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>  {
    <b>let</b> sender_account_resource = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
    // Don't allow rotating <b>to</b> clearly invalid key
    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_authentication_key) == 32, 12);
    sender_account_resource.authentication_key = new_authentication_key;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_extract_key_rotation_capability"></a>

## Function `extract_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Abort <b>if</b> we already extracted the unique key rotation capability for this account.
    <b>assert</b>(!<a href="#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(account_address), 11);
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(account_address);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_restore_key_rotation_capability"></a>

## Function `restore_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>)
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.key_rotation_capability, cap)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_add_currencies_for_account"></a>

## Function `add_currencies_for_account`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(new_account: &signer, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(
    new_account: &signer,
    add_all_currencies: bool,
) {
    <b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account);
    <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(new_account);
    <b>if</b> (add_all_currencies) {
        <b>if</b> (!exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(new_account_addr)) {
            <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(new_account);
        };
        <b>if</b> (!exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(new_account_addr)) {
            <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(new_account);
        };
        <b>if</b> (!exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(new_account_addr)) {
            <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(new_account);
        };
    };
}
</code></pre>



</details>

<a name="0x1_LibraAccount_make_account"></a>

## Function `make_account`

Creates a new account with account at
<code>new_account_address</code> with a balance of
zero in
<code>Token</code> and authentication key
<code>auth_key_prefix</code> |
<code>fresh_address</code>. If
<code>add_all_currencies</code> is true, 0 balances for all available currencies in the system will
also be added.
Aborts if there is already an account at
<code>new_account_address</code>.
Creating an account at address 0x0 will abort as it is a reserved address for the MoveVM.


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account: signer, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_make_account">make_account</a>(
    new_account: signer,
    auth_key_prefix: vector&lt;u8&gt;,
) {
    <b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&new_account);
    // cannot create an account at the reserved address 0x0
    <b>assert</b>(new_account_addr != <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), 0);

    // (1) publish <a href="#0x1_LibraAccount">LibraAccount</a>
    <b>let</b> authentication_key = auth_key_prefix;
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(
        &<b>mut</b> authentication_key, <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(<a href="Signer.md#0x1_Signer_borrow_address">Signer::borrow_address</a>(&new_account))
    );
    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&authentication_key) == 32, 12);
    move_to(
        &new_account,
        <a href="#0x1_LibraAccount">LibraAccount</a> {
            authentication_key,
            withdrawal_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
                <a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a> {
                    account_address: new_account_addr
            }),
            key_rotation_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
                <a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a> {
                    account_address: new_account_addr
            }),
            received_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(&new_account),
            sent_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>&gt;(&new_account),
            sequence_number: 0,
            is_frozen: <b>false</b>,
        }
    );

    // (2) TODO: publish account limits?
    <a href="#0x1_LibraAccount_destroy_signer">destroy_signer</a>(new_account);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_root_association_account"></a>

## Function `create_root_association_account`

Creates the root association account in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_root_association_account">create_root_association_account</a>(new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_root_association_account">create_root_association_account</a>(
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_is_genesis">LibraTimestamp::assert_is_genesis</a>();
    <b>assert</b>(new_account_address == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), 0);
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_treasury_compliance_account"></a>

## Function `create_treasury_compliance_account`

Create a treasury/compliance account at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_treasury_compliance_account">create_treasury_compliance_account</a>(_: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_LibraRootRole">Roles::LibraRootRole</a>&gt;, tc_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;, sliding_nonce_creation_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce_CreateSlidingNonce">SlidingNonce::CreateSlidingNonce</a>&gt;, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, coin1_mint_cap: <a href="Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, coin1_burn_cap: <a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="Coin1.md#0x1_Coin1_Coin1">Coin1::Coin1</a>&gt;, coin2_mint_cap: <a href="Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;, coin2_burn_cap: <a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="Coin2.md#0x1_Coin2_Coin2">Coin2::Coin2</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_treasury_compliance_account">create_treasury_compliance_account</a>(
    _: &Capability&lt;LibraRootRole&gt;,
    tc_capability: &Capability&lt;TreasuryComplianceRole&gt;,
    sliding_nonce_creation_capability: &Capability&lt;CreateSlidingNonce&gt;,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    coin1_mint_cap: <a href="Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;,
    coin1_burn_cap: <a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;,
    coin2_mint_cap: <a href="Libra.md#0x1_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;,
    coin2_burn_cap: <a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_is_genesis">LibraTimestamp::assert_is_genesis</a>();
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Libra.md#0x1_Libra_publish_mint_capability">Libra::publish_mint_capability</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(&new_account, coin1_mint_cap, tc_capability);
    <a href="Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(&new_account, coin1_burn_cap, tc_capability);
    <a href="Libra.md#0x1_Libra_publish_mint_capability">Libra::publish_mint_capability</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(&new_account, coin2_mint_cap, tc_capability);
    <a href="Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(&new_account, coin2_burn_cap, tc_capability);
    <a href="SlidingNonce.md#0x1_SlidingNonce_publish_nonce_resource">SlidingNonce::publish_nonce_resource</a>(sliding_nonce_creation_capability, &new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_designated_dealer"></a>

## Function `create_designated_dealer`

Create a designated dealer account at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code>, for non synthetic CoinType.
Creates Preburn resource under account 'new_account_address'


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(creator_account: &signer, tc_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_TreasuryComplianceRole">Roles::TreasuryComplianceRole</a>&gt;, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(
    creator_account: &signer,
    tc_capability: &Capability&lt;TreasuryComplianceRole&gt;,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
) {
    <b>let</b> new_dd_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_dd_account);
    <a href="Libra.md#0x1_Libra_publish_preburn_to_account">Libra::publish_preburn_to_account</a>&lt;CoinType&gt;(&new_dd_account, tc_capability);
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_publish_designated_dealer_credential">DesignatedDealer::publish_designated_dealer_credential</a>(&new_dd_account, tc_capability);
    <a href="Roles.md#0x1_Roles_new_designated_dealer_role">Roles::new_designated_dealer_role</a>(creator_account, &new_dd_account);
    <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;CoinType&gt;(&new_dd_account, <b>false</b>);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_dd_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_parent_vasp_account"></a>

## Function `create_parent_vasp_account`

Create an account with the ParentVASP role at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code>.  If
<code>add_all_currencies</code> is true, 0 balances for
all available currencies in the system will also be added.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_parent_vasp_account">create_parent_vasp_account</a>&lt;Token&gt;(creator_account: &signer, parent_vasp_creation_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_LibraRootRole">Roles::LibraRootRole</a>&gt;, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_parent_vasp_account">create_parent_vasp_account</a>&lt;Token&gt;(
    creator_account: &signer,
    parent_vasp_creation_capability: &Capability&lt;LibraRootRole&gt;,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;,
    add_all_currencies: bool
) {
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Roles.md#0x1_Roles_new_parent_vasp_role">Roles::new_parent_vasp_role</a>(creator_account, &new_account);
    <a href="VASP.md#0x1_VASP_publish_parent_vasp_credential">VASP::publish_parent_vasp_credential</a>(
        &new_account,
        parent_vasp_creation_capability,
        human_name,
        base_url,
        compliance_public_key
    );
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(&new_account, add_all_currencies);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_child_vasp_account"></a>

## Function `create_child_vasp_account`

Create an account with the ChildVASP role at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code> and a 0 balance of type
<code>Token</code>. If
<code>add_all_currencies</code> is true, 0 balances for all avaialable currencies in the system will
also be added. This account will be a child of
<code>creator</code>, which must be a ParentVASP.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_child_vasp_account">create_child_vasp_account</a>&lt;Token&gt;(parent: &signer, child_vasp_creation_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_ParentVASPRole">Roles::ParentVASPRole</a>&gt;, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_child_vasp_account">create_child_vasp_account</a>&lt;Token&gt;(
    parent: &signer,
    child_vasp_creation_capability: &Capability&lt;ParentVASPRole&gt;,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool,
) {
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Roles.md#0x1_Roles_new_child_vasp_role">Roles::new_child_vasp_role</a>(parent, &new_account);
    <a href="VASP.md#0x1_VASP_publish_child_vasp_credential">VASP::publish_child_vasp_credential</a>(
        parent,
        &new_account,
        child_vasp_creation_capability,
    );
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(&new_account, add_all_currencies);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_unhosted_account"></a>

## Function `create_unhosted_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_unhosted_account">create_unhosted_account</a>&lt;Token&gt;(creator_account: &signer, _: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_LibraRootRole">Roles::LibraRootRole</a>&gt;, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_unhosted_account">create_unhosted_account</a>&lt;Token&gt;(
    creator_account: &signer,
    _: &Capability&lt;LibraRootRole&gt;,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool
) {
    <b>assert</b>(!<a href="#0x1_LibraAccount_exists_at">exists_at</a>(new_account_address), 777777);
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Roles.md#0x1_Roles_new_unhosted_role">Roles::new_unhosted_role</a>(creator_account, &new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(&new_account, add_all_currencies);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_signer"></a>

## Function `create_signer`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_create_signer">create_signer</a>(addr: address): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x1_LibraAccount_create_signer">create_signer</a>(addr: address): signer;
</code></pre>



</details>

<a name="0x1_LibraAccount_destroy_signer"></a>

## Function `destroy_signer`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_destroy_signer">destroy_signer</a>(sig: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x1_LibraAccount_destroy_signer">destroy_signer</a>(sig: signer);
</code></pre>



</details>

<a name="0x1_LibraAccount_balance_for"></a>

## Function `balance_for`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_balance_for">balance_for</a>&lt;Token&gt;(balance: &<a href="#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_balance_for">balance_for</a>&lt;Token&gt;(balance: &<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;): u64 {
    <a href="Libra.md#0x1_Libra_value">Libra::value</a>&lt;Token&gt;(&balance.coin)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_balance"></a>

## Function `balance`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(addr: address): u64 <b>acquires</b> <a href="#0x1_LibraAccount_Balance">Balance</a> {
    <a href="#0x1_LibraAccount_balance_for">balance_for</a>(borrow_global&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_LibraAccount_add_currency"></a>

## Function `add_currency`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(account: &signer) {
    move_to(account, <a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;{ coin: <a href="Libra.md#0x1_Libra_zero">Libra::zero</a>&lt;Token&gt;() })
}
</code></pre>



</details>

<a name="0x1_LibraAccount_accepts_currency"></a>

## Function `accepts_currency`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_accepts_currency">accepts_currency</a>&lt;Token&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_accepts_currency">accepts_currency</a>&lt;Token&gt;(addr: address): bool {
    exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_sequence_number_for_account"></a>

## Function `sequence_number_for_account`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="#0x1_LibraAccount">LibraAccount</a>): u64 {
    account.sequence_number
}
</code></pre>



</details>

<a name="0x1_LibraAccount_sequence_number"></a>

## Function `sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_sequence_number">sequence_number</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_sequence_number">sequence_number</a>(addr: address): u64 <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <a href="#0x1_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(borrow_global&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_LibraAccount_authentication_key"></a>

## Function `authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    *&borrow_global&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).authentication_key
}
</code></pre>



</details>

<a name="0x1_LibraAccount_delegated_key_rotation_capability"></a>

## Function `delegated_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&borrow_global&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_delegated_withdraw_capability"></a>

## Function `delegated_withdraw_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&borrow_global&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_withdraw_capability_address"></a>

## Function `withdraw_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_LibraAccount_key_rotation_capability_address"></a>

## Function `key_rotation_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_LibraAccount_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_exists_at">exists_at</a>(check_addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_exists_at">exists_at</a>(check_addr: address): bool {
    exists&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(check_addr)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_freeze_account"></a>

## Function `freeze_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_freeze_account">freeze_account</a>(account: &signer, _freezing_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="#0x1_LibraAccount_AccountFreezing">LibraAccount::AccountFreezing</a>&gt;, frozen_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_freeze_account">freeze_account</a>(
    account: &signer,
    _freezing_capability: &Capability&lt;<a href="#0x1_LibraAccount_AccountFreezing">AccountFreezing</a>&gt;,
    frozen_address: address,
)
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> initiator_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // The root association account cannot be frozen
    <b>assert</b>(frozen_address != <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), 14);
    borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(frozen_address).is_frozen = <b>true</b>;
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraAccount_FreezeAccountEvent">FreezeAccountEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).freeze_event_handle,
        <a href="#0x1_LibraAccount_FreezeAccountEvent">FreezeAccountEvent</a> {
            initiator_address,
            frozen_address
        },
    );
}
</code></pre>



</details>

<a name="0x1_LibraAccount_unfreeze_account"></a>

## Function `unfreeze_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_unfreeze_account">unfreeze_account</a>(account: &signer, _unfreezing_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="#0x1_LibraAccount_AccountUnfreezing">LibraAccount::AccountUnfreezing</a>&gt;, unfrozen_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_unfreeze_account">unfreeze_account</a>(
    account: &signer,
    _unfreezing_capability: &Capability&lt;<a href="#0x1_LibraAccount_AccountUnfreezing">AccountUnfreezing</a>&gt;,
    unfrozen_address: address,
)
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> initiator_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(unfrozen_address).is_frozen = <b>false</b>;
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraAccount_UnfreezeAccountEvent">UnfreezeAccountEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).unfreeze_event_handle,
        <a href="#0x1_LibraAccount_UnfreezeAccountEvent">UnfreezeAccountEvent</a> {
            initiator_address,
            unfrozen_address
        },
    );
}
</code></pre>



</details>

<a name="0x1_LibraAccount_account_is_frozen"></a>

## Function `account_is_frozen`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_account_is_frozen">account_is_frozen</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_account_is_frozen">account_is_frozen</a>(addr: address): bool
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    borrow_global&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).is_frozen
 }
</code></pre>



</details>

<a name="0x1_LibraAccount_prologue"></a>

## Function `prologue`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_prologue">prologue</a>&lt;Token&gt;(sender: &signer, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_prologue">prologue</a>&lt;Token&gt;(
    sender: &signer,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time: u64,
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a> {
    <b>let</b> transaction_sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);

    // FUTURE: Make these error codes sequential
    // Verify that the transaction sender's account exists
    <b>assert</b>(<a href="#0x1_LibraAccount_exists_at">exists_at</a>(transaction_sender), 5);

    <b>assert</b>(!<a href="#0x1_LibraAccount_account_is_frozen">account_is_frozen</a>(transaction_sender), 0);

    // Load the transaction sender's account
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(transaction_sender);

    // Check that the hash of the transaction's <b>public</b> key matches the account's auth key
    <b>assert</b>(
        <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) == *&sender_account.authentication_key,
        2
    );

    // Check that the account has enough balance for all of the gas
    <b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
    // Don't grab the balance <b>if</b> the transaction fee is zero
    <b>if</b> (max_transaction_fee &gt; 0) {
        <b>let</b> balance_amount = <a href="#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(transaction_sender);
        <b>assert</b>(balance_amount &gt;= max_transaction_fee, 6);
    };

    // Check that the transaction sequence number matches the sequence number of the account
    <b>assert</b>(txn_sequence_number &gt;= sender_account.sequence_number, 3);
    <b>assert</b>(txn_sequence_number == sender_account.sequence_number, 4);
    <b>assert</b>(<a href="LibraTransactionTimeout.md#0x1_LibraTransactionTimeout_is_valid_transaction_timestamp">LibraTransactionTimeout::is_valid_transaction_timestamp</a>(txn_expiration_time), 7);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_epilogue"></a>

## Function `epilogue`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(sender: address, transaction_fee_amount: u64, txn_sequence_number: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(
    sender: address,
    transaction_fee_amount: u64,
    txn_sequence_number: u64,
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    // Load the transaction sender's account and balance resources
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(sender);

    // Bump the sequence number
    sender_account.sequence_number = txn_sequence_number + 1;

    <b>if</b> (transaction_fee_amount &gt; 0) {
        <b>let</b> sender_balance = borrow_global_mut&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(sender);
        <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">TransactionFee::pay_fee</a>(
            <a href="#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>(sender, sender_balance, transaction_fee_amount)
        )
    }
}
</code></pre>



</details>

<a name="0x1_LibraAccount_success_epilogue"></a>

## Function `success_epilogue`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_success_epilogue">success_epilogue</a>&lt;Token&gt;(account: &signer, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_success_epilogue">success_epilogue</a>&lt;Token&gt;(
    account: &signer,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);

    // Charge for gas
    <b>let</b> transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);

    // Load the transaction sender's balance <b>resource</b> only <b>if</b> it exists. If it doesn't we default the value <b>to</b> 0
    <b>let</b> sender_balance = <b>if</b> (exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(sender)) <a href="#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(sender) <b>else</b> 0;
    <b>assert</b>(sender_balance &gt;= transaction_fee_amount, 6);
    <a href="#0x1_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(sender, transaction_fee_amount, txn_sequence_number);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_failure_epilogue"></a>

## Function `failure_epilogue`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_failure_epilogue">failure_epilogue</a>&lt;Token&gt;(account: &signer, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_failure_epilogue">failure_epilogue</a>&lt;Token&gt;(
    account: &signer,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Charge for gas
    <b>let</b> transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);

    <a href="#0x1_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(sender, transaction_fee_amount, txn_sequence_number);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_bump_sequence_number"></a>

## Function `bump_sequence_number`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_bump_sequence_number">bump_sequence_number</a>(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_bump_sequence_number">bump_sequence_number</a>(signer: &signer) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
    sender_account.sequence_number = sender_account.sequence_number + 1;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_validator_account"></a>

## Function `create_validator_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_validator_account">create_validator_account</a>(creator_account: &signer, assoc_root_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_LibraRootRole">Roles::LibraRootRole</a>&gt;, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_validator_account">create_validator_account</a>(
    creator_account: &signer,
    assoc_root_capability: &Capability&lt;LibraRootRole&gt;,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
) {
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="Roles.md#0x1_Roles_new_validator_role">Roles::new_validator_role</a>(creator_account, &new_account);
    <a href="ValidatorConfig.md#0x1_ValidatorConfig_publish">ValidatorConfig::publish</a>(&new_account, assoc_root_capability);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_validator_operator_account"></a>

## Function `create_validator_operator_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_validator_operator_account">create_validator_operator_account</a>(creator_account: &signer, _: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="Roles.md#0x1_Roles_LibraRootRole">Roles::LibraRootRole</a>&gt;, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_validator_operator_account">create_validator_operator_account</a>(
    creator_account: &signer,
    _: &Capability&lt;LibraRootRole&gt;,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
) {
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="Roles.md#0x1_Roles_new_validator_operator_role">Roles::new_validator_operator_role</a>(creator_account, &new_account);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_Specification"></a>

## Specification


Returns field
<code>key_rotation_capability</code> of the
LibraAccount under
<code>addr</code>.


<a name="0x1_LibraAccount_spec_get_key_rotation_cap"></a>


<pre><code><b>define</b> <a href="#0x1_LibraAccount_spec_get_key_rotation_cap">spec_get_key_rotation_cap</a>(addr: address):
    <a href="Option.md#0x1_Option">Option</a>&lt;<a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>&gt; {
    <b>global</b>&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).key_rotation_capability
}
</code></pre>


Returns true if the LibraAccount at
<code>addr</code> holds
<code><a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a></code> for itself.


<a name="0x1_LibraAccount_spec_holds_own_key_rotation_cap"></a>


<pre><code><b>define</b> <a href="#0x1_LibraAccount_spec_holds_own_key_rotation_cap">spec_holds_own_key_rotation_cap</a>(addr: address): bool {
    <a href="Option.md#0x1_Option_spec_is_some">Option::spec_is_some</a>(<a href="#0x1_LibraAccount_spec_get_key_rotation_cap">spec_get_key_rotation_cap</a>(addr))
    && addr == <a href="Option.md#0x1_Option_spec_value_inside">Option::spec_value_inside</a>(
        <a href="#0x1_LibraAccount_spec_get_key_rotation_cap">spec_get_key_rotation_cap</a>(addr)).account_address
}
</code></pre>
