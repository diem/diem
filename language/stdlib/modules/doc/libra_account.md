
<a name="0x0_LibraAccount"></a>

# Module `0x0::LibraAccount`

### Table of Contents

-  [Struct `T`](#0x0_LibraAccount_T)
-  [Struct `Role`](#0x0_LibraAccount_Role)
-  [Struct `Balance`](#0x0_LibraAccount_Balance)
-  [Struct `WithdrawalCapability`](#0x0_LibraAccount_WithdrawalCapability)
-  [Struct `KeyRotationCapability`](#0x0_LibraAccount_KeyRotationCapability)
-  [Struct `SentPaymentEvent`](#0x0_LibraAccount_SentPaymentEvent)
-  [Struct `ReceivedPaymentEvent`](#0x0_LibraAccount_ReceivedPaymentEvent)
-  [Struct `FreezingPrivilege`](#0x0_LibraAccount_FreezingPrivilege)
-  [Struct `FreezeAccountEvent`](#0x0_LibraAccount_FreezeAccountEvent)
-  [Struct `UnfreezeAccountEvent`](#0x0_LibraAccount_UnfreezeAccountEvent)
-  [Struct `AccountOperationsCapability`](#0x0_LibraAccount_AccountOperationsCapability)
-  [Function `is`](#0x0_LibraAccount_is)
-  [Function `is_unhosted`](#0x0_LibraAccount_is_unhosted)
-  [Function `is_parent_vasp`](#0x0_LibraAccount_is_parent_vasp)
-  [Function `is_child_vasp`](#0x0_LibraAccount_is_child_vasp)
-  [Function `is_vasp`](#0x0_LibraAccount_is_vasp)
-  [Function `parent_vasp_address`](#0x0_LibraAccount_parent_vasp_address)
-  [Function `compliance_public_key`](#0x0_LibraAccount_compliance_public_key)
-  [Function `expiration_date`](#0x0_LibraAccount_expiration_date)
-  [Function `base_url`](#0x0_LibraAccount_base_url)
-  [Function `human_name`](#0x0_LibraAccount_human_name)
-  [Function `rotate_compliance_public_key`](#0x0_LibraAccount_rotate_compliance_public_key)
-  [Function `rotate_base_url`](#0x0_LibraAccount_rotate_base_url)
-  [Function `add_parent_vasp_role_from_association`](#0x0_LibraAccount_add_parent_vasp_role_from_association)
-  [Function `initialize`](#0x0_LibraAccount_initialize)
-  [Function `deposit`](#0x0_LibraAccount_deposit)
-  [Function `deposit_to_sender`](#0x0_LibraAccount_deposit_to_sender)
-  [Function `deposit_with_metadata`](#0x0_LibraAccount_deposit_with_metadata)
-  [Function `deposit_with_sender_and_metadata`](#0x0_LibraAccount_deposit_with_sender_and_metadata)
-  [Function `mint_to_address`](#0x0_LibraAccount_mint_to_address)
-  [Function `mint_lbr_to_address`](#0x0_LibraAccount_mint_lbr_to_address)
-  [Function `cancel_burn`](#0x0_LibraAccount_cancel_burn)
-  [Function `withdraw_from_balance`](#0x0_LibraAccount_withdraw_from_balance)
-  [Function `withdraw_from_sender`](#0x0_LibraAccount_withdraw_from_sender)
-  [Function `withdraw_with_capability`](#0x0_LibraAccount_withdraw_with_capability)
-  [Function `extract_sender_withdrawal_capability`](#0x0_LibraAccount_extract_sender_withdrawal_capability)
-  [Function `restore_withdrawal_capability`](#0x0_LibraAccount_restore_withdrawal_capability)
-  [Function `pay_from_capability`](#0x0_LibraAccount_pay_from_capability)
-  [Function `pay_from_sender_with_metadata`](#0x0_LibraAccount_pay_from_sender_with_metadata)
-  [Function `pay_from_sender`](#0x0_LibraAccount_pay_from_sender)
-  [Function `rotate_authentication_key_for_account`](#0x0_LibraAccount_rotate_authentication_key_for_account)
-  [Function `rotate_authentication_key`](#0x0_LibraAccount_rotate_authentication_key)
-  [Function `rotate_authentication_key_with_capability`](#0x0_LibraAccount_rotate_authentication_key_with_capability)
-  [Function `extract_sender_key_rotation_capability`](#0x0_LibraAccount_extract_sender_key_rotation_capability)
-  [Function `restore_key_rotation_capability`](#0x0_LibraAccount_restore_key_rotation_capability)
-  [Function `create_testnet_account`](#0x0_LibraAccount_create_testnet_account)
-  [Function `make_account`](#0x0_LibraAccount_make_account)
-  [Function `create_genesis_account`](#0x0_LibraAccount_create_genesis_account)
-  [Function `create_treasury_compliance_account`](#0x0_LibraAccount_create_treasury_compliance_account)
-  [Function `is_designated_dealer`](#0x0_LibraAccount_is_designated_dealer)
-  [Function `add_tier`](#0x0_LibraAccount_add_tier)
-  [Function `update_tier`](#0x0_LibraAccount_update_tier)
-  [Function `create_designated_dealer`](#0x0_LibraAccount_create_designated_dealer)
-  [Function `mint_to_designated_dealer`](#0x0_LibraAccount_mint_to_designated_dealer)
-  [Function `create_parent_vasp_account`](#0x0_LibraAccount_create_parent_vasp_account)
-  [Function `create_child_vasp_account`](#0x0_LibraAccount_create_child_vasp_account)
-  [Function `create_unhosted_account`](#0x0_LibraAccount_create_unhosted_account)
-  [Function `create_signer`](#0x0_LibraAccount_create_signer)
-  [Function `destroy_signer`](#0x0_LibraAccount_destroy_signer)
-  [Function `balance_for`](#0x0_LibraAccount_balance_for)
-  [Function `balance`](#0x0_LibraAccount_balance)
-  [Function `add_currency`](#0x0_LibraAccount_add_currency)
-  [Function `accepts_currency`](#0x0_LibraAccount_accepts_currency)
-  [Function `sequence_number_for_account`](#0x0_LibraAccount_sequence_number_for_account)
-  [Function `sequence_number`](#0x0_LibraAccount_sequence_number)
-  [Function `authentication_key`](#0x0_LibraAccount_authentication_key)
-  [Function `delegated_key_rotation_capability`](#0x0_LibraAccount_delegated_key_rotation_capability)
-  [Function `delegated_withdrawal_capability`](#0x0_LibraAccount_delegated_withdrawal_capability)
-  [Function `withdrawal_capability_address`](#0x0_LibraAccount_withdrawal_capability_address)
-  [Function `key_rotation_capability_address`](#0x0_LibraAccount_key_rotation_capability_address)
-  [Function `exists`](#0x0_LibraAccount_exists)
-  [Function `freeze_account`](#0x0_LibraAccount_freeze_account)
-  [Function `unfreeze_account`](#0x0_LibraAccount_unfreeze_account)
-  [Function `account_is_frozen`](#0x0_LibraAccount_account_is_frozen)
-  [Function `assert_can_freeze`](#0x0_LibraAccount_assert_can_freeze)
-  [Function `prologue`](#0x0_LibraAccount_prologue)
-  [Function `epilogue`](#0x0_LibraAccount_epilogue)
-  [Function `bump_sequence_number`](#0x0_LibraAccount_bump_sequence_number)



<a name="0x0_LibraAccount_T"></a>

## Struct `T`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraAccount_T">T</a>
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

<code>delegated_key_rotation_capability: bool</code>
</dt>
<dd>

</dd>
<dt>

<code>delegated_withdrawal_capability: bool</code>
</dt>
<dd>

</dd>
<dt>

<code>received_events: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>sent_events: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a>&gt;</code>
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

<a name="0x0_LibraAccount_Role"></a>

## Struct `Role`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraAccount_Role">Role</a>&lt;RoleData: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>role_data: RoleData</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraAccount_Balance"></a>

## Struct `Balance`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>coin: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraAccount_WithdrawalCapability"></a>

## Struct `WithdrawalCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraAccount_WithdrawalCapability">WithdrawalCapability</a>
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

<a name="0x0_LibraAccount_KeyRotationCapability"></a>

## Struct `KeyRotationCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>
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

<a name="0x0_LibraAccount_SentPaymentEvent"></a>

## Struct `SentPaymentEvent`



<pre><code><b>struct</b> <a href="#0x0_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>
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

<a name="0x0_LibraAccount_ReceivedPaymentEvent"></a>

## Struct `ReceivedPaymentEvent`



<pre><code><b>struct</b> <a href="#0x0_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>
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

<a name="0x0_LibraAccount_FreezingPrivilege"></a>

## Struct `FreezingPrivilege`



<pre><code><b>struct</b> <a href="#0x0_LibraAccount_FreezingPrivilege">FreezingPrivilege</a>
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

<a name="0x0_LibraAccount_FreezeAccountEvent"></a>

## Struct `FreezeAccountEvent`



<pre><code><b>struct</b> <a href="#0x0_LibraAccount_FreezeAccountEvent">FreezeAccountEvent</a>
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

<a name="0x0_LibraAccount_UnfreezeAccountEvent"></a>

## Struct `UnfreezeAccountEvent`



<pre><code><b>struct</b> <a href="#0x0_LibraAccount_UnfreezeAccountEvent">UnfreezeAccountEvent</a>
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

<a name="0x0_LibraAccount_AccountOperationsCapability"></a>

## Struct `AccountOperationsCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>limits_cap: <a href="account_limits.md#0x0_AccountLimits_CallingCapability">AccountLimits::CallingCapability</a></code>
</dt>
<dd>

</dd>
<dt>

<code>freeze_event_handle: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_LibraAccount_FreezeAccountEvent">LibraAccount::FreezeAccountEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>unfreeze_event_handle: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_LibraAccount_UnfreezeAccountEvent">LibraAccount::UnfreezeAccountEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraAccount_is"></a>

## Function `is`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is">is</a>&lt;RoleData: <b>copyable</b>&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is">is</a>&lt;RoleData: <b>copyable</b>&gt;(addr: address): bool {
    ::<a href="#0x0_LibraAccount_exists">exists</a>&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;RoleData&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_is_unhosted"></a>

## Function `is_unhosted`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is_unhosted">is_unhosted</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is_unhosted">is_unhosted</a>(addr: address): bool {
    <a href="#0x0_LibraAccount_is">is</a>&lt;<a href="unhosted.md#0x0_Unhosted_T">Unhosted::T</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_is_parent_vasp"></a>

## Function `is_parent_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is_parent_vasp">is_parent_vasp</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is_parent_vasp">is_parent_vasp</a>(addr: address): bool {
    <a href="#0x0_LibraAccount_is">is</a>&lt;<a href="vasp.md#0x0_VASP_ParentVASP">VASP::ParentVASP</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_is_child_vasp"></a>

## Function `is_child_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is_child_vasp">is_child_vasp</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is_child_vasp">is_child_vasp</a>(addr: address): bool {
    <a href="#0x0_LibraAccount_is">is</a>&lt;<a href="vasp.md#0x0_VASP_ChildVASP">VASP::ChildVASP</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_is_vasp"></a>

## Function `is_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is_vasp">is_vasp</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is_vasp">is_vasp</a>(addr: address): bool {
    <a href="#0x0_LibraAccount_is_parent_vasp">is_parent_vasp</a>(addr) || <a href="#0x0_LibraAccount_is_child_vasp">is_child_vasp</a>(addr)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_parent_vasp_address"></a>

## Function `parent_vasp_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_parent_vasp_address">parent_vasp_address</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_parent_vasp_address">parent_vasp_address</a>(addr: address): address <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a> {
    <b>if</b> (<a href="#0x0_LibraAccount_is_parent_vasp">is_parent_vasp</a>(addr)) {
        addr
    } <b>else</b> <b>if</b> (<a href="#0x0_LibraAccount_is_child_vasp">is_child_vasp</a>(addr)) {
        <b>let</b> child_vasp = &borrow_global&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="vasp.md#0x0_VASP_ChildVASP">VASP::ChildVASP</a>&gt;&gt;(addr).role_data;
        <a href="vasp.md#0x0_VASP_child_parent_address">VASP::child_parent_address</a>(child_vasp)
    } <b>else</b> { // wrong account type, <b>abort</b>
        <b>abort</b>(88)
    }
}
</code></pre>



</details>

<a name="0x0_LibraAccount_compliance_public_key"></a>

## Function `compliance_public_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_compliance_public_key">compliance_public_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_compliance_public_key">compliance_public_key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a> {
    <b>let</b> parent_vasp = &borrow_global&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="vasp.md#0x0_VASP_ParentVASP">VASP::ParentVASP</a>&gt;&gt;(addr).role_data;
    <a href="vasp.md#0x0_VASP_compliance_public_key">VASP::compliance_public_key</a>(parent_vasp)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_expiration_date"></a>

## Function `expiration_date`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_expiration_date">expiration_date</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_expiration_date">expiration_date</a>(addr: address): u64 <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a> {
    <b>let</b> parent_vasp = &borrow_global&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="vasp.md#0x0_VASP_ParentVASP">VASP::ParentVASP</a>&gt;&gt;(addr).role_data;
    <a href="vasp.md#0x0_VASP_expiration_date">VASP::expiration_date</a>(parent_vasp)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_base_url"></a>

## Function `base_url`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_base_url">base_url</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_base_url">base_url</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a> {
    <b>let</b> parent_vasp = &borrow_global&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="vasp.md#0x0_VASP_ParentVASP">VASP::ParentVASP</a>&gt;&gt;(addr).role_data;
    <a href="vasp.md#0x0_VASP_base_url">VASP::base_url</a>(parent_vasp)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_human_name"></a>

## Function `human_name`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_human_name">human_name</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_human_name">human_name</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a> {
    <b>let</b> parent_vasp = &borrow_global&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="vasp.md#0x0_VASP_ParentVASP">VASP::ParentVASP</a>&gt;&gt;(addr).role_data;
    <a href="vasp.md#0x0_VASP_human_name">VASP::human_name</a>(parent_vasp)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_rotate_compliance_public_key"></a>

## Function `rotate_compliance_public_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_rotate_compliance_public_key">rotate_compliance_public_key</a>(vasp: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_rotate_compliance_public_key">rotate_compliance_public_key</a>(vasp: &signer, new_key: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a> {
    <b>let</b> parent_vasp =
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="vasp.md#0x0_VASP_ParentVASP">VASP::ParentVASP</a>&gt;&gt;(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(vasp)).role_data;
    <a href="vasp.md#0x0_VASP_rotate_compliance_public_key">VASP::rotate_compliance_public_key</a>(parent_vasp, new_key)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_rotate_base_url"></a>

## Function `rotate_base_url`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_rotate_base_url">rotate_base_url</a>(vasp: &signer, new_url: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_rotate_base_url">rotate_base_url</a>(vasp: &signer, new_url: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a> {
    <b>let</b> parent_vasp =
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="vasp.md#0x0_VASP_ParentVASP">VASP::ParentVASP</a>&gt;&gt;(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(vasp)).role_data;
    <a href="vasp.md#0x0_VASP_rotate_base_url">VASP::rotate_base_url</a>(parent_vasp, new_url)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_add_parent_vasp_role_from_association"></a>

## Function `add_parent_vasp_role_from_association`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_add_parent_vasp_role_from_association">add_parent_vasp_role_from_association</a>(association: &signer, addr: address, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_add_parent_vasp_role_from_association">add_parent_vasp_role_from_association</a>(
    association: &signer,
    addr: address,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;,
) {
    Transaction::assert(<a href="#0x0_LibraAccount_exists">exists</a>(addr), 0);
    Transaction::assert(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(association) == 0xA550C18, 0);
    <b>let</b> role_data =
        <a href="vasp.md#0x0_VASP_create_parent_vasp_credential">VASP::create_parent_vasp_credential</a>(human_name, base_url, compliance_public_key);
    <b>let</b> account = <a href="#0x0_LibraAccount_create_signer">create_signer</a>(addr);
    move_to(&account, <a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="vasp.md#0x0_VASP_ParentVASP">VASP::ParentVASP</a>&gt; { role_data });
    <a href="#0x0_LibraAccount_destroy_signer">destroy_signer</a>(account);
}
</code></pre>



</details>

<a name="0x0_LibraAccount_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_initialize">initialize</a>(association: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_initialize">initialize</a>(association: &signer) {
    Transaction::assert(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(association) == 0xA550C18, 0);
    move_to(
        association,
        <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
            limits_cap: <a href="account_limits.md#0x0_AccountLimits_grant_calling_capability">AccountLimits::grant_calling_capability</a>(),
            freeze_event_handle: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>(association),
            unfreeze_event_handle: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>(association),
        }
    );
}
</code></pre>



</details>

<a name="0x0_LibraAccount_deposit"></a>

## Function `deposit`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_deposit">deposit</a>&lt;Token&gt;(payee: address, to_deposit: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_deposit">deposit</a>&lt;Token&gt;(payee: address, to_deposit: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;)
<b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Role">Role</a> {
    // Since we don't have vector&lt;u8&gt; literals in the source language at
    // the moment.
    <a href="#0x0_LibraAccount_deposit_with_metadata">deposit_with_metadata</a>(payee, to_deposit, x"", x"")
}
</code></pre>



</details>

<a name="0x0_LibraAccount_deposit_to_sender"></a>

## Function `deposit_to_sender`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_deposit_to_sender">deposit_to_sender</a>&lt;Token&gt;(to_deposit: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_deposit_to_sender">deposit_to_sender</a>&lt;Token&gt;(to_deposit: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;)
<b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Role">Role</a> {
    <a href="#0x0_LibraAccount_deposit">deposit</a>(Transaction::sender(), to_deposit)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_deposit_with_metadata"></a>

## Function `deposit_with_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_deposit_with_metadata">deposit_with_metadata</a>&lt;Token&gt;(payee: address, to_deposit: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_deposit_with_metadata">deposit_with_metadata</a>&lt;Token&gt;(
    payee: address,
    to_deposit: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Role">Role</a> {
    <a href="#0x0_LibraAccount_deposit_with_sender_and_metadata">deposit_with_sender_and_metadata</a>(
        payee,
        Transaction::sender(),
        to_deposit,
        metadata,
        metadata_signature
    );
}
</code></pre>



</details>

<a name="0x0_LibraAccount_deposit_with_sender_and_metadata"></a>

## Function `deposit_with_sender_and_metadata`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_deposit_with_sender_and_metadata">deposit_with_sender_and_metadata</a>&lt;Token&gt;(payee: address, sender: address, to_deposit: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_deposit_with_sender_and_metadata">deposit_with_sender_and_metadata</a>&lt;Token&gt;(
    payee: address,
    sender: address,
    to_deposit: <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Role">Role</a> {
    // Check that the `to_deposit` coin is non-zero
    <b>let</b> deposit_value = <a href="libra.md#0x0_Libra_value">Libra::value</a>(&to_deposit);
    Transaction::assert(deposit_value &gt; 0, 7);

    // TODO: on-chain config for travel rule limit instead of hardcoded value
    // TODO: nail down details of limit (specified in <a href="lbr.md#0x0_LBR">LBR</a>? is 1 <a href="lbr.md#0x0_LBR">LBR</a> a milliLibra or microLibra?)
    <b>let</b> travel_rule_limit = 1000;
    // travel rule only applies for payments over a threshold
    <b>let</b> above_threshold =
        <a href="libra.md#0x0_Libra_approx_lbr_for_value">Libra::approx_lbr_for_value</a>&lt;Token&gt;(deposit_value) &gt;= travel_rule_limit;
    // travel rule only applies <b>if</b> the sender and recipient are both VASPs
    <b>let</b> both_vasps = <a href="#0x0_LibraAccount_is_vasp">is_vasp</a>(sender) && <a href="#0x0_LibraAccount_is_vasp">is_vasp</a>(payee);
    // Don't check the travel rule <b>if</b> we're on testnet and sender
    // doesn't specify a metadata signature
    <b>let</b> is_testnet_transfer = <a href="testnet.md#0x0_Testnet_is_testnet">Testnet::is_testnet</a>() && <a href="vector.md#0x0_Vector_is_empty">Vector::is_empty</a>(&metadata_signature);
    <b>if</b> (!is_testnet_transfer &&
        above_threshold &&
        both_vasps &&
        // travel rule does not <b>apply</b> for intra-<a href="vasp.md#0x0_VASP">VASP</a> transactions
        <a href="#0x0_LibraAccount_parent_vasp_address">parent_vasp_address</a>(sender) != <a href="#0x0_LibraAccount_parent_vasp_address">parent_vasp_address</a>(payee)
    ) {
        // sanity check of signature validity
        Transaction::assert(<a href="vector.md#0x0_Vector_length">Vector::length</a>(&metadata_signature) == 64, 9001);
        // message should be metadata | sender_address | amount | domain_separator
        // separator is the UTF8-encoded string @@$$LIBRA_ATTEST$$@@
        <b>let</b> domain_separator = x"404024244C494252415F41545445535424244040";
        <b>let</b> message = <b>copy</b> metadata;
        <a href="vector.md#0x0_Vector_append">Vector::append</a>(&<b>mut</b> message, <a href="lcs.md#0x0_LCS_to_bytes">LCS::to_bytes</a>(&sender));
        <a href="vector.md#0x0_Vector_append">Vector::append</a>(&<b>mut</b> message, <a href="lcs.md#0x0_LCS_to_bytes">LCS::to_bytes</a>(&deposit_value));
        <a href="vector.md#0x0_Vector_append">Vector::append</a>(&<b>mut</b> message, domain_separator);
        // cryptographic check of signature validity
        Transaction::assert(
            <a href="signature.md#0x0_Signature_ed25519_verify">Signature::ed25519_verify</a>(
                metadata_signature,
                <a href="#0x0_LibraAccount_compliance_public_key">compliance_public_key</a>(payee),
                message
            ),
            9002, // TODO: proper error code
        );
    };

    // Ensure that this deposit is compliant with the account limits on
    // this account.
    <b>let</b> _ = borrow_global&lt;<a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(0xA550C18);
    /*Transaction::assert(
        <a href="account_limits.md#0x0_AccountLimits_update_deposit_limits">AccountLimits::update_deposit_limits</a>&lt;Token&gt;(
            deposit_value,
            payee,
            &borrow_global&lt;<a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(0xA550C18).limits_cap
        ),
        9
    );*/

    // Get the code symbol for this currency
    <b>let</b> currency_code = <a href="libra.md#0x0_Libra_currency_code">Libra::currency_code</a>&lt;Token&gt;();

    // Load the sender's account
    <b>let</b> sender_account_ref = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(sender);
    // Log a sent event
    <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x0_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>&gt;(
        &<b>mut</b> sender_account_ref.sent_events,
        <a href="#0x0_LibraAccount_SentPaymentEvent">SentPaymentEvent</a> {
            amount: deposit_value,
            currency_code: <b>copy</b> currency_code,
            payee: payee,
            metadata: *&metadata
        },
    );

    // Load the payee's account
    <b>let</b> payee_account_ref = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(payee);
    <b>let</b> payee_balance = borrow_global_mut&lt;<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee);
    // Deposit the `to_deposit` coin
    <a href="libra.md#0x0_Libra_deposit">Libra::deposit</a>(&<b>mut</b> payee_balance.coin, to_deposit);
    // Log a received event
    <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x0_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(
        &<b>mut</b> payee_account_ref.received_events,
        <a href="#0x0_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a> {
            amount: deposit_value,
            currency_code,
            payer: sender,
            metadata: metadata
        }
    );
}
</code></pre>



</details>

<a name="0x0_LibraAccount_mint_to_address"></a>

## Function `mint_to_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_mint_to_address">mint_to_address</a>&lt;Token&gt;(payee: address, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_mint_to_address">mint_to_address</a>&lt;Token&gt;(
    payee: address,
    amount: u64
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Role">Role</a> {
    // Mint and deposit the coin
    <a href="#0x0_LibraAccount_deposit">deposit</a>(payee, <a href="libra.md#0x0_Libra_mint">Libra::mint</a>&lt;Token&gt;(amount));
}
</code></pre>



</details>

<a name="0x0_LibraAccount_mint_lbr_to_address"></a>

## Function `mint_lbr_to_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_mint_lbr_to_address">mint_lbr_to_address</a>(payee: address, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_mint_lbr_to_address">mint_lbr_to_address</a>(
    payee: address,
    amount: u64
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Role">Role</a> {
    // Mint and deposit the coin
    <a href="#0x0_LibraAccount_deposit">deposit</a>(payee, <a href="lbr.md#0x0_LBR_mint">LBR::mint</a>(amount));
}
</code></pre>



</details>

<a name="0x0_LibraAccount_cancel_burn"></a>

## Function `cancel_burn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_cancel_burn">cancel_burn</a>&lt;Token&gt;(preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_cancel_burn">cancel_burn</a>&lt;Token&gt;(
    preburn_address: address,
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Role">Role</a> {
    <b>let</b> to_return = <a href="libra.md#0x0_Libra_cancel_burn">Libra::cancel_burn</a>&lt;Token&gt;(preburn_address);
    <a href="#0x0_LibraAccount_deposit">deposit</a>(preburn_address, to_return)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_withdraw_from_balance"></a>

## Function `withdraw_from_balance`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(_addr: address, balance: &<b>mut</b> <a href="#0x0_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;, amount: u64): <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(
    _addr: address,
    balance: &<b>mut</b> <a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;,
    amount: u64
): <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    // Make sure that this withdrawal is compliant with the limits on
    // the account.
    <b>let</b> _  = borrow_global&lt;<a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(0xA550C18);
    /*<b>let</b> can_withdraw = <a href="account_limits.md#0x0_AccountLimits_update_withdrawal_limits">AccountLimits::update_withdrawal_limits</a>&lt;Token&gt;(
        amount,
        addr,
        &borrow_global&lt;<a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(0xA550C18).limits_cap
    );
    Transaction::assert(can_withdraw, 11);*/
    <a href="libra.md#0x0_Libra_withdraw">Libra::withdraw</a>(&<b>mut</b> balance.coin, amount)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_withdraw_from_sender"></a>

## Function `withdraw_from_sender`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_withdraw_from_sender">withdraw_from_sender</a>&lt;Token&gt;(amount: u64): <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_withdraw_from_sender">withdraw_from_sender</a>&lt;Token&gt;(amount: u64): <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;
<b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> sender = Transaction::sender();
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(sender);
    <b>let</b> sender_balance = borrow_global_mut&lt;<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(sender);
    // The sender has delegated the privilege <b>to</b> withdraw from her account elsewhere--<b>abort</b>.
    Transaction::assert(!sender_account.delegated_withdrawal_capability, 11);
    // The sender has retained her withdrawal privileges--proceed.
    <a href="#0x0_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(sender, sender_balance, amount)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_withdraw_with_capability"></a>

## Function `withdraw_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_withdraw_with_capability">withdraw_with_capability</a>&lt;Token&gt;(cap: &<a href="#0x0_LibraAccount_WithdrawalCapability">LibraAccount::WithdrawalCapability</a>, amount: u64): <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_withdraw_with_capability">withdraw_with_capability</a>&lt;Token&gt;(
    cap: &<a href="#0x0_LibraAccount_WithdrawalCapability">WithdrawalCapability</a>, amount: u64
): <a href="libra.md#0x0_Libra_T">Libra::T</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> balance = borrow_global_mut&lt;<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(cap.account_address);
    <a href="#0x0_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(cap.account_address, balance , amount)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_extract_sender_withdrawal_capability"></a>

## Function `extract_sender_withdrawal_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_extract_sender_withdrawal_capability">extract_sender_withdrawal_capability</a>(): <a href="#0x0_LibraAccount_WithdrawalCapability">LibraAccount::WithdrawalCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_extract_sender_withdrawal_capability">extract_sender_withdrawal_capability</a>(): <a href="#0x0_LibraAccount_WithdrawalCapability">WithdrawalCapability</a> <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    <b>let</b> sender = Transaction::sender();
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(sender);

    // Abort <b>if</b> we already extracted the unique withdrawal capability for this account.
    Transaction::assert(!sender_account.delegated_withdrawal_capability, 11);

    // Ensure the uniqueness of the capability
    sender_account.delegated_withdrawal_capability = <b>true</b>;
    <a href="#0x0_LibraAccount_WithdrawalCapability">WithdrawalCapability</a> { account_address: sender }
}
</code></pre>



</details>

<a name="0x0_LibraAccount_restore_withdrawal_capability"></a>

## Function `restore_withdrawal_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_restore_withdrawal_capability">restore_withdrawal_capability</a>(cap: <a href="#0x0_LibraAccount_WithdrawalCapability">LibraAccount::WithdrawalCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_restore_withdrawal_capability">restore_withdrawal_capability</a>(cap: <a href="#0x0_LibraAccount_WithdrawalCapability">WithdrawalCapability</a>) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    // Destroy the capability
    <b>let</b> <a href="#0x0_LibraAccount_WithdrawalCapability">WithdrawalCapability</a> { account_address } = cap;
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(account_address);
    // Update the flag for `account_address` <b>to</b> indicate that the capability has been restored.
    // The account owner will now be able <b>to</b> call pay_from_sender, withdraw_from_sender, and
    // extract_sender_withdrawal_capability again.
    account.delegated_withdrawal_capability = <b>false</b>;
}
</code></pre>



</details>

<a name="0x0_LibraAccount_pay_from_capability"></a>

## Function `pay_from_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_pay_from_capability">pay_from_capability</a>&lt;Token&gt;(payee: address, cap: &<a href="#0x0_LibraAccount_WithdrawalCapability">LibraAccount::WithdrawalCapability</a>, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_pay_from_capability">pay_from_capability</a>&lt;Token&gt;(
    payee: address,
    cap: &<a href="#0x0_LibraAccount_WithdrawalCapability">WithdrawalCapability</a>,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Role">Role</a> {
    <a href="#0x0_LibraAccount_deposit_with_sender_and_metadata">deposit_with_sender_and_metadata</a>&lt;Token&gt;(
        payee,
        *&cap.account_address,
        <a href="#0x0_LibraAccount_withdraw_with_capability">withdraw_with_capability</a>(cap, amount),
        metadata,
        metadata_signature
    );
}
</code></pre>



</details>

<a name="0x0_LibraAccount_pay_from_sender_with_metadata"></a>

## Function `pay_from_sender_with_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_pay_from_sender_with_metadata">pay_from_sender_with_metadata</a>&lt;Token&gt;(payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_pay_from_sender_with_metadata">pay_from_sender_with_metadata</a>&lt;Token&gt;(
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Role">Role</a> {
    <a href="#0x0_LibraAccount_deposit_with_metadata">deposit_with_metadata</a>&lt;Token&gt;(
        payee,
        <a href="#0x0_LibraAccount_withdraw_from_sender">withdraw_from_sender</a>(amount),
        metadata,
        metadata_signature
    );
}
</code></pre>



</details>

<a name="0x0_LibraAccount_pay_from_sender"></a>

## Function `pay_from_sender`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_pay_from_sender">pay_from_sender</a>&lt;Token&gt;(payee: address, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_pay_from_sender">pay_from_sender</a>&lt;Token&gt;(
    payee: address,
    amount: u64
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Role">Role</a> {
    <a href="#0x0_LibraAccount_pay_from_sender_with_metadata">pay_from_sender_with_metadata</a>&lt;Token&gt;(payee, amount, x"", x"");
}
</code></pre>



</details>

<a name="0x0_LibraAccount_rotate_authentication_key_for_account"></a>

## Function `rotate_authentication_key_for_account`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_rotate_authentication_key_for_account">rotate_authentication_key_for_account</a>(account: &<b>mut</b> <a href="#0x0_LibraAccount_T">LibraAccount::T</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_rotate_authentication_key_for_account">rotate_authentication_key_for_account</a>(account: &<b>mut</b> <a href="#0x0_LibraAccount_T">T</a>, new_authentication_key: vector&lt;u8&gt;) {
  // Don't allow rotating <b>to</b> clearly invalid key
  Transaction::assert(<a href="vector.md#0x0_Vector_length">Vector::length</a>(&new_authentication_key) == 32, 12);
  account.authentication_key = new_authentication_key;
}
</code></pre>



</details>

<a name="0x0_LibraAccount_rotate_authentication_key"></a>

## Function `rotate_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_rotate_authentication_key">rotate_authentication_key</a>(new_authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_rotate_authentication_key">rotate_authentication_key</a>(new_authentication_key: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(Transaction::sender());
    // The sender has delegated the privilege <b>to</b> rotate her key elsewhere--<b>abort</b>
    Transaction::assert(!sender_account.delegated_key_rotation_capability, 11);
    // The sender has retained her key rotation privileges--proceed.
    <a href="#0x0_LibraAccount_rotate_authentication_key_for_account">rotate_authentication_key_for_account</a>(
        sender_account,
        new_authentication_key
    );
}
</code></pre>



</details>

<a name="0x0_LibraAccount_rotate_authentication_key_with_capability"></a>

## Function `rotate_authentication_key_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_rotate_authentication_key_with_capability">rotate_authentication_key_with_capability</a>(cap: &<a href="#0x0_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_rotate_authentication_key_with_capability">rotate_authentication_key_with_capability</a>(
    cap: &<a href="#0x0_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>,
    new_authentication_key: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>  {
    <a href="#0x0_LibraAccount_rotate_authentication_key_for_account">rotate_authentication_key_for_account</a>(
        borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(*&cap.account_address),
        new_authentication_key
    );
}
</code></pre>



</details>

<a name="0x0_LibraAccount_extract_sender_key_rotation_capability"></a>

## Function `extract_sender_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_extract_sender_key_rotation_capability">extract_sender_key_rotation_capability</a>(): <a href="#0x0_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_extract_sender_key_rotation_capability">extract_sender_key_rotation_capability</a>(): <a href="#0x0_LibraAccount_KeyRotationCapability">KeyRotationCapability</a> <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    <b>let</b> sender = Transaction::sender();
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(sender);
    // Abort <b>if</b> we already extracted the unique key rotation capability for this account.
    Transaction::assert(!sender_account.delegated_key_rotation_capability, 11);
    sender_account.delegated_key_rotation_capability = <b>true</b>; // Ensure uniqueness of the capability
    <a href="#0x0_LibraAccount_KeyRotationCapability">KeyRotationCapability</a> { account_address: sender }
}
</code></pre>



</details>

<a name="0x0_LibraAccount_restore_key_rotation_capability"></a>

## Function `restore_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="#0x0_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="#0x0_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    // Destroy the capability
    <b>let</b> <a href="#0x0_LibraAccount_KeyRotationCapability">KeyRotationCapability</a> { account_address } = cap;
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(account_address);
    // Update the flag for `account_address` <b>to</b> indicate that the capability has been restored.
    // The account owner will now be able <b>to</b> call rotate_authentication_key and
    // extract_sender_key_rotation_capability again
    account.delegated_key_rotation_capability = <b>false</b>;
}
</code></pre>



</details>

<a name="0x0_LibraAccount_create_testnet_account"></a>

## Function `create_testnet_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_testnet_account">create_testnet_account</a>&lt;Token&gt;(new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_testnet_account">create_testnet_account</a>&lt;Token&gt;(
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;
) {
    Transaction::assert(<a href="testnet.md#0x0_Testnet_is_testnet">Testnet::is_testnet</a>(), 10042);
    <b>let</b> vasp_parent =
        <a href="vasp.md#0x0_VASP_create_parent_vasp_credential">VASP::create_parent_vasp_credential</a>(
            // "testnet"
            x"746573746E6574",
            // "https://libra.org"
            x"68747470733A2F2F6C696272612E6F72672F",
            // An empty compliance key
            x"00000000000000000000000000000000"
        );
    <b>let</b> new_account = <a href="#0x0_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="event.md#0x0_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x0_LibraAccount_make_account">make_account</a>&lt;Token, <a href="vasp.md#0x0_VASP_ParentVASP">VASP::ParentVASP</a>&gt;(new_account, auth_key_prefix, vasp_parent, <b>false</b>)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_make_account"></a>

## Function `make_account`

Creates a new account with account type
<code><a href="#0x0_LibraAccount_Role">Role</a></code> at
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


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_make_account">make_account</a>&lt;Token, RoleData: <b>copyable</b>&gt;(new_account: signer, auth_key_prefix: vector&lt;u8&gt;, role_data: RoleData, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_make_account">make_account</a>&lt;Token, RoleData: <b>copyable</b>&gt;(
    new_account: signer,
    auth_key_prefix: vector&lt;u8&gt;,
    role_data: RoleData,
    add_all_currencies: bool
) {
    <b>let</b> new_account_addr = <a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(&new_account);
    // cannot create an account at the reserved address 0x0
    Transaction::assert(new_account_addr != 0x0, 0);

    // (1) publish Account::T
    <b>let</b> authentication_key = auth_key_prefix;
    <a href="vector.md#0x0_Vector_append">Vector::append</a>(
        &<b>mut</b> authentication_key, <a href="lcs.md#0x0_LCS_to_bytes">LCS::to_bytes</a>(<a href="signer.md#0x0_Signer_borrow_address">Signer::borrow_address</a>(&new_account))
    );
    Transaction::assert(<a href="vector.md#0x0_Vector_length">Vector::length</a>(&authentication_key) == 32, 12);
    move_to(
        &new_account,
        <a href="#0x0_LibraAccount_T">T</a> {
            authentication_key,
            delegated_key_rotation_capability: <b>false</b>,
            delegated_withdrawal_capability: <b>false</b>,
            received_events: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(&new_account),
            sent_events: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>&gt;(&new_account),
            sequence_number: 0,
            is_frozen: <b>false</b>,
        }
    );
    // (2) publish Account::Role. it's the caller's job <b>to</b> publish resources with role-specific
    //     configuration such <b>as</b> <a href="account_limits.md#0x0_AccountLimits">AccountLimits</a> before calling this
    move_to(&new_account, <a href="#0x0_LibraAccount_Role">Role</a>&lt;RoleData&gt; { role_data });
    // (3) publish <a href="#0x0_LibraAccount_Balance">Balance</a> <b>resource</b>(s)
    <a href="#0x0_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(&new_account);
    <b>if</b> (add_all_currencies) {
        <b>if</b> (!::<a href="#0x0_LibraAccount_exists">exists</a>&lt;<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;&gt;(new_account_addr)) {
            <a href="#0x0_LibraAccount_add_currency">add_currency</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(&new_account);
        };
        <b>if</b> (!::<a href="#0x0_LibraAccount_exists">exists</a>&lt;<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;&gt;(new_account_addr)) {
            <a href="#0x0_LibraAccount_add_currency">add_currency</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(&new_account);
        };
        <b>if</b> (!::<a href="#0x0_LibraAccount_exists">exists</a>&lt;<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;<a href="lbr.md#0x0_LBR_T">LBR::T</a>&gt;&gt;(new_account_addr)) {
            <a href="#0x0_LibraAccount_add_currency">add_currency</a>&lt;<a href="lbr.md#0x0_LBR_T">LBR::T</a>&gt;(&new_account);
        };
    };
    // (4) TODO: publish account limits?

    <a href="#0x0_LibraAccount_destroy_signer">destroy_signer</a>(new_account);
}
</code></pre>



</details>

<a name="0x0_LibraAccount_create_genesis_account"></a>

## Function `create_genesis_account`

Create an account with the Empty role at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_genesis_account">create_genesis_account</a>&lt;Token&gt;(new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_genesis_account">create_genesis_account</a>&lt;Token&gt;(
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;
) {
    Transaction::assert(<a href="libra_time.md#0x0_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), 0);
    <b>let</b> new_account = <a href="#0x0_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="#0x0_LibraAccount_make_account">make_account</a>&lt;Token, <a href="empty.md#0x0_Empty_T">Empty::T</a>&gt;(new_account, auth_key_prefix, <a href="empty.md#0x0_Empty_create">Empty::create</a>(), <b>false</b>)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_create_treasury_compliance_account"></a>

## Function `create_treasury_compliance_account`

Create a treasury/compliance account at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_treasury_compliance_account">create_treasury_compliance_account</a>&lt;Token&gt;(new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, coin1_mint_cap: <a href="libra.md#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;, coin1_burn_cap: <a href="libra.md#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;, coin2_mint_cap: <a href="libra.md#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;, coin2_burn_cap: <a href="libra.md#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_treasury_compliance_account">create_treasury_compliance_account</a>&lt;Token&gt;(
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    coin1_mint_cap: <a href="libra.md#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;,
    coin1_burn_cap: <a href="libra.md#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;,
    coin2_mint_cap: <a href="libra.md#0x0_Libra_MintCapability">Libra::MintCapability</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;,
    coin2_burn_cap: <a href="libra.md#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;,
) {
    <a href="association.md#0x0_Association_assert_sender_is_root">Association::assert_sender_is_root</a>();
    <b>let</b> new_account = <a href="#0x0_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="association.md#0x0_Association_grant_association_address">Association::grant_association_address</a>(&new_account);
    <a href="association.md#0x0_Association_grant_privilege">Association::grant_privilege</a>&lt;<a href="#0x0_LibraAccount_FreezingPrivilege">FreezingPrivilege</a>&gt;(&new_account);
    <a href="libra.md#0x0_Libra_publish_mint_capability">Libra::publish_mint_capability</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(&new_account, coin1_mint_cap);
    <a href="libra.md#0x0_Libra_publish_burn_capability">Libra::publish_burn_capability</a>&lt;<a href="coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(&new_account, coin1_burn_cap);
    <a href="libra.md#0x0_Libra_publish_mint_capability">Libra::publish_mint_capability</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(&new_account, coin2_mint_cap);
    <a href="libra.md#0x0_Libra_publish_burn_capability">Libra::publish_burn_capability</a>&lt;<a href="coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(&new_account, coin2_burn_cap);

    // TODO: add <a href="association.md#0x0_Association">Association</a> or TreasuryCompliance role instead of using <a href="empty.md#0x0_Empty">Empty</a>?
    <a href="event.md#0x0_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x0_LibraAccount_make_account">make_account</a>&lt;Token, <a href="empty.md#0x0_Empty_T">Empty::T</a>&gt;(new_account, auth_key_prefix, <a href="empty.md#0x0_Empty_create">Empty::create</a>(), <b>false</b>)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_is_designated_dealer"></a>

## Function `is_designated_dealer`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is_designated_dealer">is_designated_dealer</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_is_designated_dealer">is_designated_dealer</a>(addr: address): bool <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a> {
    <b>let</b> dealer =
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="designated_dealer.md#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>&gt;&gt;(addr).role_data;
    <a href="designated_dealer.md#0x0_DesignatedDealer_is_designated_dealer">DesignatedDealer::is_designated_dealer</a>(dealer)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_add_tier"></a>

## Function `add_tier`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_add_tier">add_tier</a>(blessed: &signer, addr: address, tier_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_add_tier">add_tier</a>(blessed: &signer, addr: address, tier_upperbound: u64) <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a> {
    <a href="designated_dealer.md#0x0_DesignatedDealer_assert_account_is_blessed">DesignatedDealer::assert_account_is_blessed</a>(blessed);
    <b>let</b> dealer =
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="designated_dealer.md#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>&gt;&gt;(addr).role_data;
    <a href="designated_dealer.md#0x0_DesignatedDealer_add_tier">DesignatedDealer::add_tier</a>(dealer, tier_upperbound)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_update_tier"></a>

## Function `update_tier`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_update_tier">update_tier</a>(blessed: &signer, addr: address, tier_index: u64, new_upperbound: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_update_tier">update_tier</a>(blessed: &signer, addr: address, tier_index: u64, new_upperbound: u64) <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a> {
    <a href="designated_dealer.md#0x0_DesignatedDealer_assert_account_is_blessed">DesignatedDealer::assert_account_is_blessed</a>(blessed);
    <b>let</b> dealer =
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="designated_dealer.md#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>&gt;&gt;(addr).role_data;
    <a href="designated_dealer.md#0x0_DesignatedDealer_update_tier">DesignatedDealer::update_tier</a>(dealer, tier_index, new_upperbound)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_create_designated_dealer"></a>

## Function `create_designated_dealer`

Create a designated dealer account at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code>, for non synthetic CoinType


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(blessed: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(
    blessed: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
) {
    <a href="designated_dealer.md#0x0_DesignatedDealer_assert_account_is_blessed">DesignatedDealer::assert_account_is_blessed</a>(blessed);
    Transaction::assert(!<a href="libra.md#0x0_Libra_is_synthetic_currency">Libra::is_synthetic_currency</a>&lt;CoinType&gt;(), 202);
    <b>let</b> new_dd_account = <a href="#0x0_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <b>let</b> dealer =
        <a href="designated_dealer.md#0x0_DesignatedDealer_create_designated_dealer">DesignatedDealer::create_designated_dealer</a>();
    <a href="event.md#0x0_Event_publish_generator">Event::publish_generator</a>(&new_dd_account);
    <a href="#0x0_LibraAccount_make_account">make_account</a>&lt;CoinType, <a href="designated_dealer.md#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>&gt;(new_dd_account, auth_key_prefix, dealer, <b>false</b>)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_mint_to_designated_dealer"></a>

## Function `mint_to_designated_dealer`

Tiered Mint called by Treasury Compliance
CoinType should match type called with create_designated_dealer


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_mint_to_designated_dealer">mint_to_designated_dealer</a>&lt;CoinType&gt;(blessed: &signer, dealer_address: address, amount: u64, tier: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_mint_to_designated_dealer">mint_to_designated_dealer</a>&lt;CoinType&gt;(blessed: &signer, dealer_address: address, amount: u64, tier: u64
) <b>acquires</b> <a href="#0x0_LibraAccount_Role">Role</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_T">T</a> {
    <a href="designated_dealer.md#0x0_DesignatedDealer_assert_account_is_blessed">DesignatedDealer::assert_account_is_blessed</a>(blessed);
    // INVALID_MINT_AMOUNT
    Transaction::assert(amount &gt; 0, 6);
    <b>let</b> dealer = &<b>mut</b> borrow_global_mut&lt;<a href="#0x0_LibraAccount_Role">Role</a>&lt;<a href="designated_dealer.md#0x0_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>&gt;&gt;(dealer_address).role_data;
    // NOT_A_DD
    Transaction::assert(<a href="designated_dealer.md#0x0_DesignatedDealer_is_designated_dealer">DesignatedDealer::is_designated_dealer</a>(dealer), 1);
    <b>let</b> tier_check = <a href="designated_dealer.md#0x0_DesignatedDealer_tiered_mint">DesignatedDealer::tiered_mint</a>(dealer, amount, tier);
    // INVALID_AMOUNT_FOR_TIER
    Transaction::assert(tier_check, 5);
    <b>let</b> coins = <a href="libra.md#0x0_Libra_mint">Libra::mint</a>&lt;CoinType&gt;(amount);
    <a href="#0x0_LibraAccount_deposit">deposit</a>(dealer_address, coins);
}
</code></pre>



</details>

<a name="0x0_LibraAccount_create_parent_vasp_account"></a>

## Function `create_parent_vasp_account`

Create an account with the ParentVASP role at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code>.  If
<code>add_all_currencies</code> is true, 0 balances for
all available currencies in the system will also be added.
This can only be invoked by an Association account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_parent_vasp_account">create_parent_vasp_account</a>&lt;Token&gt;(new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_parent_vasp_account">create_parent_vasp_account</a>&lt;Token&gt;(
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;,
    add_all_currencies: bool
) {
    <a href="association.md#0x0_Association_assert_sender_is_association">Association::assert_sender_is_association</a>();
    <b>let</b> vasp_parent =
        <a href="vasp.md#0x0_VASP_create_parent_vasp_credential">VASP::create_parent_vasp_credential</a>(human_name, base_url, compliance_public_key);
    <b>let</b> new_account = <a href="#0x0_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="event.md#0x0_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x0_LibraAccount_make_account">make_account</a>&lt;Token, <a href="vasp.md#0x0_VASP_ParentVASP">VASP::ParentVASP</a>&gt;(
        new_account, auth_key_prefix, vasp_parent, add_all_currencies
    )
}
</code></pre>



</details>

<a name="0x0_LibraAccount_create_child_vasp_account"></a>

## Function `create_child_vasp_account`

Create an account with the ChildVASP role at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code> and a 0 balance of type
<code>Token</code>. If
<code>add_all_currencies</code> is true, 0 balances for all avaialable currencies in the system will
also be added. This account will be a child of
<code>creator</code>, which must be a ParentVASP.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_child_vasp_account">create_child_vasp_account</a>&lt;Token&gt;(creator: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_child_vasp_account">create_child_vasp_account</a>&lt;Token&gt;(
    creator: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool,
) {
    <b>let</b> child_vasp = <a href="vasp.md#0x0_VASP_create_child_vasp">VASP::create_child_vasp</a>(creator);
    <b>let</b> new_account = <a href="#0x0_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="event.md#0x0_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x0_LibraAccount_make_account">make_account</a>&lt;Token, <a href="vasp.md#0x0_VASP_ChildVASP">VASP::ChildVASP</a>&gt;(
        new_account, auth_key_prefix, child_vasp, add_all_currencies
    )
}
</code></pre>



</details>

<a name="0x0_LibraAccount_create_unhosted_account"></a>

## Function `create_unhosted_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_unhosted_account">create_unhosted_account</a>&lt;Token&gt;(new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_create_unhosted_account">create_unhosted_account</a>&lt;Token&gt;(
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool
) {
    <b>let</b> unhosted = <a href="unhosted.md#0x0_Unhosted_create">Unhosted::create</a>();
    <b>let</b> new_account = <a href="#0x0_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="event.md#0x0_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x0_LibraAccount_make_account">make_account</a>&lt;Token, <a href="unhosted.md#0x0_Unhosted_T">Unhosted::T</a>&gt;(new_account, auth_key_prefix, unhosted, add_all_currencies)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_create_signer"></a>

## Function `create_signer`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_create_signer">create_signer</a>(addr: address): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x0_LibraAccount_create_signer">create_signer</a>(addr: address): signer;
</code></pre>



</details>

<a name="0x0_LibraAccount_destroy_signer"></a>

## Function `destroy_signer`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_destroy_signer">destroy_signer</a>(sig: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x0_LibraAccount_destroy_signer">destroy_signer</a>(sig: signer);
</code></pre>



</details>

<a name="0x0_LibraAccount_balance_for"></a>

## Function `balance_for`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_balance_for">balance_for</a>&lt;Token&gt;(balance: &<a href="#0x0_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_balance_for">balance_for</a>&lt;Token&gt;(balance: &<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;): u64 {
    <a href="libra.md#0x0_Libra_value">Libra::value</a>&lt;Token&gt;(&balance.coin)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_balance"></a>

## Function `balance`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_balance">balance</a>&lt;Token&gt;(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_balance">balance</a>&lt;Token&gt;(addr: address): u64 <b>acquires</b> <a href="#0x0_LibraAccount_Balance">Balance</a> {
    <a href="#0x0_LibraAccount_balance_for">balance_for</a>(borrow_global&lt;<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr))
}
</code></pre>



</details>

<a name="0x0_LibraAccount_add_currency"></a>

## Function `add_currency`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(account: &signer) {
    move_to(account, <a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;{ coin: <a href="libra.md#0x0_Libra_zero">Libra::zero</a>&lt;Token&gt;() })
}
</code></pre>



</details>

<a name="0x0_LibraAccount_accepts_currency"></a>

## Function `accepts_currency`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_accepts_currency">accepts_currency</a>&lt;Token&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_accepts_currency">accepts_currency</a>&lt;Token&gt;(addr: address): bool {
    ::<a href="#0x0_LibraAccount_exists">exists</a>&lt;<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_sequence_number_for_account"></a>

## Function `sequence_number_for_account`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="#0x0_LibraAccount_T">LibraAccount::T</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="#0x0_LibraAccount_T">T</a>): u64 {
    account.sequence_number
}
</code></pre>



</details>

<a name="0x0_LibraAccount_sequence_number"></a>

## Function `sequence_number`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_sequence_number">sequence_number</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_sequence_number">sequence_number</a>(addr: address): u64 <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    <a href="#0x0_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(borrow_global&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(addr))
}
</code></pre>



</details>

<a name="0x0_LibraAccount_authentication_key"></a>

## Function `authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    *&borrow_global&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(addr).authentication_key
}
</code></pre>



</details>

<a name="0x0_LibraAccount_delegated_key_rotation_capability"></a>

## Function `delegated_key_rotation_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    borrow_global&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(addr).delegated_key_rotation_capability
}
</code></pre>



</details>

<a name="0x0_LibraAccount_delegated_withdrawal_capability"></a>

## Function `delegated_withdrawal_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_delegated_withdrawal_capability">delegated_withdrawal_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_delegated_withdrawal_capability">delegated_withdrawal_capability</a>(addr: address): bool <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    borrow_global&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(addr).delegated_withdrawal_capability
}
</code></pre>



</details>

<a name="0x0_LibraAccount_withdrawal_capability_address"></a>

## Function `withdrawal_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_withdrawal_capability_address">withdrawal_capability_address</a>(cap: &<a href="#0x0_LibraAccount_WithdrawalCapability">LibraAccount::WithdrawalCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_withdrawal_capability_address">withdrawal_capability_address</a>(cap: &<a href="#0x0_LibraAccount_WithdrawalCapability">WithdrawalCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x0_LibraAccount_key_rotation_capability_address"></a>

## Function `key_rotation_capability_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="#0x0_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="#0x0_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x0_LibraAccount_exists"></a>

## Function `exists`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_exists">exists</a>(check_addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_exists">exists</a>(check_addr: address): bool {
    ::<a href="#0x0_LibraAccount_exists">exists</a>&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(check_addr)
}
</code></pre>



</details>

<a name="0x0_LibraAccount_freeze_account"></a>

## Function `freeze_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_freeze_account">freeze_account</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_freeze_account">freeze_account</a>(addr: address)
<b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="#0x0_LibraAccount_assert_can_freeze">assert_can_freeze</a>(Transaction::sender());
    // The root association account cannot be frozen
    Transaction::assert(addr != <a href="association.md#0x0_Association_root_address">Association::root_address</a>(), 14);
    borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(addr).is_frozen = <b>true</b>;
    <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x0_LibraAccount_FreezeAccountEvent">FreezeAccountEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(0xA550C18).freeze_event_handle,
        <a href="#0x0_LibraAccount_FreezeAccountEvent">FreezeAccountEvent</a> {
            initiator_address: Transaction::sender(),
            frozen_address: addr
        },
    );
}
</code></pre>



</details>

<a name="0x0_LibraAccount_unfreeze_account"></a>

## Function `unfreeze_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_unfreeze_account">unfreeze_account</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_unfreeze_account">unfreeze_account</a>(addr: address)
<b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="#0x0_LibraAccount_assert_can_freeze">assert_can_freeze</a>(Transaction::sender());
    borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(addr).is_frozen = <b>false</b>;
    <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x0_LibraAccount_UnfreezeAccountEvent">UnfreezeAccountEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(0xA550C18).unfreeze_event_handle,
        <a href="#0x0_LibraAccount_UnfreezeAccountEvent">UnfreezeAccountEvent</a> {
            initiator_address: Transaction::sender(),
            unfrozen_address: addr
        },
    );
}
</code></pre>



</details>

<a name="0x0_LibraAccount_account_is_frozen"></a>

## Function `account_is_frozen`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_account_is_frozen">account_is_frozen</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraAccount_account_is_frozen">account_is_frozen</a>(addr: address): bool
<b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    borrow_global&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(addr).is_frozen
 }
</code></pre>



</details>

<a name="0x0_LibraAccount_assert_can_freeze"></a>

## Function `assert_can_freeze`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_assert_can_freeze">assert_can_freeze</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_assert_can_freeze">assert_can_freeze</a>(addr: address) {
    Transaction::assert(<a href="association.md#0x0_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x0_LibraAccount_FreezingPrivilege">FreezingPrivilege</a>&gt;(addr), 13);
}
</code></pre>



</details>

<a name="0x0_LibraAccount_prologue"></a>

## Function `prologue`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_prologue">prologue</a>&lt;Token&gt;(txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_prologue">prologue</a>&lt;Token&gt;(
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time: u64,
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a> {
    <b>let</b> transaction_sender = Transaction::sender();

    // FUTURE: Make these error codes sequential
    // Verify that the transaction sender's account exists
    Transaction::assert(<a href="#0x0_LibraAccount_exists">exists</a>(transaction_sender), 5);

    Transaction::assert(!<a href="#0x0_LibraAccount_account_is_frozen">account_is_frozen</a>(transaction_sender), 0);

    // Load the transaction sender's account
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(transaction_sender);

    // Check that the hash of the transaction's <b>public</b> key matches the account's auth key
    Transaction::assert(
        <a href="hash.md#0x0_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) == *&sender_account.authentication_key,
        2
    );

    // Check that the account has enough balance for all of the gas
    <b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
    <b>let</b> balance_amount = <a href="#0x0_LibraAccount_balance">balance</a>&lt;Token&gt;(transaction_sender);
    Transaction::assert(balance_amount &gt;= max_transaction_fee, 6);

    // Check that the transaction sequence number matches the sequence number of the account
    Transaction::assert(txn_sequence_number &gt;= sender_account.sequence_number, 3);
    Transaction::assert(txn_sequence_number == sender_account.sequence_number, 4);
    Transaction::assert(<a href="libra_transaction_timeout.md#0x0_LibraTransactionTimeout_is_valid_transaction_timestamp">LibraTransactionTimeout::is_valid_transaction_timestamp</a>(txn_expiration_time), 7);
}
</code></pre>



</details>

<a name="0x0_LibraAccount_epilogue"></a>

## Function `epilogue`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64
) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a>, <a href="#0x0_LibraAccount_Balance">Balance</a>, <a href="#0x0_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    // Load the transaction sender's account and balance resources
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(Transaction::sender());
    <b>let</b> sender_balance = borrow_global_mut&lt;<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(Transaction::sender());

    // Charge for gas
    <b>let</b> transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);
    Transaction::assert(
        <a href="#0x0_LibraAccount_balance_for">balance_for</a>(sender_balance) &gt;= transaction_fee_amount,
        6
    );
    // Bump the sequence number
    sender_account.sequence_number = txn_sequence_number + 1;

    <b>if</b> (transaction_fee_amount &gt; 0) {
        <b>let</b> transaction_fee = <a href="#0x0_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>(
                Transaction::sender(),
                sender_balance,
                transaction_fee_amount
        );
        // Pay the transaction fee into the transaction fee balance.
        // Don't <b>use</b> the account deposit in order <b>to</b> not emit a
        // sent/received payment event.
        <b>let</b> transaction_fee_balance = borrow_global_mut&lt;<a href="#0x0_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(0xFEE);
        <a href="libra.md#0x0_Libra_deposit">Libra::deposit</a>(&<b>mut</b> transaction_fee_balance.coin, transaction_fee);
    }
}
</code></pre>



</details>

<a name="0x0_LibraAccount_bump_sequence_number"></a>

## Function `bump_sequence_number`



<pre><code><b>fun</b> <a href="#0x0_LibraAccount_bump_sequence_number">bump_sequence_number</a>(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraAccount_bump_sequence_number">bump_sequence_number</a>(signer: &signer) <b>acquires</b> <a href="#0x0_LibraAccount_T">T</a> {
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x0_LibraAccount_T">T</a>&gt;(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(signer));
    sender_account.sequence_number = sender_account.sequence_number + 1;
}
</code></pre>



</details>
