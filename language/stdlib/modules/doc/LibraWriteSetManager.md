
<a name="0x1_LibraWriteSetManager"></a>

# Module `0x1::LibraWriteSetManager`

### Table of Contents

-  [Resource `LibraWriteSetManager`](#0x1_LibraWriteSetManager_LibraWriteSetManager)
-  [Struct `UpgradeEvent`](#0x1_LibraWriteSetManager_UpgradeEvent)
-  [Const `ELIBRA_WRITE_SET_MANAGER`](#0x1_LibraWriteSetManager_ELIBRA_WRITE_SET_MANAGER)
-  [Const `PROLOGUE_EINVALID_WRITESET_SENDER`](#0x1_LibraWriteSetManager_PROLOGUE_EINVALID_WRITESET_SENDER)
-  [Const `PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY`](#0x1_LibraWriteSetManager_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY)
-  [Const `PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD`](#0x1_LibraWriteSetManager_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD)
-  [Const `PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW`](#0x1_LibraWriteSetManager_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW)
-  [Function `initialize`](#0x1_LibraWriteSetManager_initialize)
-  [Function `prologue`](#0x1_LibraWriteSetManager_prologue)
-  [Function `epilogue`](#0x1_LibraWriteSetManager_epilogue)
-  [Specification](#0x1_LibraWriteSetManager_Specification)
    -  [Function `initialize`](#0x1_LibraWriteSetManager_Specification_initialize)
    -  [Function `prologue`](#0x1_LibraWriteSetManager_Specification_prologue)



<a name="0x1_LibraWriteSetManager_LibraWriteSetManager"></a>

## Resource `LibraWriteSetManager`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraWriteSetManager">LibraWriteSetManager</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>upgrade_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraWriteSetManager_UpgradeEvent">LibraWriteSetManager::UpgradeEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraWriteSetManager_UpgradeEvent"></a>

## Struct `UpgradeEvent`



<pre><code><b>struct</b> <a href="#0x1_LibraWriteSetManager_UpgradeEvent">UpgradeEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>writeset_payload: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraWriteSetManager_ELIBRA_WRITE_SET_MANAGER"></a>

## Const `ELIBRA_WRITE_SET_MANAGER`

The <code><a href="#0x1_LibraWriteSetManager">LibraWriteSetManager</a></code> was not in the required state


<pre><code><b>const</b> <a href="#0x1_LibraWriteSetManager_ELIBRA_WRITE_SET_MANAGER">ELIBRA_WRITE_SET_MANAGER</a>: u64 = 0;
</code></pre>



<a name="0x1_LibraWriteSetManager_PROLOGUE_EINVALID_WRITESET_SENDER"></a>

## Const `PROLOGUE_EINVALID_WRITESET_SENDER`



<pre><code><b>const</b> <a href="#0x1_LibraWriteSetManager_PROLOGUE_EINVALID_WRITESET_SENDER">PROLOGUE_EINVALID_WRITESET_SENDER</a>: u64 = 1033;
</code></pre>



<a name="0x1_LibraWriteSetManager_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY"></a>

## Const `PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY`



<pre><code><b>const</b> <a href="#0x1_LibraWriteSetManager_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY">PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY</a>: u64 = 1001;
</code></pre>



<a name="0x1_LibraWriteSetManager_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD"></a>

## Const `PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD`



<pre><code><b>const</b> <a href="#0x1_LibraWriteSetManager_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD">PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD</a>: u64 = 1002;
</code></pre>



<a name="0x1_LibraWriteSetManager_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW"></a>

## Const `PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW`



<pre><code><b>const</b> <a href="#0x1_LibraWriteSetManager_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW">PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW</a>: u64 = 1011;
</code></pre>



<a name="0x1_LibraWriteSetManager_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraWriteSetManager_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraWriteSetManager_initialize">initialize</a>(account: &signer) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    // Operational constraint
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(account);

    <b>assert</b>(
        !exists&lt;<a href="#0x1_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="#0x1_LibraWriteSetManager_ELIBRA_WRITE_SET_MANAGER">ELIBRA_WRITE_SET_MANAGER</a>)
    );
    move_to(
        account,
        <a href="#0x1_LibraWriteSetManager">LibraWriteSetManager</a> {
            upgrade_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_LibraWriteSetManager_UpgradeEvent">Self::UpgradeEvent</a>&gt;(account),
        }
    );
}
</code></pre>



</details>

<a name="0x1_LibraWriteSetManager_prologue"></a>

## Function `prologue`



<pre><code><b>fun</b> <a href="#0x1_LibraWriteSetManager_prologue">prologue</a>(account: &signer, writeset_sequence_number: u64, writeset_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraWriteSetManager_prologue">prologue</a>(
    account: &signer,
    writeset_sequence_number: u64,
    writeset_public_key: vector&lt;u8&gt;,
) {
    // The below code uses direct <b>abort</b> codes <b>as</b> per contract with VM.
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(
        sender == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="#0x1_LibraWriteSetManager_PROLOGUE_EINVALID_WRITESET_SENDER">PROLOGUE_EINVALID_WRITESET_SENDER</a>)
    );
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account), <a href="#0x1_LibraWriteSetManager_PROLOGUE_EINVALID_WRITESET_SENDER">PROLOGUE_EINVALID_WRITESET_SENDER</a>);

    <b>let</b> lr_auth_key = <a href="LibraAccount.md#0x1_LibraAccount_authentication_key">LibraAccount::authentication_key</a>(sender);
    <b>let</b> sequence_number = <a href="LibraAccount.md#0x1_LibraAccount_sequence_number">LibraAccount::sequence_number</a>(sender);

    <b>assert</b>(
        writeset_sequence_number &gt;= sequence_number,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="#0x1_LibraWriteSetManager_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD">PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD</a>)
    );

    <b>assert</b>(
        writeset_sequence_number == sequence_number,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="#0x1_LibraWriteSetManager_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW">PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW</a>)
    );
    <b>assert</b>(
        <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(writeset_public_key) == lr_auth_key,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="#0x1_LibraWriteSetManager_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY">PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY</a>)
    );
}
</code></pre>



</details>

<a name="0x1_LibraWriteSetManager_epilogue"></a>

## Function `epilogue`



<pre><code><b>fun</b> <a href="#0x1_LibraWriteSetManager_epilogue">epilogue</a>(lr_account: &signer, writeset_payload: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraWriteSetManager_epilogue">epilogue</a>(lr_account: &signer, writeset_payload: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x1_LibraWriteSetManager">LibraWriteSetManager</a> {
    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x1_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());

    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraWriteSetManager_UpgradeEvent">Self::UpgradeEvent</a>&gt;(
        &<b>mut</b> t_ref.upgrade_events,
        <a href="#0x1_LibraWriteSetManager_UpgradeEvent">UpgradeEvent</a> { writeset_payload },
    );
    <a href="LibraConfig.md#0x1_LibraConfig_reconfigure">LibraConfig::reconfigure</a>(lr_account)
}
</code></pre>



</details>

<a name="0x1_LibraWriteSetManager_Specification"></a>

## Specification



<pre><code><b>invariant</b> [<b>global</b>]
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; exists&lt;<a href="#0x1_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>



<a name="0x1_LibraWriteSetManager_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraWriteSetManager_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>;
<b>aborts_if</b> exists&lt;<a href="#0x1_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) with <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
</code></pre>



<a name="0x1_LibraWriteSetManager_Specification_prologue"></a>

### Function `prologue`


<pre><code><b>fun</b> <a href="#0x1_LibraWriteSetManager_prologue">prologue</a>(account: &signer, writeset_sequence_number: u64, writeset_public_key: vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>


Must abort if the signer does not have the LibraRoot role [B18].


<pre><code><b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_libra_root_role_addr">Roles::spec_has_libra_root_role_addr</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>
