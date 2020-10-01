
<a name="0x1_AccountFreezing"></a>

# Module `0x1::AccountFreezing`

Module which manages freezing of accounts.


-  [Resource <code><a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a></code>](#0x1_AccountFreezing_FreezingBit)
-  [Resource <code><a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a></code>](#0x1_AccountFreezing_FreezeEventsHolder)
-  [Struct <code><a href="AccountFreezing.md#0x1_AccountFreezing_FreezeAccountEvent">FreezeAccountEvent</a></code>](#0x1_AccountFreezing_FreezeAccountEvent)
-  [Struct <code><a href="AccountFreezing.md#0x1_AccountFreezing_UnfreezeAccountEvent">UnfreezeAccountEvent</a></code>](#0x1_AccountFreezing_UnfreezeAccountEvent)
-  [Const <code><a href="AccountFreezing.md#0x1_AccountFreezing_EFREEZE_EVENTS_HOLDER">EFREEZE_EVENTS_HOLDER</a></code>](#0x1_AccountFreezing_EFREEZE_EVENTS_HOLDER)
-  [Const <code><a href="AccountFreezing.md#0x1_AccountFreezing_EFREEZING_BIT">EFREEZING_BIT</a></code>](#0x1_AccountFreezing_EFREEZING_BIT)
-  [Const <code><a href="AccountFreezing.md#0x1_AccountFreezing_ECANNOT_FREEZE_LIBRA_ROOT">ECANNOT_FREEZE_LIBRA_ROOT</a></code>](#0x1_AccountFreezing_ECANNOT_FREEZE_LIBRA_ROOT)
-  [Const <code><a href="AccountFreezing.md#0x1_AccountFreezing_ECANNOT_FREEZE_TC">ECANNOT_FREEZE_TC</a></code>](#0x1_AccountFreezing_ECANNOT_FREEZE_TC)
-  [Const <code><a href="AccountFreezing.md#0x1_AccountFreezing_EACCOUNT_FROZEN">EACCOUNT_FROZEN</a></code>](#0x1_AccountFreezing_EACCOUNT_FROZEN)
-  [Function <code>initialize</code>](#0x1_AccountFreezing_initialize)
-  [Function <code>create</code>](#0x1_AccountFreezing_create)
-  [Function <code>freeze_account</code>](#0x1_AccountFreezing_freeze_account)
-  [Function <code>unfreeze_account</code>](#0x1_AccountFreezing_unfreeze_account)
-  [Function <code>account_is_frozen</code>](#0x1_AccountFreezing_account_is_frozen)
-  [Function <code>assert_not_frozen</code>](#0x1_AccountFreezing_assert_not_frozen)
-  [Module Specification](#@Module_Specification_0)
    -  [Initialization](#@Initialization_1)
    -  [Access Control](#@Access_Control_2)
    -  [Helper Functions](#@Helper_Functions_3)


<a name="0x1_AccountFreezing_FreezingBit"></a>

## Resource `FreezingBit`



<pre><code><b>resource</b> <b>struct</b> <a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>is_frozen: bool</code>
</dt>
<dd>
 If <code>is_frozen</code> is set true, the account cannot be used to send transactions or receive funds
</dd>
</dl>


</details>

<a name="0x1_AccountFreezing_FreezeEventsHolder"></a>

## Resource `FreezeEventsHolder`



<pre><code><b>resource</b> <b>struct</b> <a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>freeze_event_handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezeAccountEvent">AccountFreezing::FreezeAccountEvent</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>unfreeze_event_handle: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_UnfreezeAccountEvent">AccountFreezing::UnfreezeAccountEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_AccountFreezing_FreezeAccountEvent"></a>

## Struct `FreezeAccountEvent`

Message for freeze account events


<pre><code><b>struct</b> <a href="AccountFreezing.md#0x1_AccountFreezing_FreezeAccountEvent">FreezeAccountEvent</a>
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


<pre><code><b>struct</b> <a href="AccountFreezing.md#0x1_AccountFreezing_UnfreezeAccountEvent">UnfreezeAccountEvent</a>
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

A property expected of the <code><a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a></code> resource didn't hold


<pre><code><b>const</b> <a href="AccountFreezing.md#0x1_AccountFreezing_EFREEZE_EVENTS_HOLDER">EFREEZE_EVENTS_HOLDER</a>: u64 = 1;
</code></pre>



<a name="0x1_AccountFreezing_EFREEZING_BIT"></a>

## Const `EFREEZING_BIT`

The <code><a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="AccountFreezing.md#0x1_AccountFreezing_EFREEZING_BIT">EFREEZING_BIT</a>: u64 = 2;
</code></pre>



<a name="0x1_AccountFreezing_ECANNOT_FREEZE_LIBRA_ROOT"></a>

## Const `ECANNOT_FREEZE_LIBRA_ROOT`

An attempt to freeze the Libra Root account was attempted


<pre><code><b>const</b> <a href="AccountFreezing.md#0x1_AccountFreezing_ECANNOT_FREEZE_LIBRA_ROOT">ECANNOT_FREEZE_LIBRA_ROOT</a>: u64 = 3;
</code></pre>



<a name="0x1_AccountFreezing_ECANNOT_FREEZE_TC"></a>

## Const `ECANNOT_FREEZE_TC`

An attempt to freeze the Treasury & Compliance account was attempted


<pre><code><b>const</b> <a href="AccountFreezing.md#0x1_AccountFreezing_ECANNOT_FREEZE_TC">ECANNOT_FREEZE_TC</a>: u64 = 4;
</code></pre>



<a name="0x1_AccountFreezing_EACCOUNT_FROZEN"></a>

## Const `EACCOUNT_FROZEN`

The account is frozen


<pre><code><b>const</b> <a href="AccountFreezing.md#0x1_AccountFreezing_EACCOUNT_FROZEN">EACCOUNT_FROZEN</a>: u64 = 5;
</code></pre>



<a name="0x1_AccountFreezing_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_initialize">initialize</a>(lr_account: &signer) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);
    <b>assert</b>(
        !<b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="AccountFreezing.md#0x1_AccountFreezing_EFREEZE_EVENTS_HOLDER">EFREEZE_EVENTS_HOLDER</a>)
    );
    move_to(lr_account, <a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a> {
        freeze_event_handle: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>(lr_account),
        unfreeze_event_handle: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>(lr_account),
    });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>{account: lr_account};
<a name="0x1_AccountFreezing_addr$8"></a>
<b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account);
<b>aborts_if</b> <b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
<b>ensures</b> <b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(addr);
</code></pre>



</details>

<a name="0x1_AccountFreezing_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_create">create</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_create">create</a>(account: &signer) {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(!<b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="AccountFreezing.md#0x1_AccountFreezing_EFREEZING_BIT">EFREEZING_BIT</a>));
    move_to(account, <a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a> { is_frozen: <b>false</b> })
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<a name="0x1_AccountFreezing_addr$9"></a>


<pre><code><b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
<b>aborts_if</b> <b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
<b>ensures</b> <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(addr);
</code></pre>



</details>

<a name="0x1_AccountFreezing_freeze_account"></a>

## Function `freeze_account`

Freeze the account at <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_freeze_account">freeze_account</a>(account: &signer, frozen_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_freeze_account">freeze_account</a>(
    account: &signer,
    frozen_address: address,
)
<b>acquires</b> <a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>, <a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(account);
    // The libra root account and TC cannot be frozen
    <b>assert</b>(frozen_address != <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="AccountFreezing.md#0x1_AccountFreezing_ECANNOT_FREEZE_LIBRA_ROOT">ECANNOT_FREEZE_LIBRA_ROOT</a>));
    <b>assert</b>(frozen_address != <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="AccountFreezing.md#0x1_AccountFreezing_ECANNOT_FREEZE_TC">ECANNOT_FREEZE_TC</a>));
    <b>assert</b>(<b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(frozen_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="AccountFreezing.md#0x1_AccountFreezing_EFREEZING_BIT">EFREEZING_BIT</a>));
    borrow_global_mut&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(frozen_address).is_frozen = <b>true</b>;
    <b>let</b> initiator_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezeAccountEvent">FreezeAccountEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).freeze_event_handle,
        <a href="AccountFreezing.md#0x1_AccountFreezing_FreezeAccountEvent">FreezeAccountEvent</a> {
            initiator_address,
            frozen_address
        },
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>;
<b>aborts_if</b> frozen_address == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>() <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>aborts_if</b> frozen_address == <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>() <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(frozen_address) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
<b>ensures</b> <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(frozen_address);
</code></pre>



</details>

<a name="0x1_AccountFreezing_unfreeze_account"></a>

## Function `unfreeze_account`

Unfreeze the account at <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_unfreeze_account">unfreeze_account</a>(account: &signer, unfrozen_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_unfreeze_account">unfreeze_account</a>(
    account: &signer,
    unfrozen_address: address,
)
<b>acquires</b> <a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>, <a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(account);
    <b>assert</b>(<b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(unfrozen_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="AccountFreezing.md#0x1_AccountFreezing_EFREEZING_BIT">EFREEZING_BIT</a>));
    borrow_global_mut&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(unfrozen_address).is_frozen = <b>false</b>;
    <b>let</b> initiator_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_UnfreezeAccountEvent">UnfreezeAccountEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).unfreeze_event_handle,
        <a href="AccountFreezing.md#0x1_AccountFreezing_UnfreezeAccountEvent">UnfreezeAccountEvent</a> {
            initiator_address,
            unfrozen_address
        },
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(unfrozen_address) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
<b>ensures</b> !<a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(unfrozen_address);
</code></pre>



</details>

<a name="0x1_AccountFreezing_account_is_frozen"></a>

## Function `account_is_frozen`

Returns if the account at <code>addr</code> is frozen.


<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">account_is_frozen</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">account_is_frozen</a>(addr: address): bool
<b>acquires</b> <a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a> {
    <b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr) && borrow_global&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr).is_frozen
 }
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> <b>false</b>;
<b>pragma</b> opaque = <b>true</b>;
<b>ensures</b> result == <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(addr);
</code></pre>



</details>

<a name="0x1_AccountFreezing_assert_not_frozen"></a>

## Function `assert_not_frozen`

Assert that an account is not frozen.


<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_assert_not_frozen">assert_not_frozen</a>(account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="AccountFreezing.md#0x1_AccountFreezing_assert_not_frozen">assert_not_frozen</a>(account: address) <b>acquires</b> <a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a> {
    <b>assert</b>(!<a href="AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">account_is_frozen</a>(account), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="AccountFreezing.md#0x1_AccountFreezing_EACCOUNT_FROZEN">EACCOUNT_FROZEN</a>));
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="AccountFreezing.md#0x1_AccountFreezing_AbortsIfFrozen">AbortsIfFrozen</a>;
</code></pre>




<a name="0x1_AccountFreezing_AbortsIfFrozen"></a>


<pre><code><b>schema</b> <a href="AccountFreezing.md#0x1_AccountFreezing_AbortsIfFrozen">AbortsIfFrozen</a> {
    account: address;
    <b>aborts_if</b> <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(account) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



<a name="@Initialization_1"></a>

### Initialization


<code><a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a></code> always exists after genesis.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
    <b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezeEventsHolder">FreezeEventsHolder</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>



<a name="@Access_Control_2"></a>

### Access Control


The account of LibraRoot is not freezable [[F1]][ROLE].
After genesis, FreezingBit of LibraRoot is always false.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
    <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>


The account of TreasuryCompliance is not freezable [[F2]][ROLE].
After genesis, FreezingBit of TreasuryCompliance is always false.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
    <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>());
</code></pre>


The permission "{Freeze,Unfreeze}Account" is granted to TreasuryCompliance only [[H6]][PERMISSION].


<pre><code><b>apply</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a> <b>to</b> freeze_account, unfreeze_account;
</code></pre>


Only (un)freeze functions can change the freezing bits of accounts [[H6]][PERMISSION].


<pre><code><b>apply</b> <a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBitRemainsSame">FreezingBitRemainsSame</a> <b>to</b> * <b>except</b> freeze_account, unfreeze_account;
</code></pre>




<a name="0x1_AccountFreezing_FreezingBitRemainsSame"></a>


<pre><code><b>schema</b> <a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBitRemainsSame">FreezingBitRemainsSame</a> {
    <b>ensures</b> <b>forall</b> a: address <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(a)):
        <b>global</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(a).is_frozen == <b>old</b>(<b>global</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(a).is_frozen);
}
</code></pre>



<a name="@Helper_Functions_3"></a>

### Helper Functions



<a name="0x1_AccountFreezing_spec_account_is_frozen"></a>


<pre><code><b>define</b> <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_frozen">spec_account_is_frozen</a>(addr: address): bool {
    <b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr) && <b>global</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr).is_frozen
}
<a name="0x1_AccountFreezing_spec_account_is_not_frozen"></a>
<b>define</b> <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_not_frozen">spec_account_is_not_frozen</a>(addr: address): bool {
    <b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr) && !<b>global</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">FreezingBit</a>&gt;(addr).is_frozen
}
</code></pre>
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
