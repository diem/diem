
<a name="0x1_LibraConfig"></a>

# Module `0x1::LibraConfig`

### Table of Contents

-  [Resource `LibraConfig`](#0x1_LibraConfig_LibraConfig)
-  [Struct `NewEpochEvent`](#0x1_LibraConfig_NewEpochEvent)
-  [Resource `Configuration`](#0x1_LibraConfig_Configuration)
-  [Resource `ModifyConfigCapability`](#0x1_LibraConfig_ModifyConfigCapability)
-  [Function `initialize`](#0x1_LibraConfig_initialize)
-  [Function `get`](#0x1_LibraConfig_get)
-  [Function `set`](#0x1_LibraConfig_set)
-  [Function `publish_new_config`](#0x1_LibraConfig_publish_new_config)
-  [Function `reconfigure`](#0x1_LibraConfig_reconfigure)
-  [Function `reconfigure_`](#0x1_LibraConfig_reconfigure_)
-  [Function `emit_reconfiguration_event`](#0x1_LibraConfig_emit_reconfiguration_event)
-  [Specification](#0x1_LibraConfig_Specification)
    -  [Function `publish_new_config`](#0x1_LibraConfig_Specification_publish_new_config)
    -  [Function `reconfigure_`](#0x1_LibraConfig_Specification_reconfigure_)



<a name="0x1_LibraConfig_LibraConfig"></a>

## Resource `LibraConfig`



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

## Resource `Configuration`



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

<a name="0x1_LibraConfig_ModifyConfigCapability"></a>

## Resource `ModifyConfigCapability`



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



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_initialize">initialize</a>(config_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_initialize">initialize</a>(
    config_account: &signer,
) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    // Operational constraint
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), EINVALID_SINGLETON_ADDRESS);
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


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_get">get</a>&lt;Config: <b>copyable</b>&gt;(): Config
<b>acquires</b> <a href="#0x1_LibraConfig">LibraConfig</a> {
    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), ECONFIG_DOES_NOT_EXIST);
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


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_set">set</a>&lt;Config: <b>copyable</b>&gt;(account: &signer, payload: Config)
<b>acquires</b> <a href="#0x1_LibraConfig">LibraConfig</a>, <a href="#0x1_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), ECONFIG_DOES_NOT_EXIST);
    <b>let</b> signer_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(signer_address), EMODIFY_CAPABILITY_NOT_HELD);

    <b>let</b> config = borrow_global_mut&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr);
    config.payload = payload;

    <a href="#0x1_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<a name="0x1_LibraConfig_publish_new_config"></a>

## Function `publish_new_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(
    config_account: &signer,
    payload: Config
) {
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_on_chain_config_privilege">Roles::has_on_chain_config_privilege</a>(config_account), ENO_CONFIG_PRIVILEGE);
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), EINVALID_SINGLETON_ADDRESS);
    move_to(config_account, <a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {});
    move_to(config_account, <a href="#0x1_LibraConfig">LibraConfig</a>{ payload });
    // We don't trigger reconfiguration here, instead we'll wait for all validators <b>update</b> the binary
    // <b>to</b> register this config into ON_CHAIN_CONFIG_REGISTRY then send another transaction <b>to</b> change
    // the value which triggers the reconfiguration.
}
</code></pre>



</details>

<a name="0x1_LibraConfig_reconfigure"></a>

## Function `reconfigure`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_reconfigure">reconfigure</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_reconfigure">reconfigure</a>(
    lr_account: &signer,
) <b>acquires</b> <a href="#0x1_LibraConfig_Configuration">Configuration</a> {
    // Only callable by libra root account or by the VM internally.
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(lr_account), ENOT_LIBRA_ROOT);
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
   <b>if</b> (<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_not_initialized">LibraTimestamp::is_not_initialized</a>()) {
       <b>return</b> ()
   };

   <b>let</b> config_ref = borrow_global_mut&lt;<a href="#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());

   // Ensure that there is at most one reconfiguration per transaction. This <b>ensures</b> that there is a 1-1
   // correspondence between system reconfigurations and emitted ReconfigurationEvents.

   <b>let</b> current_block_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
   <b>assert</b>(current_block_time &gt; config_ref.last_reconfiguration_time, EINVALID_BLOCK_TIME);
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
    <b>let</b> config_ref = borrow_global_mut&lt;<a href="#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
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


<a name="0x1_LibraConfig_Specification_publish_new_config"></a>

### Function `publish_new_config`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, payload: Config)
</code></pre>



TODO(wrwg): enable
aborts_if spec_is_published<Config>();


<pre><code><b>ensures</b> <b>old</b>(!<a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;());
<b>ensures</b> <a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;();
</code></pre>



<a name="0x1_LibraConfig_Specification_reconfigure_"></a>

### Function `reconfigure_`


<pre><code><b>fun</b> <a href="#0x1_LibraConfig_reconfigure_">reconfigure_</a>()
</code></pre>



Consider only states for verification where this function does not abort
for a caller. This prevents that callers need to propagate the abort conditions of this
function up the call chain. The abort conditions of this function represent
internal programming errors.

> TODO(wrwg): we should have a convention to distinguish error codes resulting from
contract program errors and from errors coming from inputs to transaction
scripts. In most cases, only the later one should be propagated upwards to callers.
For now, we use the pragma below to simulate this.


<pre><code>pragma assume_no_abort_from_here = <b>true</b>;
</code></pre>



> TODO(wrwg): We've removed an invariant in RegisteredCurrencies that config is only stored
Specifications of LibraConfig are very incomplete.  There are just a few
definitions that are used by RegisteredCurrencies


<pre><code>pragma verify = <b>true</b>;
<a name="0x1_LibraConfig_spec_has_config"></a>
<b>define</b> <a href="#0x1_LibraConfig_spec_has_config">spec_has_config</a>(): bool {
    exists&lt;<a href="#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>


Spec version of
<code><a href="#0x1_LibraConfig_get">LibraConfig::get</a>&lt;Config&gt;</code>.


<a name="0x1_LibraConfig_spec_get"></a>


<pre><code><b>define</b> <a href="#0x1_LibraConfig_spec_get">spec_get</a>&lt;Config&gt;(): Config {
    <b>global</b>&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>()).payload
}
</code></pre>


Spec version of
<code>LibraConfig::is_published&lt;Config&gt;</code>.


<a name="0x1_LibraConfig_spec_is_published"></a>


<pre><code><b>define</b> <a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;(): bool {
    exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>())
}
<a name="0x1_LibraConfig_spec_has_on_chain_config_privilege_addr"></a>
<b>define</b> <a href="#0x1_LibraConfig_spec_has_on_chain_config_privilege_addr">spec_has_on_chain_config_privilege_addr</a>(addr: address): bool {
    <a href="Roles.md#0x1_Roles_spec_has_libra_root_role_addr">Roles::spec_has_libra_root_role_addr</a>(addr)
}
</code></pre>
