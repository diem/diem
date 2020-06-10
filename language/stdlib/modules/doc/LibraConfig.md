
<a name="0x1_LibraConfig"></a>

# Module `0x1::LibraConfig`

### Table of Contents

-  [Struct `LibraConfig`](#0x1_LibraConfig_LibraConfig)
-  [Struct `NewEpochEvent`](#0x1_LibraConfig_NewEpochEvent)
-  [Struct `Configuration`](#0x1_LibraConfig_Configuration)
-  [Struct `CreateConfigCapability`](#0x1_LibraConfig_CreateConfigCapability)
-  [Struct `ModifyConfigCapability`](#0x1_LibraConfig_ModifyConfigCapability)
-  [Function `initialize`](#0x1_LibraConfig_initialize)
-  [Function `get`](#0x1_LibraConfig_get)
-  [Function `set`](#0x1_LibraConfig_set)
-  [Function `set_with_capability`](#0x1_LibraConfig_set_with_capability)
-  [Function `publish_new_config_with_capability`](#0x1_LibraConfig_publish_new_config_with_capability)
-  [Function `publish_new_treasury_compliance_config`](#0x1_LibraConfig_publish_new_treasury_compliance_config)
-  [Function `publish_new_config`](#0x1_LibraConfig_publish_new_config)
-  [Function `publish_new_config_with_delegate`](#0x1_LibraConfig_publish_new_config_with_delegate)
-  [Function `claim_delegated_modify_config`](#0x1_LibraConfig_claim_delegated_modify_config)
-  [Function `reconfigure`](#0x1_LibraConfig_reconfigure)
-  [Function `reconfigure_`](#0x1_LibraConfig_reconfigure_)
-  [Function `emit_reconfiguration_event`](#0x1_LibraConfig_emit_reconfiguration_event)
-  [Specification](#0x1_LibraConfig_Specification)
    -  [Function `publish_new_config_with_capability`](#0x1_LibraConfig_Specification_publish_new_config_with_capability)
    -  [Function `publish_new_config`](#0x1_LibraConfig_Specification_publish_new_config)



<a name="0x1_LibraConfig_LibraConfig"></a>

## Struct `LibraConfig`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config: <b>copyable</b>&gt;
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

<a name="0x1_LibraConfig_NewEpochEvent"></a>

## Struct `NewEpochEvent`



<pre><code><b>struct</b> <a href="#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a>
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

<a name="0x1_LibraConfig_Configuration"></a>

## Struct `Configuration`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraConfig_Configuration">Configuration</a>
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

<code>events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraConfig_CreateConfigCapability"></a>

## Struct `CreateConfigCapability`



<pre><code><b>struct</b> <a href="#0x1_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>
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

<a name="0x1_LibraConfig_ModifyConfigCapability"></a>

## Struct `ModifyConfigCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;TypeName&gt;
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

<a name="0x1_LibraConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_initialize">initialize</a>(config_account: &signer, association_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_initialize">initialize</a>(config_account: &signer, association_account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>(), 1);
    <a href="Association.md#0x1_Association_grant_privilege">Association::grant_privilege</a>&lt;<a href="#0x1_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;(association_account, config_account);
    <a href="Association.md#0x1_Association_grant_privilege">Association::grant_privilege</a>&lt;<a href="#0x1_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;(association_account, association_account);


    move_to&lt;<a href="#0x1_LibraConfig_Configuration">Configuration</a>&gt;(
        config_account,
        <a href="#0x1_LibraConfig_Configuration">Configuration</a> {
            epoch: 0,
            last_reconfiguration_time: 0,
            events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a>&gt;(config_account),
        }
    );
}
</code></pre>



</details>

<a name="0x1_LibraConfig_get"></a>

## Function `get`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_get">get</a>&lt;Config: <b>copyable</b>&gt;(): Config
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_get">get</a>&lt;Config: <b>copyable</b>&gt;(): Config <b>acquires</b> <a href="#0x1_LibraConfig">LibraConfig</a> {
    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>();
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), 24);
    *&borrow_global&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr).payload
}
</code></pre>



</details>

<a name="0x1_LibraConfig_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_set">set</a>&lt;Config: <b>copyable</b>&gt;(account: &signer, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_set">set</a>&lt;Config: <b>copyable</b>&gt;(account: &signer, payload: Config) <b>acquires</b> <a href="#0x1_LibraConfig">LibraConfig</a>, <a href="#0x1_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>();
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), 24);
    <b>let</b> signer_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(
        exists&lt;<a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(signer_address)
        || <a href="Association.md#0x1_Association_addr_is_association">Association::addr_is_association</a>(signer_address),
        24
    );

    <b>let</b> config = borrow_global_mut&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr);
    config.payload = payload;

    <a href="#0x1_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<a name="0x1_LibraConfig_set_with_capability"></a>

## Function `set_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_set_with_capability">set_with_capability</a>&lt;Config: <b>copyable</b>&gt;(_cap: &<a href="#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;Config&gt;, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_set_with_capability">set_with_capability</a>&lt;Config: <b>copyable</b>&gt;(
    _cap: &<a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;,
    payload: Config
) <b>acquires</b> <a href="#0x1_LibraConfig">LibraConfig</a>, <a href="#0x1_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>();
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), 24);
    <b>let</b> config = borrow_global_mut&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr);
    config.payload = payload;

    <a href="#0x1_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<a name="0x1_LibraConfig_publish_new_config_with_capability"></a>

## Function `publish_new_config_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, payload: Config): <a href="#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;Config&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;Config: <b>copyable</b>&gt;(
    config_account: &signer,
    payload: Config,
): <a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {
    <b>assert</b>(
        <a href="Association.md#0x1_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x1_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account)),
        1
    );

    move_to(config_account, <a href="#0x1_LibraConfig">LibraConfig</a> { payload });
    // We don't trigger reconfiguration here, instead we'll wait for all validators <b>update</b> the binary
    // <b>to</b> register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction <b>to</b> change
    // the value which triggers the reconfiguration.

    <b>return</b> <a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {}
}
</code></pre>



</details>

<a name="0x1_LibraConfig_publish_new_treasury_compliance_config"></a>

## Function `publish_new_treasury_compliance_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_treasury_compliance_config">publish_new_treasury_compliance_config</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, tc_account: &signer, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_treasury_compliance_config">publish_new_treasury_compliance_config</a>&lt;Config: <b>copyable</b>&gt;(
    config_account: &signer,
    tc_account: &signer,
    payload: Config,
) {
    <b>assert</b>(
        <a href="Association.md#0x1_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x1_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account)),
        1
    );
    move_to(config_account, <a href="#0x1_LibraConfig">LibraConfig</a> { payload });
    move_to(tc_account, <a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {});
}
</code></pre>



</details>

<a name="0x1_LibraConfig_publish_new_config"></a>

## Function `publish_new_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, payload: Config) {
    <b>assert</b>(
        <a href="Association.md#0x1_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x1_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account)),
        1
    );

    move_to(config_account, <a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {});
    move_to(config_account, <a href="#0x1_LibraConfig">LibraConfig</a>{ payload });
    // We don't trigger reconfiguration here, instead we'll wait for all validators <b>update</b> the binary
    // <b>to</b> register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction <b>to</b> change
    // the value which triggers the reconfiguration.
}
</code></pre>



</details>

<a name="0x1_LibraConfig_publish_new_config_with_delegate"></a>

## Function `publish_new_config_with_delegate`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config_with_delegate">publish_new_config_with_delegate</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, payload: Config, delegate: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config_with_delegate">publish_new_config_with_delegate</a>&lt;Config: <b>copyable</b>&gt;(
    config_account: &signer,
    payload: Config,
    delegate: address,
) {
    <b>assert</b>(
        <a href="Association.md#0x1_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x1_LibraConfig_CreateConfigCapability">CreateConfigCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account)),
        1
    );

    <a href="Offer.md#0x1_Offer_create">Offer::create</a>(config_account, <a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;{}, delegate);
    move_to(config_account, <a href="#0x1_LibraConfig">LibraConfig</a> { payload });
    // We don't trigger reconfiguration here, instead we'll wait for all validators <b>update</b> the
    // binary <b>to</b> register this config into ON_CHAIN_CONFIG_REGISTRY then send another
    // transaction <b>to</b> change the value which triggers the reconfiguration.
}
</code></pre>



</details>

<a name="0x1_LibraConfig_claim_delegated_modify_config"></a>

## Function `claim_delegated_modify_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_claim_delegated_modify_config">claim_delegated_modify_config</a>&lt;Config&gt;(account: &signer, offer_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_claim_delegated_modify_config">claim_delegated_modify_config</a>&lt;Config&gt;(account: &signer, offer_address: address) {
    move_to(account, <a href="Offer.md#0x1_Offer_redeem">Offer::redeem</a>&lt;<a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(account, offer_address))
}
</code></pre>



</details>

<a name="0x1_LibraConfig_reconfigure"></a>

## Function `reconfigure`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_reconfigure">reconfigure</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_reconfigure">reconfigure</a>(account: &signer) <b>acquires</b> <a href="#0x1_LibraConfig_Configuration">Configuration</a> {
    // Only callable by association address or by the VM internally.
    <b>assert</b>(
        <a href="Association.md#0x1_Association_has_privilege">Association::has_privilege</a>&lt;<a href="#0x1_LibraConfig_CreateConfigCapability">Self::CreateConfigCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        1
    );
    <a href="#0x1_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<a name="0x1_LibraConfig_reconfigure_"></a>

## Function `reconfigure_`



<pre><code><b>fun</b> <a href="#0x1_LibraConfig_reconfigure_">reconfigure_</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraConfig_reconfigure_">reconfigure_</a>() <b>acquires</b> <a href="#0x1_LibraConfig_Configuration">Configuration</a> {
   // Do not do anything <b>if</b> time is not set up yet, this is <b>to</b> avoid genesis emit too many epochs.
   <b>if</b> (<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>()) {
       <b>return</b> ()
   };

   <b>let</b> config_ref = borrow_global_mut&lt;<a href="#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>());

   // Ensure that there is at most one reconfiguration per transaction. This <b>ensures</b> that there is a 1-1
   // correspondence between system reconfigurations and emitted ReconfigurationEvents.

   <b>let</b> current_block_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
   <b>assert</b>(current_block_time &gt; config_ref.last_reconfiguration_time, 23);
   config_ref.last_reconfiguration_time = current_block_time;

   <a href="#0x1_LibraConfig_emit_reconfiguration_event">emit_reconfiguration_event</a>();
}
</code></pre>



</details>

<a name="0x1_LibraConfig_emit_reconfiguration_event"></a>

## Function `emit_reconfiguration_event`



<pre><code><b>fun</b> <a href="#0x1_LibraConfig_emit_reconfiguration_event">emit_reconfiguration_event</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraConfig_emit_reconfiguration_event">emit_reconfiguration_event</a>() <b>acquires</b> <a href="#0x1_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> config_ref = borrow_global_mut&lt;<a href="#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>());
    config_ref.epoch = config_ref.epoch + 1;

    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a>&gt;(
        &<b>mut</b> config_ref.events,
        <a href="#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a> {
            epoch: config_ref.epoch,
        },
    );
}
</code></pre>



</details>

<a name="0x1_LibraConfig_Specification"></a>

## Specification


Specifications of LibraConfig are very incomplete.  There are just a few
definitions that are used by RegisteredCurrencies


<pre><code>pragma verify = <b>true</b>;
<a name="0x1_LibraConfig_spec_get"></a>
<b>define</b> <a href="#0x1_LibraConfig_spec_get">spec_get</a>&lt;Config&gt;(): Config {
    <b>global</b>&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(0xF1A95).payload
}
<a name="0x1_LibraConfig_spec_is_published"></a>
<b>define</b> <a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;(addr: address): bool {
    exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr)
}
</code></pre>



<a name="0x1_LibraConfig_Specification_publish_new_config_with_capability"></a>

### Function `publish_new_config_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config_with_capability">publish_new_config_with_capability</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, payload: Config): <a href="#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;Config&gt;
</code></pre>




<pre><code><b>ensures</b> <b>old</b>(!<a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(config_account)));
<b>ensures</b> <a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(config_account));
</code></pre>



<a name="0x1_LibraConfig_Specification_publish_new_config"></a>

### Function `publish_new_config`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, payload: Config)
</code></pre>




<pre><code><b>ensures</b> <b>old</b>(!<a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(config_account)));
<b>ensures</b> <a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(config_account));
</code></pre>
