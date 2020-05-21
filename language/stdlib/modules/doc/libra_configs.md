
<a name="0x0_LibraConfig"></a>

# Module `0x0::LibraConfig`

### Table of Contents

-  [Struct `T`](#0x0_LibraConfig_T)
-  [Struct `NewEpochEvent`](#0x0_LibraConfig_NewEpochEvent)
-  [Struct `Configuration`](#0x0_LibraConfig_Configuration)
-  [Struct `CreateConfigCapability`](#0x0_LibraConfig_CreateConfigCapability)
-  [Struct `ModifyConfigCapability`](#0x0_LibraConfig_ModifyConfigCapability)
-  [Function `initialize_configuration`](#0x0_LibraConfig_initialize_configuration)
-  [Function `apply_for_creator_privilege`](#0x0_LibraConfig_apply_for_creator_privilege)
-  [Function `grant_creator_privilege`](#0x0_LibraConfig_grant_creator_privilege)
-  [Function `get`](#0x0_LibraConfig_get)
-  [Function `set`](#0x0_LibraConfig_set)
-  [Function `set_with_capability`](#0x0_LibraConfig_set_with_capability)
-  [Function `publish_new_config_with_capability`](#0x0_LibraConfig_publish_new_config_with_capability)
-  [Function `publish_new_config`](#0x0_LibraConfig_publish_new_config)
-  [Function `publish_new_config_with_delegate`](#0x0_LibraConfig_publish_new_config_with_delegate)
-  [Function `claim_delegated_modify_config`](#0x0_LibraConfig_claim_delegated_modify_config)
-  [Function `reconfigure`](#0x0_LibraConfig_reconfigure)
-  [Function `reconfigure_`](#0x0_LibraConfig_reconfigure_)
-  [Function `emit_reconfiguration_event`](#0x0_LibraConfig_emit_reconfiguration_event)
-  [Function `default_config_address`](#0x0_LibraConfig_default_config_address)



<a name="0x0_LibraConfig_T"></a>

## Struct `T`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraConfig_T">T</a>&lt;Config: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>payload: Config</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraConfig_NewEpochEvent"></a>

## Struct `NewEpochEvent`



<pre><code><b>struct</b> <a href="#0x0_LibraConfig_NewEpochEvent">NewEpochEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>epoch: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraConfig_Configuration"></a>

## Struct `Configuration`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraConfig_Configuration">Configuration</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>epoch: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>last_reconfiguration_time: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>events: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraConfig_CreateConfigCapability"></a>

## Struct `CreateConfigCapability`



<pre><code><b>struct</b> <a href="#0x0_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>
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

<a name="0x0_LibraConfig_ModifyConfigCapability"></a>

## Struct `ModifyConfigCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;TypeName&gt;
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

<a name="0x0_LibraConfig_initialize_configuration"></a>

## Function `initialize_configuration`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_initialize_configuration">initialize_configuration</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_initialize_configuration">initialize_configuration</a>() {
    <b>let</b> sender = Transaction::sender();
    Transaction::assert(sender == <a href="#0x0_LibraConfig_default_config_address">default_config_address</a>(), 1);

    move_to_sender&lt;<a href="#0x0_LibraConfig_Configuration">Configuration</a>&gt;(<a href="#0x0_LibraConfig_Configuration">Configuration</a> {
        epoch: 0,
        last_reconfiguration_time: 0,
        events: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_LibraConfig_NewEpochEvent">NewEpochEvent</a>&gt;(),
    });
}
</code></pre>



</details>

<a name="0x0_LibraConfig_apply_for_creator_privilege"></a>

## Function `apply_for_creator_privilege`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_apply_for_creator_privilege">apply_for_creator_privilege</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_apply_for_creator_privilege">apply_for_creator_privilege</a>() {
    <a href="association.md#0x0_Association_apply_for_association">Association::apply_for_association</a>();
    <a href="association.md#0x0_Association_apply_for_privilege">Association::apply_for_privilege</a>&lt;<a href="#0x0_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;();
}
</code></pre>



</details>

<a name="0x0_LibraConfig_grant_creator_privilege"></a>

## Function `grant_creator_privilege`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_grant_creator_privilege">grant_creator_privilege</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_grant_creator_privilege">grant_creator_privilege</a>(addr: address) {
    <a href="association.md#0x0_Association_grant_association_address">Association::grant_association_address</a>(addr);
    <a href="association.md#0x0_Association_grant_privilege">Association::grant_privilege</a>&lt;<a href="#0x0_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;(addr);
}
</code></pre>



</details>

<a name="0x0_LibraConfig_get"></a>

## Function `get`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_get">get</a>&lt;Config: <b>copyable</b>&gt;(): Config
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_get">get</a>&lt;Config: <b>copyable</b>&gt;(): Config <b>acquires</b> <a href="#0x0_LibraConfig_T">T</a> {
    <b>let</b> addr = <a href="#0x0_LibraConfig_default_config_address">default_config_address</a>();
    Transaction::assert(::exists&lt;<a href="#0x0_LibraConfig_T">T</a>&lt;Config&gt;&gt;(addr), 24);
    *&borrow_global&lt;<a href="#0x0_LibraConfig_T">T</a>&lt;Config&gt;&gt;(addr).payload
}
</code></pre>



</details>

<a name="0x0_LibraConfig_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_set">set</a>&lt;Config: <b>copyable</b>&gt;(payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_set">set</a>&lt;Config: <b>copyable</b>&gt;(payload: Config) <b>acquires</b> <a href="#0x0_LibraConfig_T">T</a>, <a href="#0x0_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> addr = <a href="#0x0_LibraConfig_default_config_address">default_config_address</a>();
    Transaction::assert(::exists&lt;<a href="#0x0_LibraConfig_T">T</a>&lt;Config&gt;&gt;(addr), 24);
    Transaction::assert(
        ::exists&lt;<a href="#0x0_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(Transaction::sender())
         || Transaction::sender() == <a href="association.md#0x0_Association_root_address">Association::root_address</a>(),
        24
    );

    <b>let</b> config = borrow_global_mut&lt;<a href="#0x0_LibraConfig_T">T</a>&lt;Config&gt;&gt;(addr);
    config.payload = payload;

    <a href="#0x0_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<a name="0x0_LibraConfig_set_with_capability"></a>

## Function `set_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_set_with_capability">set_with_capability</a>&lt;Config: <b>copyable</b>&gt;(_cap: &<a href="#0x0_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;Config&gt;, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_set_with_capability">set_with_capability</a>&lt;Config: <b>copyable</b>&gt;(
    _cap: &<a href="#0x0_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;,
    payload: Config
) <b>acquires</b> <a href="#0x0_LibraConfig_T">T</a>, <a href="#0x0_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> addr = <a href="#0x0_LibraConfig_default_config_address">default_config_address</a>();
    Transaction::assert(::exists&lt;<a href="#0x0_LibraConfig_T">T</a>&lt;Config&gt;&gt;(addr), 24);
    <b>let</b> config = borrow_global_mut&lt;<a href="#0x0_LibraConfig_T">T</a>&lt;Config&gt;&gt;(addr);
    config.payload = payload;

    <a href="#0x0_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<a name="0x0_LibraConfig_publish_new_config_with_capability"></a>

## Function `publish_new_config_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;Config: <b>copyable</b>&gt;(payload: Config): <a href="#0x0_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;Config&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;Config: <b>copyable</b>&gt;(payload: Config): <a href="#0x0_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {
    Transaction::assert(
        <a href="association.md#0x0_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x0_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;(Transaction::sender()),
        1
    );

    move_to_sender(<a href="#0x0_LibraConfig_T">T</a>{ payload });
    // We don't trigger reconfiguration here, instead we'll wait for all validators <b>update</b> the binary
    // <b>to</b> register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction <b>to</b> change
    // the value which triggers the reconfiguration.

    <b>return</b> <a href="#0x0_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {}
}
</code></pre>



</details>

<a name="0x0_LibraConfig_publish_new_config"></a>

## Function `publish_new_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(
    payload: Config,
) {
    Transaction::assert(
        <a href="association.md#0x0_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x0_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;(Transaction::sender()),
        1
    );

    move_to_sender(<a href="#0x0_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {});
    move_to_sender(<a href="#0x0_LibraConfig_T">T</a>{ payload });
    // We don't trigger reconfiguration here, instead we'll wait for all validators <b>update</b> the binary
    // <b>to</b> register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction <b>to</b> change
    // the value which triggers the reconfiguration.
}
</code></pre>



</details>

<a name="0x0_LibraConfig_publish_new_config_with_delegate"></a>

## Function `publish_new_config_with_delegate`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_publish_new_config_with_delegate">publish_new_config_with_delegate</a>&lt;Config: <b>copyable</b>&gt;(payload: Config, delegate: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_publish_new_config_with_delegate">publish_new_config_with_delegate</a>&lt;Config: <b>copyable</b>&gt;(
    payload: Config,
    delegate: address,
) {
    Transaction::assert(
        <a href="association.md#0x0_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x0_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;(Transaction::sender()),
        1
    );

    <a href="offer.md#0x0_Offer_create">Offer::create</a>(<a href="#0x0_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {}, delegate);
    move_to_sender(<a href="#0x0_LibraConfig_T">T</a>{ payload });
    // We don't trigger reconfiguration here, instead we'll wait for all validators <b>update</b> the binary
    // <b>to</b> register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction <b>to</b> change
    // the value which triggers the reconfiguration.
}
</code></pre>



</details>

<a name="0x0_LibraConfig_claim_delegated_modify_config"></a>

## Function `claim_delegated_modify_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_claim_delegated_modify_config">claim_delegated_modify_config</a>&lt;Config&gt;(offer_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_claim_delegated_modify_config">claim_delegated_modify_config</a>&lt;Config&gt;(offer_address: address) {
    move_to_sender(<a href="offer.md#0x0_Offer_redeem">Offer::redeem</a>&lt;<a href="#0x0_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(offer_address))
}
</code></pre>



</details>

<a name="0x0_LibraConfig_reconfigure"></a>

## Function `reconfigure`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_reconfigure">reconfigure</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_reconfigure">reconfigure</a>() <b>acquires</b> <a href="#0x0_LibraConfig_Configuration">Configuration</a> {
    // Only callable by association address or by the VM internally.
    Transaction::assert(
        <a href="association.md#0x0_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x0_LibraConfig_CreateConfigCapability">Self::CreateConfigCapability</a>&gt;(Transaction::sender()),
        1
    );
    <a href="#0x0_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<a name="0x0_LibraConfig_reconfigure_"></a>

## Function `reconfigure_`



<pre><code><b>fun</b> <a href="#0x0_LibraConfig_reconfigure_">reconfigure_</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraConfig_reconfigure_">reconfigure_</a>() <b>acquires</b> <a href="#0x0_LibraConfig_Configuration">Configuration</a> {
   // Do not do anything <b>if</b> time is not set up yet, this is <b>to</b> avoid genesis emit too many epochs.
   <b>if</b>(<a href="libra_time.md#0x0_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>()) {
       <b>return</b> ()
   };

   <b>let</b> config_ref = borrow_global_mut&lt;<a href="#0x0_LibraConfig_Configuration">Configuration</a>&gt;(<a href="#0x0_LibraConfig_default_config_address">default_config_address</a>());

   // Ensure that there is at most one reconfiguration per transaction. This <b>ensures</b> that there is a 1-1
   // correspondence between system reconfigurations and emitted ReconfigurationEvents.

   <b>let</b> current_block_time = <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
   Transaction::assert(current_block_time &gt; config_ref.last_reconfiguration_time, 23);
   config_ref.last_reconfiguration_time = current_block_time;

   <a href="#0x0_LibraConfig_emit_reconfiguration_event">emit_reconfiguration_event</a>();
}
</code></pre>



</details>

<a name="0x0_LibraConfig_emit_reconfiguration_event"></a>

## Function `emit_reconfiguration_event`



<pre><code><b>fun</b> <a href="#0x0_LibraConfig_emit_reconfiguration_event">emit_reconfiguration_event</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraConfig_emit_reconfiguration_event">emit_reconfiguration_event</a>() <b>acquires</b> <a href="#0x0_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> config_ref = borrow_global_mut&lt;<a href="#0x0_LibraConfig_Configuration">Configuration</a>&gt;(<a href="#0x0_LibraConfig_default_config_address">default_config_address</a>());
    config_ref.epoch = config_ref.epoch + 1;

    <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x0_LibraConfig_NewEpochEvent">NewEpochEvent</a>&gt;(
        &<b>mut</b> config_ref.events,
        <a href="#0x0_LibraConfig_NewEpochEvent">NewEpochEvent</a> {
            epoch: config_ref.epoch,
        },
    );
}
</code></pre>



</details>

<a name="0x0_LibraConfig_default_config_address"></a>

## Function `default_config_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_default_config_address">default_config_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraConfig_default_config_address">default_config_address</a>(): address {
    0xF1A95
}
</code></pre>



</details>
