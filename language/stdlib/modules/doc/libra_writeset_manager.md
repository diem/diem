
<a name="0x0_LibraWriteSetManager"></a>

# Module `0x0::LibraWriteSetManager`

### Table of Contents

-  [Struct `T`](#0x0_LibraWriteSetManager_T)
-  [Struct `UpgradeEvent`](#0x0_LibraWriteSetManager_UpgradeEvent)
-  [Function `initialize`](#0x0_LibraWriteSetManager_initialize)
-  [Function `prologue`](#0x0_LibraWriteSetManager_prologue)
-  [Function `epilogue`](#0x0_LibraWriteSetManager_epilogue)



<a name="0x0_LibraWriteSetManager_T"></a>

## Struct `T`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraWriteSetManager_T">T</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>sequence_number: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>upgrade_events: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_LibraWriteSetManager_UpgradeEvent">LibraWriteSetManager::UpgradeEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraWriteSetManager_UpgradeEvent"></a>

## Struct `UpgradeEvent`



<pre><code><b>struct</b> <a href="#0x0_LibraWriteSetManager_UpgradeEvent">UpgradeEvent</a>
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

<a name="0x0_LibraWriteSetManager_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraWriteSetManager_initialize">initialize</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraWriteSetManager_initialize">initialize</a>() {
    Transaction::assert(Transaction::sender() == 0xA550C18, 1);

    move_to_sender&lt;<a href="#0x0_LibraWriteSetManager_T">T</a>&gt;(<a href="#0x0_LibraWriteSetManager_T">T</a> {
        sequence_number: 0,
        upgrade_events: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_LibraWriteSetManager_UpgradeEvent">Self::UpgradeEvent</a>&gt;(),
    });
}
</code></pre>



</details>

<a name="0x0_LibraWriteSetManager_prologue"></a>

## Function `prologue`



<pre><code><b>fun</b> <a href="#0x0_LibraWriteSetManager_prologue">prologue</a>(writeset_sequence_number: u64, writeset_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraWriteSetManager_prologue">prologue</a>(
    writeset_sequence_number: u64,
    writeset_public_key: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x0_LibraWriteSetManager_T">T</a> {
    <b>let</b> sender = Transaction::sender();
    Transaction::assert(sender == 0xA550C18, 33);

    <b>let</b> association_auth_key = <a href="libra_account.md#0x0_LibraAccount_authentication_key">LibraAccount::authentication_key</a>(sender);

    <b>let</b> t_ref = borrow_global&lt;<a href="#0x0_LibraWriteSetManager_T">T</a>&gt;(0xA550C18);
    Transaction::assert(writeset_sequence_number &gt;= t_ref.sequence_number, 3);

    Transaction::assert(writeset_sequence_number == t_ref.sequence_number, 11);
    Transaction::assert(
        <a href="hash.md#0x0_Hash_sha3_256">Hash::sha3_256</a>(writeset_public_key) == association_auth_key,
        2
    );
}
</code></pre>



</details>

<a name="0x0_LibraWriteSetManager_epilogue"></a>

## Function `epilogue`



<pre><code><b>fun</b> <a href="#0x0_LibraWriteSetManager_epilogue">epilogue</a>(writeset_payload: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraWriteSetManager_epilogue">epilogue</a>(writeset_payload: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x0_LibraWriteSetManager_T">T</a> {
    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x0_LibraWriteSetManager_T">T</a>&gt;(0xA550C18);
    t_ref.sequence_number = t_ref.sequence_number + 1;

    <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x0_LibraWriteSetManager_UpgradeEvent">Self::UpgradeEvent</a>&gt;(
        &<b>mut</b> t_ref.upgrade_events,
        <a href="#0x0_LibraWriteSetManager_UpgradeEvent">UpgradeEvent</a> { writeset_payload },
    );
    <a href="libra_configs.md#0x0_LibraConfig_reconfigure">LibraConfig::reconfigure</a>();
}
</code></pre>



</details>
