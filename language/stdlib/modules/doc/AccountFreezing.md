
<a name="0x1_AccountFreezing"></a>

# Module `0x1::AccountFreezing`

### Table of Contents

-  [Resource `FreezingBit`](#0x1_AccountFreezing_FreezingBit)
-  [Resource `FreezeEventsHolder`](#0x1_AccountFreezing_FreezeEventsHolder)
-  [Struct `FreezeAccountEvent`](#0x1_AccountFreezing_FreezeAccountEvent)
-  [Struct `UnfreezeAccountEvent`](#0x1_AccountFreezing_UnfreezeAccountEvent)
-  [Function `initialize`](#0x1_AccountFreezing_initialize)
-  [Function `create`](#0x1_AccountFreezing_create)
-  [Function `freeze_account`](#0x1_AccountFreezing_freeze_account)
-  [Function `unfreeze_account`](#0x1_AccountFreezing_unfreeze_account)
-  [Function `account_is_frozen`](#0x1_AccountFreezing_account_is_frozen)
-  [Specification](#0x1_AccountFreezing_Specification)
    -  [Function `initialize`](#0x1_AccountFreezing_Specification_initialize)
    -  [Function `create`](#0x1_AccountFreezing_Specification_create)
    -  [Function `freeze_account`](#0x1_AccountFreezing_Specification_freeze_account)
    -  [Function `unfreeze_account`](#0x1_AccountFreezing_Specification_unfreeze_account)
    -  [Function `account_is_frozen`](#0x1_AccountFreezing_Specification_account_is_frozen)



<a name="0x1_AccountFreezing_FreezingBit"></a>

## Resource `FreezingBit`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>is_frozen: bool</code>
</dt>
<dd>
 If
<code>is_frozen</code> is set true, the account cannot be used to send transactions or receive funds
</dd>
</dl>


</details>

<a name="0x1_AccountFreezing_FreezeEventsHolder"></a>

## Resource `FreezeEventsHolder`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>freeze_event_handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_AccountFreezing_FreezeAccountEvent">AccountFreezing::FreezeAccountEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>unfreeze_event_handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_AccountFreezing_UnfreezeAccountEvent">AccountFreezing::UnfreezeAccountEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_AccountFreezing_FreezeAccountEvent"></a>

## Struct `FreezeAccountEvent`

Message for freeze account events


<pre><code><b>struct</b> <a href="#0x1_AccountFreezing_FreezeAccountEvent">FreezeAccountEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>initiator_address: address</code>
</dt>
<dd>
 The address that initiated freeze txn
</dd>
<dt>

<code>frozen_address: address</code>
</dt>
<dd>
 The address that was frozen
</dd>
</dl>


</details>

<a name="0x1_AccountFreezing_UnfreezeAccountEvent"></a>

## Struct `UnfreezeAccountEvent`

Message for unfreeze account events


<pre><code><b>struct</b> <a href="#0x1_AccountFreezing_UnfreezeAccountEvent">UnfreezeAccountEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>initiator_address: address</code>
</dt>
<dd>
 The address that initiated unfreeze txn
</dd>
<dt>

<code>unfrozen_address: address</code>
</dt>
<dd>
 The address that was unfrozen
</dd>
</dl>


</details>

<a name="0x1_AccountFreezing_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_initialize">initialize</a>(lr_account: &signer) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(),
        EINVALID_SINGLETON_ADDRESS
    );
    move_to(lr_account, <a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a> {
        freeze_event_handle: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>(lr_account),
        unfreeze_event_handle: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>(lr_account),
    });
}
</code></pre>



</details>

<a name="0x1_AccountFreezing_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_create">create</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_create">create</a>(account: &signer) {
    move_to(account, <a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a> { is_frozen: <b>false</b> })
}
</code></pre>



</details>

<a name="0x1_AccountFreezing_freeze_account"></a>

## Function `freeze_account`

Freeze the account at
<code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_freeze_account">freeze_account</a>(account: &signer, frozen_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_freeze_account">freeze_account</a>(
    account: &signer,
    frozen_address: address,
)
<b>acquires</b> <a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>, <a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a> {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(account), ENOT_ABLE_TO_FREEZE);
    <b>let</b> initiator_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // The libra root account and TC cannot be frozen
    <b>assert</b>(frozen_address != <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), ECANNOT_FREEZE_LIBRA_ROOT);
    <b>assert</b>(frozen_address != <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>(), ECANNOT_FREEZE_TC);
    borrow_global_mut&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(frozen_address).is_frozen = <b>true</b>;
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_AccountFreezing_FreezeAccountEvent">FreezeAccountEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).freeze_event_handle,
        <a href="#0x1_AccountFreezing_FreezeAccountEvent">FreezeAccountEvent</a> {
            initiator_address,
            frozen_address
        },
    );
}
</code></pre>



</details>

<a name="0x1_AccountFreezing_unfreeze_account"></a>

## Function `unfreeze_account`

Unfreeze the account at
<code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_unfreeze_account">unfreeze_account</a>(account: &signer, unfrozen_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_unfreeze_account">unfreeze_account</a>(
    account: &signer,
    unfrozen_address: address,
)
<b>acquires</b> <a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>, <a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a> {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_treasury_compliance_role">Roles::has_treasury_compliance_role</a>(account), ENOT_ABLE_TO_UNFREEZE);
    <b>let</b> initiator_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    borrow_global_mut&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(unfrozen_address).is_frozen = <b>false</b>;
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_AccountFreezing_UnfreezeAccountEvent">UnfreezeAccountEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).unfreeze_event_handle,
        <a href="#0x1_AccountFreezing_UnfreezeAccountEvent">UnfreezeAccountEvent</a> {
            initiator_address,
            unfrozen_address
        },
    );
}
</code></pre>



</details>

<a name="0x1_AccountFreezing_account_is_frozen"></a>

## Function `account_is_frozen`

Returns if the account at
<code>addr</code> is frozen.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_account_is_frozen">account_is_frozen</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_account_is_frozen">account_is_frozen</a>(addr: address): bool
<b>acquires</b> <a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a> {
    exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr) && borrow_global&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr).is_frozen
 }
</code></pre>



</details>

<a name="0x1_AccountFreezing_Specification"></a>

## Specification


<a name="0x1_AccountFreezing_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_initialize">initialize</a>(lr_account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_is_genesis">LibraTimestamp::spec_is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>();
<b>aborts_if</b> exists&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account));
<b>ensures</b> exists&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account));
</code></pre>



<a name="0x1_AccountFreezing_Specification_create"></a>

### Function `create`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_create">create</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <a href="#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="0x1_AccountFreezing_Specification_freeze_account"></a>

### Function `freeze_account`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_freeze_account">freeze_account</a>(account: &signer, frozen_address: address)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_treasury_compliance_role">Roles::spec_has_treasury_compliance_role</a>(account);
<b>aborts_if</b> frozen_address == <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>();
<b>aborts_if</b> frozen_address == <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::SPEC_TREASURY_COMPLIANCE_ADDRESS</a>();
<b>aborts_if</b> !exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(frozen_address);
<b>aborts_if</b> !exists&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>());
<b>ensures</b> <a href="#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(frozen_address);
</code></pre>



<a name="0x1_AccountFreezing_Specification_unfreeze_account"></a>

### Function `unfreeze_account`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_unfreeze_account">unfreeze_account</a>(account: &signer, unfrozen_address: address)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_treasury_compliance_role">Roles::spec_has_treasury_compliance_role</a>(account);
<b>aborts_if</b> !exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(unfrozen_address);
<b>aborts_if</b> !exists&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>());
<b>ensures</b> !<a href="#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(unfrozen_address);
</code></pre>



<a name="0x1_AccountFreezing_Specification_account_is_frozen"></a>

### Function `account_is_frozen`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_account_is_frozen">account_is_frozen</a>(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(addr);
</code></pre>




<pre><code>pragma verify = <b>true</b>;
<a name="0x1_AccountFreezing_spec_account_is_frozen"></a>
<b>define</b> <a href="#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(addr: address): bool {
    exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr) && <b>global</b>&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr).is_frozen
}
<a name="0x1_AccountFreezing_spec_account_is_not_frozen"></a>
<b>define</b> <a href="#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(addr: address): bool {
    exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr) && !<b>global</b>&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr).is_frozen
}
</code></pre>


FreezeEventsHolder always exists after genesis.


<pre><code><b>invariant</b> !<a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_is_genesis">LibraTimestamp::spec_is_genesis</a>() ==&gt;
    exists&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>());
</code></pre>


The account of LibraRoot is not freezable [G2].
After genesis, FreezingBit of LibraRoot is always false.


<pre><code><b>invariant</b> !<a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_is_genesis">LibraTimestamp::spec_is_genesis</a>() ==&gt;
    <a href="#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>());
</code></pre>


The account of TreasuryCompliance is not freezable [G3].
After genesis, FreezingBit of TreasuryCompliance is always false.


<pre><code><b>invariant</b> !<a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_is_genesis">LibraTimestamp::spec_is_genesis</a>() ==&gt;
    <a href="#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::SPEC_TREASURY_COMPLIANCE_ADDRESS</a>());
</code></pre>
