
<a name="0x0_VASPRegistry"></a>

# Module `0x0::VASPRegistry`

### Table of Contents

-  [Struct `VASPRegistry`](#0x0_VASPRegistry_VASPRegistry)
-  [Struct `VASPRegistrationCapability`](#0x0_VASPRegistry_VASPRegistrationCapability)
-  [Function `initialize`](#0x0_VASPRegistry_initialize)
-  [Function `registered_vasp_addresses`](#0x0_VASPRegistry_registered_vasp_addresses)
-  [Function `remove_vasp`](#0x0_VASPRegistry_remove_vasp)
-  [Function `add_vasp`](#0x0_VASPRegistry_add_vasp)



<a name="0x0_VASPRegistry_VASPRegistry"></a>

## Struct `VASPRegistry`



<pre><code><b>struct</b> <a href="#0x0_VASPRegistry">VASPRegistry</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>registered_vasps: vector&lt;address&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_VASPRegistry_VASPRegistrationCapability"></a>

## Struct `VASPRegistrationCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_VASPRegistry_VASPRegistrationCapability">VASPRegistrationCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="libra_configs.md#0x0_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;<a href="#0x0_VASPRegistry_VASPRegistry">VASPRegistry::VASPRegistry</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_VASPRegistry_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPRegistry_initialize">initialize</a>(config_account: &signer): <a href="#0x0_VASPRegistry_VASPRegistrationCapability">VASPRegistry::VASPRegistrationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPRegistry_initialize">initialize</a>(config_account: &signer): <a href="#0x0_VASPRegistry_VASPRegistrationCapability">VASPRegistrationCapability</a> {
    // enforce that this is only going <b>to</b> one specific address,
    Transaction::assert(
        <a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(config_account) == <a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>(),
        0
    );
    <b>let</b> cap = <a href="libra_configs.md#0x0_LibraConfig_publish_new_config_with_capability">LibraConfig::publish_new_config_with_capability</a>(
                 <a href="#0x0_VASPRegistry">VASPRegistry</a> {
                     registered_vasps: <a href="vector.md#0x0_Vector_empty">Vector::empty</a>(),
                 },
                 config_account
              );

    <a href="#0x0_VASPRegistry_VASPRegistrationCapability">VASPRegistrationCapability</a> { cap }
}
</code></pre>



</details>

<a name="0x0_VASPRegistry_registered_vasp_addresses"></a>

## Function `registered_vasp_addresses`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPRegistry_registered_vasp_addresses">registered_vasp_addresses</a>(): vector&lt;address&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPRegistry_registered_vasp_addresses">registered_vasp_addresses</a>(): vector&lt;address&gt; {
    <b>let</b> config = <a href="libra_configs.md#0x0_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x0_VASPRegistry">VASPRegistry</a>&gt;();
    *&config.registered_vasps
}
</code></pre>



</details>

<a name="0x0_VASPRegistry_remove_vasp"></a>

## Function `remove_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPRegistry_remove_vasp">remove_vasp</a>(parent_vasp_addr: address, cap: &<a href="#0x0_VASPRegistry_VASPRegistrationCapability">VASPRegistry::VASPRegistrationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPRegistry_remove_vasp">remove_vasp</a>(
    parent_vasp_addr: address,
    cap: &<a href="#0x0_VASPRegistry_VASPRegistrationCapability">VASPRegistrationCapability</a>,
) {
    <b>let</b> config = <a href="libra_configs.md#0x0_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x0_VASPRegistry">VASPRegistry</a>&gt;();
    <b>let</b> (found, index) = <a href="vector.md#0x0_Vector_index_of">Vector::index_of</a>(&config.registered_vasps, &parent_vasp_addr);
    <b>if</b> (found) { <b>let</b> _ = <a href="vector.md#0x0_Vector_swap_remove">Vector::swap_remove</a>(&<b>mut</b> config.registered_vasps, index) };
    <a href="libra_configs.md#0x0_LibraConfig_set_with_capability">LibraConfig::set_with_capability</a>(&cap.cap, config);
}
</code></pre>



</details>

<a name="0x0_VASPRegistry_add_vasp"></a>

## Function `add_vasp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPRegistry_add_vasp">add_vasp</a>(parent_vasp_addr: address, cap: &<a href="#0x0_VASPRegistry_VASPRegistrationCapability">VASPRegistry::VASPRegistrationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPRegistry_add_vasp">add_vasp</a>(
    parent_vasp_addr: address,
    cap: &<a href="#0x0_VASPRegistry_VASPRegistrationCapability">VASPRegistrationCapability</a>,
) {
    <b>let</b> config = <a href="libra_configs.md#0x0_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x0_VASPRegistry">VASPRegistry</a>&gt;();
    <a href="vector.md#0x0_Vector_push_back">Vector::push_back</a>(&<b>mut</b> config.registered_vasps, parent_vasp_addr);
    <a href="libra_configs.md#0x0_LibraConfig_set_with_capability">LibraConfig::set_with_capability</a>(&cap.cap, config);
}
</code></pre>



</details>
