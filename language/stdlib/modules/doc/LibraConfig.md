
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
-  [Function `set_with_capability_and_reconfigure`](#0x1_LibraConfig_set_with_capability_and_reconfigure)
-  [Function `publish_new_config_and_get_capability`](#0x1_LibraConfig_publish_new_config_and_get_capability)
-  [Function `publish_new_config`](#0x1_LibraConfig_publish_new_config)
-  [Function `reconfigure`](#0x1_LibraConfig_reconfigure)
-  [Function `reconfigure_`](#0x1_LibraConfig_reconfigure_)
-  [Function `emit_reconfiguration_event`](#0x1_LibraConfig_emit_reconfiguration_event)
-  [Specification](#0x1_LibraConfig_Specification)
    -  [Function `get`](#0x1_LibraConfig_Specification_get)
    -  [Function `set`](#0x1_LibraConfig_Specification_set)
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
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(config_account);
    <b>assert</b>(!exists&lt;<a href="#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(ECONFIGURATION));
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
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(ELIBRA_CONFIG));
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
    <b>let</b> signer_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(signer_address), <a href="Errors.md#0x1_Errors_requires_privilege">Errors::requires_privilege</a>(EMODIFY_CAPABILITY));

    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(ELIBRA_CONFIG));
    <b>let</b> config = borrow_global_mut&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr);
    config.payload = payload;

    <a href="#0x1_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<a name="0x1_LibraConfig_set_with_capability_and_reconfigure"></a>

## Function `set_with_capability_and_reconfigure`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_set_with_capability_and_reconfigure">set_with_capability_and_reconfigure</a>&lt;Config: <b>copyable</b>&gt;(_cap: &<a href="#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;Config&gt;, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_set_with_capability_and_reconfigure">set_with_capability_and_reconfigure</a>&lt;Config: <b>copyable</b>&gt;(
    _cap: &<a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;,
    payload: Config
) <b>acquires</b> <a href="#0x1_LibraConfig">LibraConfig</a>, <a href="#0x1_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(ELIBRA_CONFIG));
    <b>let</b> config = borrow_global_mut&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr);
    config.payload = payload;
    <a href="#0x1_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<a name="0x1_LibraConfig_publish_new_config_and_get_capability"></a>

## Function `publish_new_config_and_get_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config_and_get_capability">publish_new_config_and_get_capability</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, payload: Config): <a href="#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;Config&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config_and_get_capability">publish_new_config_and_get_capability</a>&lt;Config: <b>copyable</b>&gt;(
    config_account: &signer,
    payload: Config,
): <a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(config_account);
    <b>assert</b>(
        !exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(ELIBRA_CONFIG)
    );
    move_to(config_account, <a href="#0x1_LibraConfig">LibraConfig</a> { payload });
    <a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {}
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
    <b>let</b> capability = <a href="#0x1_LibraConfig_publish_new_config_and_get_capability">publish_new_config_and_get_capability</a>&lt;Config&gt;(config_account, payload);
    <b>assert</b>(
        !exists&lt;<a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(EMODIFY_CAPABILITY)
    );
    move_to(config_account, capability);
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
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
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
   <b>assert</b>(current_block_time &gt; config_ref.last_reconfiguration_time, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(EINVALID_BLOCK_TIME));
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
    <b>assert</b>(exists&lt;<a href="#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(ECONFIGURATION));
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


<a name="0x1_LibraConfig_Specification_get"></a>

### Function `get`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_get">get</a>&lt;Config: <b>copyable</b>&gt;(): Config
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_LibraConfig_AbortsIfNotPublished">AbortsIfNotPublished</a>&lt;Config&gt;;
<b>ensures</b> result == <a href="#0x1_LibraConfig_spec_get">spec_get</a>&lt;Config&gt;();
</code></pre>




<a name="0x1_LibraConfig_AbortsIfNotPublished"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraConfig_AbortsIfNotPublished">AbortsIfNotPublished</a>&lt;Config&gt; {
    <b>aborts_if</b> !exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) with Errors::NOT_PUBLISHED;
}
</code></pre>



<a name="0x1_LibraConfig_Specification_set"></a>

### Function `set`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_set">set</a>&lt;Config: <b>copyable</b>&gt;(account: &signer, payload: Config)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_LibraConfig_AbortsIfNotModifiable">AbortsIfNotModifiable</a>&lt;Config&gt;;
</code></pre>




<a name="0x1_LibraConfig_AbortsIfNotModifiable"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraConfig_AbortsIfNotModifiable">AbortsIfNotModifiable</a>&lt;Config&gt; {
    account: signer;
    <b>include</b> <a href="#0x1_LibraConfig_AbortsIfNotPublished">AbortsIfNotPublished</a>&lt;Config&gt;;
    <b>aborts_if</b> !exists&lt;<a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account))
        with Errors::REQUIRES_PRIVILEGE;
}
</code></pre>



<a name="0x1_LibraConfig_Specification_publish_new_config"></a>

### Function `publish_new_config`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(config_account: &signer, payload: Config)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_LibraConfig_PublishNewConfigAbortsIf">PublishNewConfigAbortsIf</a>&lt;Config&gt;;
<b>include</b> <a href="#0x1_LibraConfig_PublishNewConfigEnsures">PublishNewConfigEnsures</a>&lt;Config&gt;;
</code></pre>




<a name="0x1_LibraConfig_PublishNewConfigAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraConfig_PublishNewConfigAbortsIf">PublishNewConfigAbortsIf</a>&lt;Config&gt; {
    config_account: signer;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: config_account};
    <b>aborts_if</b> <a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;();
    <b>aborts_if</b> exists&lt;<a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(config_account));
}
</code></pre>




<a name="0x1_LibraConfig_PublishNewConfigEnsures"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraConfig_PublishNewConfigEnsures">PublishNewConfigEnsures</a>&lt;Config&gt; {
    config_account: signer;
    <b>ensures</b> <a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;();
    <b>ensures</b> exists&lt;<a href="#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(config_account));
}
</code></pre>



<a name="0x1_LibraConfig_Specification_reconfigure_"></a>

### Function `reconfigure_`


<pre><code><b>fun</b> <a href="#0x1_LibraConfig_reconfigure_">reconfigure_</a>()
</code></pre>



The effect of this function is currently excluded from verification.
> TODO: still specify this function using the
<code>[concrete]</code> property so it can be locally verified.


<pre><code>pragma opaque, verify = <b>false</b>;
<b>aborts_if</b> <b>false</b>;
</code></pre>



TODO: Specifications of LibraConfig are very incomplete.


<pre><code>pragma verify = <b>true</b>;
</code></pre>


Configurations are only stored at the libra root address.


<pre><code><b>invariant</b>
    forall config_address: address, config_type: type where exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;config_type&gt;&gt;(config_address):
        config_address == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
</code></pre>


After genesis, no new configurations are added.


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>]
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
        (forall config_type: type
         where <b>old</b>(!exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;config_type&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())):
             !exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;config_type&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()));
<a name="0x1_LibraConfig_spec_has_config"></a>
<b>define</b> <a href="#0x1_LibraConfig_spec_has_config">spec_has_config</a>(): bool {
    exists&lt;<a href="#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>


Spec version of
<code><a href="#0x1_LibraConfig_get">LibraConfig::get</a>&lt;Config&gt;</code>.


<a name="0x1_LibraConfig_spec_get"></a>


<pre><code><b>define</b> <a href="#0x1_LibraConfig_spec_get">spec_get</a>&lt;Config&gt;(): Config {
    <b>global</b>&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).payload
}
</code></pre>


Spec version of
<code>LibraConfig::is_published&lt;Config&gt;</code>.


<a name="0x1_LibraConfig_spec_is_published"></a>


<pre><code><b>define</b> <a href="#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;(): bool {
    exists&lt;<a href="#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>
