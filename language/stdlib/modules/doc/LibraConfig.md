
<a name="0x1_LibraConfig"></a>

# Module `0x1::LibraConfig`

Publishes configuration information for validators, and issues reconfiguration events
to synchronize configuration changes for the validators.


-  [Resource `LibraConfig`](#0x1_LibraConfig_LibraConfig)
-  [Struct `NewEpochEvent`](#0x1_LibraConfig_NewEpochEvent)
-  [Resource `Configuration`](#0x1_LibraConfig_Configuration)
-  [Resource `ModifyConfigCapability`](#0x1_LibraConfig_ModifyConfigCapability)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_LibraConfig_initialize)
-  [Function `get`](#0x1_LibraConfig_get)
-  [Function `set`](#0x1_LibraConfig_set)
-  [Function `set_with_capability_and_reconfigure`](#0x1_LibraConfig_set_with_capability_and_reconfigure)
-  [Function `publish_new_config_and_get_capability`](#0x1_LibraConfig_publish_new_config_and_get_capability)
-  [Function `publish_new_config`](#0x1_LibraConfig_publish_new_config)
-  [Function `reconfigure`](#0x1_LibraConfig_reconfigure)
-  [Function `reconfigure_`](#0x1_LibraConfig_reconfigure_)
-  [Function `emit_genesis_reconfiguration_event`](#0x1_LibraConfig_emit_genesis_reconfiguration_event)
-  [Module Specification](#@Module_Specification_1)
    -  [Initialization](#@Initialization_2)
    -  [Invariants](#@Invariants_3)
    -  [Helper Functions](#@Helper_Functions_4)


<pre><code><b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp">0x1::LibraTimestamp</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_LibraConfig_LibraConfig"></a>

## Resource `LibraConfig`

A generic singleton resource that holds a value of a specific type.


<pre><code><b>resource</b> <b>struct</b> <a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config: <b>copyable</b>&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>payload: Config</code>
</dt>
<dd>
 Holds specific info for instance of <code>Config</code> type.
</dd>
</dl>


</details>

<a name="0x1_LibraConfig_NewEpochEvent"></a>

## Struct `NewEpochEvent`

Event that signals LibraBFT algorithm to start a new epoch,
with new configuration information. This is also called a
"reconfiguration event"


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

Holds information about state of reconfiguration


<pre><code><b>resource</b> <b>struct</b> <a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>epoch: u64</code>
</dt>
<dd>
 Epoch number
</dd>
<dt>
<code>last_reconfiguration_time: u64</code>
</dt>
<dd>
 Time of last reconfiguration. Only changes on reconfiguration events.
</dd>
<dt>
<code>events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">LibraConfig::NewEpochEvent</a>&gt;</code>
</dt>
<dd>
 Event handle for reconfiguration events
</dd>
</dl>


</details>

<a name="0x1_LibraConfig_ModifyConfigCapability"></a>

## Resource `ModifyConfigCapability`

Accounts with this privilege can modify LibraConfig<TypeName> under Libra root address.


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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_LibraConfig_MAX_U64"></a>

The largest possible u64 value


<pre><code><b>const</b> <a href="LibraConfig.md#0x1_LibraConfig_MAX_U64">MAX_U64</a>: u64 = 18446744073709551615;
</code></pre>



<a name="0x1_LibraConfig_ECONFIGURATION"></a>

The <code><a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="LibraConfig.md#0x1_LibraConfig_ECONFIGURATION">ECONFIGURATION</a>: u64 = 0;
</code></pre>



<a name="0x1_LibraConfig_EINVALID_BLOCK_TIME"></a>

An invalid block time was encountered.


<pre><code><b>const</b> <a href="LibraConfig.md#0x1_LibraConfig_EINVALID_BLOCK_TIME">EINVALID_BLOCK_TIME</a>: u64 = 4;
</code></pre>



<a name="0x1_LibraConfig_ELIBRA_CONFIG"></a>

A <code><a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="LibraConfig.md#0x1_LibraConfig_ELIBRA_CONFIG">ELIBRA_CONFIG</a>: u64 = 1;
</code></pre>



<a name="0x1_LibraConfig_EMODIFY_CAPABILITY"></a>

A <code><a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a></code> is in a different state than was expected


<pre><code><b>const</b> <a href="LibraConfig.md#0x1_LibraConfig_EMODIFY_CAPABILITY">EMODIFY_CAPABILITY</a>: u64 = 2;
</code></pre>



<a name="0x1_LibraConfig_initialize"></a>

## Function `initialize`

Publishes <code><a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a></code> resource. Can only be invoked by Libra root, and only a single time in Genesis.


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
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_InitializeEnsures">InitializeEnsures</a>;
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>




<a name="0x1_LibraConfig_InitializeAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_InitializeAbortsIf">InitializeAbortsIf</a> {
    lr_account: signer;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
    <b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>{account: lr_account};
    <b>aborts_if</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>() <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
}
</code></pre>




<a name="0x1_LibraConfig_InitializeEnsures"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_InitializeEnsures">InitializeEnsures</a> {
    <b>ensures</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>();
    <a name="0x1_LibraConfig_new_config$13"></a>
    <b>let</b> new_config = <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <b>ensures</b> new_config.epoch == 0;
    <b>ensures</b> new_config.last_reconfiguration_time == 0;
}
</code></pre>



</details>

<a name="0x1_LibraConfig_get"></a>

## Function `get`

Returns a copy of <code>Config</code> value stored under <code>addr</code>.


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

Set a config item to a new value with the default capability stored under config address and trigger a
reconfiguration. This function requires that the signer be Libra root.


<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_set">set</a>&lt;Config: <b>copyable</b>&gt;(account: &signer, payload: Config)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraConfig.md#0x1_LibraConfig_set">set</a>&lt;Config: <b>copyable</b>&gt;(account: &signer, payload: Config)
<b>acquires</b> <a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>, <a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a> {
    <b>let</b> signer_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Next should always be <b>true</b> <b>if</b> properly initialized.
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
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
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
    <b>ensures</b> <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>()) == <a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>();
}
</code></pre>



</details>

<a name="0x1_LibraConfig_set_with_capability_and_reconfigure"></a>

## Function `set_with_capability_and_reconfigure`

Set a config item to a new value and trigger a reconfiguration. This function
requires a reference to a <code><a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">ModifyConfigCapability</a></code>, which is returned when the
config is published using <code>publish_new_config_and_get_capability</code>.
It is called by <code><a href="LibraSystem.md#0x1_LibraSystem_update_config_and_reconfigure">LibraSystem::update_config_and_reconfigure</a></code>, which allows
validator operators to change the validator set.  All other config changes require
a Libra root signer.


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
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">AbortsIfNotPublished</a>&lt;Config&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">ReconfigureAbortsIf</a>;
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;Config&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetEnsures">SetEnsures</a>&lt;Config&gt;;
</code></pre>



</details>

<a name="0x1_LibraConfig_publish_new_config_and_get_capability"></a>

## Function `publish_new_config_and_get_capability`

Publishes a new config.
The caller will use the returned ModifyConfigCapability to specify the access control
policy for who can modify the config.
Does not trigger a reconfiguration.


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

Publish a new config item. Only Libra root can modify such config.
Publishes the capability to modify this config under the Libra root account.
Does not trigger a reconfiguration.


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

Signal validators to start using new configuration. Must be called by Libra root.


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
<b>modifies</b> <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">ReconfigureAbortsIf</a>;
</code></pre>



</details>

<a name="0x1_LibraConfig_reconfigure_"></a>

## Function `reconfigure_`

Private function to do reconfiguration.  Updates reconfiguration status resource
<code><a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a></code> and emits a <code><a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a></code>


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
    //
    // This is OK because:
    // - The time changes in every non-empty block
    // - A block automatically ends after a transaction that emits a reconfiguration event, which is guaranteed by
    //   LibraVM <b>spec</b> that all transactions comming after a reconfiguration transaction will be returned <b>as</b> Retry
    //   status.
    // - Each transaction must emit at most one reconfiguration event
    //
    // Thus, this check <b>ensures</b> that a transaction that does multiple "reconfiguration required" actions emits only
    // one reconfiguration event.
    //
    <b>if</b> (current_time == config_ref.last_reconfiguration_time) {
        <b>return</b>
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
<b>ensures</b> <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>()) == <a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>();
<a name="0x1_LibraConfig_config$18"></a>
<b>let</b> config = <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<a name="0x1_LibraConfig_now$19"></a>
<b>let</b> now = <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>();
<a name="0x1_LibraConfig_epoch$20"></a>
<b>let</b> epoch = config.epoch;
<b>include</b> !<a href="LibraConfig.md#0x1_LibraConfig_spec_reconfigure_omitted">spec_reconfigure_omitted</a>() || (config.last_reconfiguration_time == now)
    ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_InternalReconfigureAbortsIf">InternalReconfigureAbortsIf</a> && <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">ReconfigureAbortsIf</a>;
<b>ensures</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_reconfigure_omitted">spec_reconfigure_omitted</a>() || (<b>old</b>(config).last_reconfiguration_time == now)
    ==&gt; config == <b>old</b>(config);
<b>ensures</b> !(<a href="LibraConfig.md#0x1_LibraConfig_spec_reconfigure_omitted">spec_reconfigure_omitted</a>() || (config.last_reconfiguration_time == now))
    ==&gt; config ==
        update_field(
        update_field(<b>old</b>(config),
            epoch, <b>old</b>(config.epoch) + 1),
            last_reconfiguration_time, now);
</code></pre>


The following schema describes aborts conditions which we do not want to be propagated to the verification
of callers, and which are therefore marked as <code>concrete</code> to be only verified against the implementation.
These conditions are unlikely to happen in reality, and excluding them avoids formal noise.


<a name="0x1_LibraConfig_InternalReconfigureAbortsIf"></a>


<a name="0x1_LibraConfig_config$16"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_InternalReconfigureAbortsIf">InternalReconfigureAbortsIf</a> {
    <b>let</b> config = <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <a name="0x1_LibraConfig_current_time$17"></a>
    <b>let</b> current_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>();
    <b>aborts_if</b> [concrete] current_time &lt; config.last_reconfiguration_time <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
    <b>aborts_if</b> [concrete] config.epoch == <a href="LibraConfig.md#0x1_LibraConfig_MAX_U64">MAX_U64</a>
        && current_time != config.last_reconfiguration_time <b>with</b> EXECUTION_FAILURE;
}
</code></pre>


This schema is to be used by callers of <code>reconfigure</code>


<a name="0x1_LibraConfig_ReconfigureAbortsIf"></a>


<a name="0x1_LibraConfig_config$14"></a>


<pre><code><b>schema</b> <a href="LibraConfig.md#0x1_LibraConfig_ReconfigureAbortsIf">ReconfigureAbortsIf</a> {
    <b>let</b> config = <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <a name="0x1_LibraConfig_current_time$15"></a>
    <b>let</b> current_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>();
    <b>aborts_if</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>()
        && <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>() &gt; 0
        && config.epoch &lt; <a href="LibraConfig.md#0x1_LibraConfig_MAX_U64">MAX_U64</a>
        && current_time &lt; config.last_reconfiguration_time
            <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
}
</code></pre>



</details>

<a name="0x1_LibraConfig_emit_genesis_reconfiguration_event"></a>

## Function `emit_genesis_reconfiguration_event`

Emit a <code><a href="LibraConfig.md#0x1_LibraConfig_NewEpochEvent">NewEpochEvent</a></code> event. This function will be invoked by genesis directly to generate the very first
reconfiguration event.


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

<a name="@Module_Specification_1"></a>

## Module Specification



<a name="0x1_LibraConfig_spec_reconfigure_omitted"></a>


<pre><code><b>define</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_reconfigure_omitted">spec_reconfigure_omitted</a>(): bool {
  <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>() || <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>() == 0
}
</code></pre>




<a name="@Initialization_2"></a>

### Initialization


After genesis, the <code><a href="LibraConfig.md#0x1_LibraConfig_Configuration">Configuration</a></code> is published.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_spec_has_config">spec_has_config</a>();
</code></pre>



<a name="@Invariants_3"></a>

### Invariants


Configurations are only stored at the libra root address.


<pre><code><b>invariant</b> [<b>global</b>]
    <b>forall</b> config_address: address, config_type: type <b>where</b> <b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;config_type&gt;&gt;(config_address):
        config_address == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
</code></pre>


After genesis, no new configurations are added.


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>]
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
        (<b>forall</b> config_type: type <b>where</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;config_type&gt;(): <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;config_type&gt;()));
</code></pre>


Published configurations are persistent.


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>]
    (<b>forall</b> config_type: type <b>where</b> <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;config_type&gt;()): <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">spec_is_published</a>&lt;config_type&gt;());
</code></pre>



<a name="@Helper_Functions_4"></a>

### Helper Functions



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


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/libra/lip/blob/master/lips/lip-2.md
[ROLE]: https://github.com/libra/lip/blob/master/lips/lip-2.md#roles
[PERMISSION]: https://github.com/libra/lip/blob/master/lips/lip-2.md#permissions
