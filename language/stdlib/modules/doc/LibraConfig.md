
<a name="0x1_LibraConfig"></a>

# Module `0x1::LibraConfig`



-  [Resource <code><a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a></code>](#0x1_LibraConfig_LibraConfig)
-  [Struct <code><a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a></code>](#0x1_LibraConfig_NewEpochEvent)
-  [Resource <code><a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a></code>](#0x1_LibraConfig_Configuration)
-  [Resource <code><a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a></code>](#0x1_LibraConfig_ModifyConfigCapability)
-  [Const <code><a href="LibraConfig.md#0x1_LibraConfig_ECONFIGURATION">ECONFIGURATION</a></code>](#0x1_LibraConfig_ECONFIGURATION)
-  [Const <code><a href="LibraConfig.md#0x1_LibraConfig_ELIBRA_CONFIG">ELIBRA_CONFIG</a></code>](#0x1_LibraConfig_ELIBRA_CONFIG)
-  [Const <code><a href="LibraConfig.md#0x1_LibraConfig_EMODIFY_CAPABILITY">EMODIFY_CAPABILITY</a></code>](#0x1_LibraConfig_EMODIFY_CAPABILITY)
-  [Const <code><a href="LibraConfig.md#0x1_LibraConfig_EINVALID_BLOCK_TIME">EINVALID_BLOCK_TIME</a></code>](#0x1_LibraConfig_EINVALID_BLOCK_TIME)
-  [Const <code><a href="LibraConfig.md#0x1_LibraConfig_MAX_U64">MAX_U64</a></code>](#0x1_LibraConfig_MAX_U64)
-  [Function <code>initialize</code>](#0x1_LibraConfig_initialize)
-  [Function <code>get</code>](#0x1_LibraConfig_get)
-  [Function <code>set</code>](#0x1_LibraConfig_set)
-  [Function <code>set_with_capability_and_reconfigure</code>](#0x1_LibraConfig_set_with_capability_and_reconfigure)
-  [Function <code>publish_new_config_and_get_capability</code>](#0x1_LibraConfig_publish_new_config_and_get_capability)
-  [Function <code>publish_new_config</code>](#0x1_LibraConfig_publish_new_config)
-  [Function <code>reconfigure</code>](#0x1_LibraConfig_reconfigure)
-  [Function <code>reconfigure_</code>](#0x1_LibraConfig_reconfigure_)
-  [Function <code>emit_genesis_reconfiguration_event</code>](#0x1_LibraConfig_emit_genesis_reconfiguration_event)
-  [Module Specification](#@Module_Specification_0)


<a name="0x1_LibraConfig_LibraConfig"></a>

## Resource `LibraConfig`



<pre><code><b>resource</b> <b>struct</b> <a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config: <b>copyable</b>&gt;
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



<pre><code><b>struct</b> <a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a>
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



<pre><code><b>resource</b> <b>struct</b> <a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>
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
<code>events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraConfig_ModifyConfigCapability"></a>

## Resource `ModifyConfigCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;TypeName&gt;
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

<a name="0x1_LibraConfig_ECONFIGURATION"></a>

## Const `ECONFIGURATION`

The <code><a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="LibraConfig.md#0x1_LibraConfig_ECONFIGURATION">ECONFIGURATION</a>: u64 = 0;
</code></pre>



<a name="0x1_LibraConfig_ELIBRA_CONFIG"></a>

## Const `ELIBRA_CONFIG`

A <code><a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="LibraConfig.md#0x1_LibraConfig_ELIBRA_CONFIG">ELIBRA_CONFIG</a>: u64 = 1;
</code></pre>



<a name="0x1_LibraConfig_EMODIFY_CAPABILITY"></a>

## Const `EMODIFY_CAPABILITY`

A <code><a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a></code> is in a different state than was expected


<pre><code><b>const</b> <a href="LibraConfig.md#0x1_LibraConfig_EMODIFY_CAPABILITY">EMODIFY_CAPABILITY</a>: u64 = 2;
</code></pre>



<a name="0x1_LibraConfig_EINVALID_BLOCK_TIME"></a>

## Const `EINVALID_BLOCK_TIME`

An invalid block time was encountered.


<pre><code><b>const</b> <a href="LibraConfig.md#0x1_LibraConfig_EINVALID_BLOCK_TIME">EINVALID_BLOCK_TIME</a>: u64 = 4;
</code></pre>



<a name="0x1_LibraConfig_MAX_U64"></a>

## Const `MAX_U64`



<pre><code><b>const</b> <a href="LibraConfig.md#0x1_LibraConfig_MAX_U64">MAX_U64</a>: u64 = 18446744073709551615;
</code></pre>



<a name="0x1_LibraConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_initialize">initialize</a>(
    lr_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);
    <b>assert</b>(!<b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="LibraConfig.md#0x1_LibraConfig_ECONFIGURATION">ECONFIGURATION</a>));
    move_to&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(
        lr_account,
        <a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a> {
            epoch: 0,
            last_reconfiguration_time: 0,
            events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a>&gt;(lr_account),
        }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_InitializeAbortsIf">InitializeAbortsIf</a>;
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>ensures</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>();
<a name="0x1_LibraConfig_new_config$17"></a>
<b>let</b> new_config = <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>ensures</b> new_config.epoch == 0;
<b>ensures</b> new_config.last_reconfiguration_time == 0;
</code></pre>




<a name="0x1_LibraConfig_InitializeAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_InitializeAbortsIf">InitializeAbortsIf</a> {
    lr_account: signer;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
    <b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>{account: lr_account};
    <b>aborts_if</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>() <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
}
</code></pre>



</details>

<a name="0x1_LibraConfig_get"></a>

## Function `get`



<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_get">get</a>&lt;Config: <b>copyable</b>&gt;(): Config
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_get">get</a>&lt;Config: <b>copyable</b>&gt;(): Config
<b>acquires</b> <a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a> {
    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
    <b>assert</b>(<b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraConfig.md#0x1_LibraConfig_ELIBRA_CONFIG">ELIBRA_CONFIG</a>));
    *&borrow_global&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr).payload
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">AbortsIfNotPublished</a>&lt;Config&gt;;
<b>ensures</b> result == <a href="LibraConfig.md#0x1_LibraConfig_get">get</a>&lt;Config&gt;();
</code></pre>




<a name="0x1_LibraConfig_AbortsIfNotPublished"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">AbortsIfNotPublished</a>&lt;Config&gt; {
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
}
</code></pre>



</details>

<a name="0x1_LibraConfig_set"></a>

## Function `set`



<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_set">set</a>&lt;Config: <b>copyable</b>&gt;(account: &signer, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_set">set</a>&lt;Config: <b>copyable</b>&gt;(account: &signer, payload: Config)
<b>acquires</b> <a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>, <a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> signer_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(<b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(signer_address), <a href="Errors.md#0x1_Errors_requires_capability">Errors::requires_capability</a>(<a href="LibraConfig.md#0x1_LibraConfig_EMODIFY_CAPABILITY">EMODIFY_CAPABILITY</a>));

    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
    <b>assert</b>(<b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraConfig.md#0x1_LibraConfig_ELIBRA_CONFIG">ELIBRA_CONFIG</a>));
    <b>let</b> config = borrow_global_mut&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr);
    config.payload = payload;

    <a href="LibraConfig.md#0x1_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetAbortsIf">SetAbortsIf</a>&lt;Config&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetEnsures">SetEnsures</a>&lt;Config&gt;;
</code></pre>




<a name="0x1_LibraConfig_SetAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_SetAbortsIf">SetAbortsIf</a>&lt;Config&gt; {
    account: signer;
    <b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotModifiable">AbortsIfNotModifiable</a>&lt;Config&gt;;
    <b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">AbortsIfNotPublished</a>&lt;Config&gt;;
    <b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">ReconfigureAbortsIf</a>;
}
</code></pre>




<a name="0x1_LibraConfig_AbortsIfNotModifiable"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotModifiable">AbortsIfNotModifiable</a>&lt;Config&gt; {
    account: signer;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account))
        <b>with</b> <a href="Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a>;
}
</code></pre>




<a name="0x1_LibraConfig_SetEnsures"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_SetEnsures">SetEnsures</a>&lt;Config&gt; {
    payload: Config;
    <b>ensures</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;();
    <b>ensures</b> <a href="LibraConfig.md#0x1_LibraConfig_get">get</a>&lt;Config&gt;() == payload;
}
</code></pre>



</details>

<a name="0x1_LibraConfig_set_with_capability_and_reconfigure"></a>

## Function `set_with_capability_and_reconfigure`



<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_set_with_capability_and_reconfigure">set_with_capability_and_reconfigure</a>&lt;Config: <b>copyable</b>&gt;(_cap: &<a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;Config&gt;, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_set_with_capability_and_reconfigure">set_with_capability_and_reconfigure</a>&lt;Config: <b>copyable</b>&gt;(
    _cap: &<a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;,
    payload: Config
) <b>acquires</b> <a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>, <a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> addr = <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
    <b>assert</b>(<b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraConfig.md#0x1_LibraConfig_ELIBRA_CONFIG">ELIBRA_CONFIG</a>));
    <b>let</b> config = borrow_global_mut&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(addr);
    config.payload = payload;
    <a href="LibraConfig.md#0x1_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">AbortsIfNotPublished</a>&lt;Config&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">ReconfigureAbortsIf</a>;
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetEnsures">SetEnsures</a>&lt;Config&gt;;
</code></pre>



</details>

<a name="0x1_LibraConfig_publish_new_config_and_get_capability"></a>

## Function `publish_new_config_and_get_capability`



<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config_and_get_capability">publish_new_config_and_get_capability</a>&lt;Config: <b>copyable</b>&gt;(lr_account: &signer, payload: Config): <a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;Config&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config_and_get_capability">publish_new_config_and_get_capability</a>&lt;Config: <b>copyable</b>&gt;(
    lr_account: &signer,
    payload: Config,
): <a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <b>assert</b>(
        !<b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="LibraConfig.md#0x1_LibraConfig_ELIBRA_CONFIG">ELIBRA_CONFIG</a>)
    );
    move_to(lr_account, <a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a> { payload });
    <a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt; {}
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfPublished">AbortsIfPublished</a>&lt;Config&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetEnsures">SetEnsures</a>&lt;Config&gt;;
</code></pre>




<a name="0x1_LibraConfig_AbortsIfPublished"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfPublished">AbortsIfPublished</a>&lt;Config&gt; {
    <b>aborts_if</b> <b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
}
</code></pre>



</details>

<a name="0x1_LibraConfig_publish_new_config"></a>

## Function `publish_new_config`



<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(lr_account: &signer, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">publish_new_config</a>&lt;Config: <b>copyable</b>&gt;(
    lr_account: &signer,
    payload: Config
) {
    <b>let</b> capability = <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config_and_get_capability">publish_new_config_and_get_capability</a>&lt;Config&gt;(lr_account, payload);
    <b>assert</b>(
        !<b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account)),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="LibraConfig.md#0x1_LibraConfig_EMODIFY_CAPABILITY">EMODIFY_CAPABILITY</a>)
    );
    move_to(lr_account, capability);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigAbortsIf">PublishNewConfigAbortsIf</a>&lt;Config&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigEnsures">PublishNewConfigEnsures</a>&lt;Config&gt;;
</code></pre>




<a name="0x1_LibraConfig_PublishNewConfigAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigAbortsIf">PublishNewConfigAbortsIf</a>&lt;Config&gt; {
    lr_account: signer;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
    <b>aborts_if</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;();
    <b>aborts_if</b> <b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account));
}
</code></pre>




<a name="0x1_LibraConfig_PublishNewConfigEnsures"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigEnsures">PublishNewConfigEnsures</a>&lt;Config&gt; {
    lr_account: signer;
    payload: Config;
    <b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetEnsures">SetEnsures</a>&lt;Config&gt;;
    <b>ensures</b> <b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a>&lt;Config&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account));
}
</code></pre>



</details>

<a name="0x1_LibraConfig_reconfigure"></a>

## Function `reconfigure`



<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_reconfigure">reconfigure</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_reconfigure">reconfigure</a>(
    lr_account: &signer,
) <b>acquires</b> <a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a> {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <a href="LibraConfig.md#0x1_LibraConfig_reconfigure_">reconfigure_</a>();
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">ReconfigureAbortsIf</a>;
</code></pre>



</details>

<a name="0x1_LibraConfig_reconfigure_"></a>

## Function `reconfigure_`



<pre><code><b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_reconfigure_">reconfigure_</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_reconfigure_">reconfigure_</a>() <b>acquires</b> <a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a> {
    // Do not do anything <b>if</b> genesis has not finished.
    <b>if</b> (<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>() || <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>() == 0) {
        <b>return</b> ()
    };

    <b>let</b> config_ref = borrow_global_mut&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <b>let</b> current_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();

    // Do not do anything <b>if</b> a reconfiguration event is already emitted within this transaction.
    <b>if</b> (current_time == config_ref.last_reconfiguration_time) {
        <b>return</b> ()
    };

    <b>assert</b>(current_time &gt; config_ref.last_reconfiguration_time, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="LibraConfig.md#0x1_LibraConfig_EINVALID_BLOCK_TIME">EINVALID_BLOCK_TIME</a>));
    config_ref.last_reconfiguration_time = current_time;
    config_ref.epoch = config_ref.epoch + 1;

    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a>&gt;(
        &<b>mut</b> config_ref.events,
        <a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a> {
            epoch: config_ref.epoch,
        },
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<a name="0x1_LibraConfig_config$18"></a>
<b>let</b> config = <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<a name="0x1_LibraConfig_now$19"></a>
<b>let</b> now = <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>();
<a name="0x1_LibraConfig_epoch$20"></a>
<b>let</b> epoch = config.epoch;
<b>include</b> !<a href="LibraConfig.md#0x1_LibraConfig_spec_reconfigure_omitted">spec_reconfigure_omitted</a>() ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_InternalReconfigureAbortsIf">InternalReconfigureAbortsIf</a> && <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">ReconfigureAbortsIf</a>;
<b>ensures</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_reconfigure_omitted">spec_reconfigure_omitted</a>() ==&gt; config == <b>old</b>(config);
<b>ensures</b> !<a href="LibraConfig.md#0x1_LibraConfig_spec_reconfigure_omitted">spec_reconfigure_omitted</a>() ==&gt; config ==
    update_field(
    update_field(<b>old</b>(config),
        epoch, <b>old</b>(config.epoch) + 1),
        last_reconfiguration_time, now);
</code></pre>


The following schema describes aborts conditions which we do not want to be propagated to the verification
of callers, and which are therefore marked as <code>concrete</code> to be only verified against the implementation.
These conditions are unlikely to happen in reality, and excluding them avoids formal noise.


<a name="0x1_LibraConfig_InternalReconfigureAbortsIf"></a>


<a name="0x1_LibraConfig_config$15"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_InternalReconfigureAbortsIf">InternalReconfigureAbortsIf</a> {
    <b>let</b> config = <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <a name="0x1_LibraConfig_current_time$16"></a>
    <b>let</b> current_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>();
    <b>aborts_if</b> [concrete] current_time &lt;= config.last_reconfiguration_time <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
    <b>aborts_if</b> [concrete] config.epoch == <a href="LibraConfig.md#0x1_LibraConfig_MAX_U64">MAX_U64</a> <b>with</b> EXECUTION_FAILURE;
}
</code></pre>




<a name="0x1_LibraConfig_ReconfigureAbortsIf"></a>


<a name="0x1_LibraConfig_config$13"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">ReconfigureAbortsIf</a> {
    <b>let</b> config = <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <a name="0x1_LibraConfig_current_time$14"></a>
    <b>let</b> current_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>();
    <b>aborts_if</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>()
        && <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>() &gt; 0
        && config.epoch &lt; <a href="LibraConfig.md#0x1_LibraConfig_MAX_U64">MAX_U64</a>
        && current_time == config.last_reconfiguration_time <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
}
</code></pre>



</details>

<a name="0x1_LibraConfig_emit_genesis_reconfiguration_event"></a>

## Function `emit_genesis_reconfiguration_event`



<pre><code><b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_emit_genesis_reconfiguration_event">emit_genesis_reconfiguration_event</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_emit_genesis_reconfiguration_event">emit_genesis_reconfiguration_event</a>() <b>acquires</b> <a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a> {
    <b>assert</b>(<b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraConfig.md#0x1_LibraConfig_ECONFIGURATION">ECONFIGURATION</a>));
    <b>let</b> config_ref = borrow_global_mut&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <b>assert</b>(config_ref.epoch == 0 && config_ref.last_reconfiguration_time == 0, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="LibraConfig.md#0x1_LibraConfig_ECONFIGURATION">ECONFIGURATION</a>));
    config_ref.epoch = 1;

    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a>&gt;(
        &<b>mut</b> config_ref.events,
        <a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a> {
            epoch: config_ref.epoch,
        },
    );
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



<a name="0x1_LibraConfig_spec_reconfigure_omitted"></a>


<pre><code><b>define</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_reconfigure_omitted">spec_reconfigure_omitted</a>(): bool {
  <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>() || <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>() == 0
}
</code></pre>




<a name="0x1_LibraConfig_spec_has_config"></a>


<pre><code><b>define</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>(): bool {
    <b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
<a name="0x1_LibraConfig_spec_is_published"></a>
<b>define</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;Config&gt;(): bool {
    <b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
<a name="0x1_LibraConfig_spec_get_config"></a>
<b>define</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_get_config">spec_get_config</a>&lt;Config&gt;(): Config {
    <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).payload
}
</code></pre>


After genesis, the <code><a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a></code> is published.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>();
</code></pre>


Configurations are only stored at the libra root address.


<pre><code><b>invariant</b> [<b>global</b>]
    <b>forall</b> config_address: address, config_type: type <b>where</b> <b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;config_type&gt;&gt;(config_address):
        config_address == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
</code></pre>


After genesis, no new configurations are added.


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>]
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
        (<b>forall</b> config_type: type <b>where</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;config_type&gt;(): <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;config_type&gt;()));
<b>invariant</b> <b>update</b> [<b>global</b>]
    (<b>forall</b> config_type: type <b>where</b> <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;config_type&gt;()): <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;config_type&gt;());
</code></pre>

[]: # (File containing markdown style reference definitions to be included in each generated doc)
