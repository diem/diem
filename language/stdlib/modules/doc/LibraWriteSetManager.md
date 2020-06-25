
<a name="0x1_LibraWriteSetManager"></a>

# Module `0x1::LibraWriteSetManager`

### Table of Contents

-  [Resource `LibraWriteSetManager`](#0x1_LibraWriteSetManager_LibraWriteSetManager)
-  [Struct `UpgradeEvent`](#0x1_LibraWriteSetManager_UpgradeEvent)
-  [Function `initialize`](#0x1_LibraWriteSetManager_initialize)
-  [Function `prologue`](#0x1_LibraWriteSetManager_prologue)
-  [Function `epilogue`](#0x1_LibraWriteSetManager_epilogue)



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

<a name="0x1_LibraWriteSetManager_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraWriteSetManager_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraWriteSetManager_initialize">initialize</a>(account: &signer) {
    // Operational constraint
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), 1);

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
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(sender == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), 33);

    <b>let</b> association_auth_key = <a href="LibraAccount.md#0x1_LibraAccount_authentication_key">LibraAccount::authentication_key</a>(sender);
    <b>let</b> sequence_number = <a href="LibraAccount.md#0x1_LibraAccount_sequence_number">LibraAccount::sequence_number</a>(sender);

    <b>assert</b>(writeset_sequence_number &gt;= sequence_number, 3);

    <b>assert</b>(writeset_sequence_number == sequence_number, 11);
    <b>assert</b>(
        <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(writeset_public_key) == association_auth_key,
        2
    );
}
</code></pre>



</details>

<a name="0x1_LibraWriteSetManager_epilogue"></a>

## Function `epilogue`



<pre><code><b>fun</b> <a href="#0x1_LibraWriteSetManager_epilogue">epilogue</a>(account: &signer, writeset_payload: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraWriteSetManager_epilogue">epilogue</a>(account: &signer, writeset_payload: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x1_LibraWriteSetManager">LibraWriteSetManager</a> {
    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x1_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <b>let</b> association_root_capability = <a href="Roles.md#0x1_Roles_extract_privilege_to_capability">Roles::extract_privilege_to_capability</a>(account);

    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraWriteSetManager_UpgradeEvent">Self::UpgradeEvent</a>&gt;(
        &<b>mut</b> t_ref.upgrade_events,
        <a href="#0x1_LibraWriteSetManager_UpgradeEvent">UpgradeEvent</a> { writeset_payload },
    );
    <a href="LibraConfig.md#0x1_LibraConfig_reconfigure">LibraConfig::reconfigure</a>(&association_root_capability);
    <a href="Roles.md#0x1_Roles_restore_capability_to_privilege">Roles::restore_capability_to_privilege</a>(account, association_root_capability);
}
</code></pre>



</details>
