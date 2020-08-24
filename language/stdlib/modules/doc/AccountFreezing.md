
<a name="0x1_AccountFreezing"></a>

# Module `0x1::AccountFreezing`

### Table of Contents

-  [Resource `FreezingBit`](#0x1_AccountFreezing_FreezingBit)
-  [Resource `FreezeEventsHolder`](#0x1_AccountFreezing_FreezeEventsHolder)
-  [Struct `FreezeAccountEvent`](#0x1_AccountFreezing_FreezeAccountEvent)
-  [Struct `UnfreezeAccountEvent`](#0x1_AccountFreezing_UnfreezeAccountEvent)
-  [Const `EFREEZE_EVENTS_HOLDER`](#0x1_AccountFreezing_EFREEZE_EVENTS_HOLDER)
-  [Const `EFREEZING_BIT`](#0x1_AccountFreezing_EFREEZING_BIT)
-  [Const `ECANNOT_FREEZE_LIBRA_ROOT`](#0x1_AccountFreezing_ECANNOT_FREEZE_LIBRA_ROOT)
-  [Const `ECANNOT_FREEZE_TC`](#0x1_AccountFreezing_ECANNOT_FREEZE_TC)
-  [Const `EACCOUNT_FROZEN`](#0x1_AccountFreezing_EACCOUNT_FROZEN)
-  [Function `initialize`](#0x1_AccountFreezing_initialize)
-  [Function `create`](#0x1_AccountFreezing_create)
-  [Function `freeze_account`](#0x1_AccountFreezing_freeze_account)
-  [Function `unfreeze_account`](#0x1_AccountFreezing_unfreeze_account)
-  [Function `account_is_frozen`](#0x1_AccountFreezing_account_is_frozen)
-  [Function `assert_not_frozen`](#0x1_AccountFreezing_assert_not_frozen)
-  [Specification](#0x1_AccountFreezing_Specification)
    -  [Function `initialize`](#0x1_AccountFreezing_Specification_initialize)
    -  [Function `create`](#0x1_AccountFreezing_Specification_create)
    -  [Function `freeze_account`](#0x1_AccountFreezing_Specification_freeze_account)
    -  [Function `unfreeze_account`](#0x1_AccountFreezing_Specification_unfreeze_account)
    -  [Function `account_is_frozen`](#0x1_AccountFreezing_Specification_account_is_frozen)
    -  [Function `assert_not_frozen`](#0x1_AccountFreezing_Specification_assert_not_frozen)



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

<a name="0x1_AccountFreezing_EFREEZE_EVENTS_HOLDER"></a>

## Const `EFREEZE_EVENTS_HOLDER`

A property expected of the
<code><a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a></code> resource didn't hold


<pre><code><b>const</b> EFREEZE_EVENTS_HOLDER: u64 = 1;
</code></pre>



<a name="0x1_AccountFreezing_EFREEZING_BIT"></a>

## Const `EFREEZING_BIT`

The
<code><a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a></code> resource is in an invalid state


<pre><code><b>const</b> EFREEZING_BIT: u64 = 2;
</code></pre>



<a name="0x1_AccountFreezing_ECANNOT_FREEZE_LIBRA_ROOT"></a>

## Const `ECANNOT_FREEZE_LIBRA_ROOT`

An attempt to freeze the Libra Root account was attempted


<pre><code><b>const</b> ECANNOT_FREEZE_LIBRA_ROOT: u64 = 3;
</code></pre>



<a name="0x1_AccountFreezing_ECANNOT_FREEZE_TC"></a>

## Const `ECANNOT_FREEZE_TC`

An attempt to freeze the Treasury & Compliance account was attempted


<pre><code><b>const</b> ECANNOT_FREEZE_TC: u64 = 4;
</code></pre>



<a name="0x1_AccountFreezing_EACCOUNT_FROZEN"></a>

## Const `EACCOUNT_FROZEN`

The account is frozen


<pre><code><b>const</b> EACCOUNT_FROZEN: u64 = 5;
</code></pre>



<a name="0x1_AccountFreezing_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_initialize">initialize</a>(lr_account: &signer) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);
    <b>assert</b>(
        !exists&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(EFREEZE_EVENTS_HOLDER)
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
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(!exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(EFREEZING_BIT));
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
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(account);
    // The libra root account and TC cannot be frozen
    <b>assert</b>(frozen_address != <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(ECANNOT_FREEZE_LIBRA_ROOT));
    <b>assert</b>(frozen_address != <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(ECANNOT_FREEZE_TC));
    <b>assert</b>(exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(frozen_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EFREEZING_BIT));
    borrow_global_mut&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(frozen_address).is_frozen = <b>true</b>;
    <b>let</b> initiator_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
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
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(unfrozen_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(EFREEZING_BIT));
    borrow_global_mut&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(unfrozen_address).is_frozen = <b>false</b>;
    <b>let</b> initiator_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
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

<a name="0x1_AccountFreezing_assert_not_frozen"></a>

## Function `assert_not_frozen`

Assert that an account is not frozen.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_assert_not_frozen">assert_not_frozen</a>(account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_assert_not_frozen">assert_not_frozen</a>(account: address) <b>acquires</b> <a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a> {
    <b>assert</b>(!<a href="#0x1_AccountFreezing_account_is_frozen">account_is_frozen</a>(account), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(EACCOUNT_FROZEN));
}
</code></pre>



</details>

<a name="0x1_AccountFreezing_Specification"></a>

## Specification


<a name="0x1_AccountFreezing_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_initialize">initialize</a>(lr_account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>{account: lr_account};
<a name="0x1_AccountFreezing_addr$8"></a>
<b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account);
<b>aborts_if</b> exists&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(addr) with Errors::ALREADY_PUBLISHED;
<b>ensures</b> exists&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(addr);
</code></pre>



<a name="0x1_AccountFreezing_Specification_create"></a>

### Function `create`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_create">create</a>(account: &signer)
</code></pre>




<a name="0x1_AccountFreezing_addr$9"></a>


<pre><code><b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<b>aborts_if</b> exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr) with Errors::ALREADY_PUBLISHED;
<b>ensures</b> <a href="#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(addr);
</code></pre>



<a name="0x1_AccountFreezing_Specification_freeze_account"></a>

### Function `freeze_account`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_freeze_account">freeze_account</a>(account: &signer, frozen_address: address)
</code></pre>




<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>;
<b>aborts_if</b> frozen_address == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>() with Errors::INVALID_ARGUMENT;
<b>aborts_if</b> frozen_address == <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>() with Errors::INVALID_ARGUMENT;
<b>aborts_if</b> !exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(frozen_address) with Errors::NOT_PUBLISHED;
<b>ensures</b> <a href="#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(frozen_address);
</code></pre>



<a name="0x1_AccountFreezing_Specification_unfreeze_account"></a>

### Function `unfreeze_account`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_unfreeze_account">unfreeze_account</a>(account: &signer, unfrozen_address: address)
</code></pre>




<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>;
<b>aborts_if</b> !exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(unfrozen_address) with Errors::NOT_PUBLISHED;
<b>ensures</b> !<a href="#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(unfrozen_address);
</code></pre>



<a name="0x1_AccountFreezing_Specification_account_is_frozen"></a>

### Function `account_is_frozen`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_account_is_frozen">account_is_frozen</a>(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
pragma opaque = <b>true</b>;
<b>ensures</b> result == <a href="#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(addr);
</code></pre>



<a name="0x1_AccountFreezing_Specification_assert_not_frozen"></a>

### Function `assert_not_frozen`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_AccountFreezing_assert_not_frozen">assert_not_frozen</a>(account: address)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_AccountFreezing_AbortsIfFrozen">AbortsIfFrozen</a>;
</code></pre>




<a name="0x1_AccountFreezing_AbortsIfFrozen"></a>


<pre><code><b>schema</b> <a href="#0x1_AccountFreezing_AbortsIfFrozen">AbortsIfFrozen</a> {
    account: address;
    <b>aborts_if</b> <a href="#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(account) with Errors::INVALID_STATE;
}
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


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
    exists&lt;<a href="#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>


The account of LibraRoot is not freezable [G2].
After genesis, FreezingBit of LibraRoot is always false.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
    <a href="#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>


The account of TreasuryCompliance is not freezable [G3].
After genesis, FreezingBit of TreasuryCompliance is always false.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
    <a href="#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>());
</code></pre>


The permission "{Freeze,Unfreeze}Account" is granted to TreasuryCompliance [B16].


<pre><code><b>apply</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a> <b>to</b> freeze_account, unfreeze_account;
</code></pre>




<a name="0x1_AccountFreezing_FreezingBitRemainsSame"></a>

The freezing bit stays constant.


<pre><code><b>schema</b> <a href="#0x1_AccountFreezing_FreezingBitRemainsSame">FreezingBitRemainsSame</a> {
    <b>ensures</b> forall a: address where <b>old</b>(exists&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(a)):
        <b>global</b>&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(a).is_frozen == <b>old</b>(<b>global</b>&lt;<a href="#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(a).is_frozen);
}
</code></pre>



only (un)freeze functions can change the freezing bits of accounts [B16].


<pre><code><b>apply</b> <a href="#0x1_AccountFreezing_FreezingBitRemainsSame">FreezingBitRemainsSame</a> <b>to</b> * <b>except</b> freeze_account, unfreeze_account;
</code></pre>
