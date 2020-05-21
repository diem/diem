
<a name="0x0_RegisteredCurrencies"></a>

# Module `0x0::RegisteredCurrencies`

### Table of Contents

-  [Struct `T`](#0x0_RegisteredCurrencies_T)
-  [Struct `RegistrationCapability`](#0x0_RegisteredCurrencies_RegistrationCapability)
-  [Function `initialize`](#0x0_RegisteredCurrencies_initialize)
-  [Function `empty`](#0x0_RegisteredCurrencies_empty)
-  [Function `add_currency_code`](#0x0_RegisteredCurrencies_add_currency_code)
-  [Function `singleton_address`](#0x0_RegisteredCurrencies_singleton_address)



<a name="0x0_RegisteredCurrencies_T"></a>

## Struct `T`



<pre><code><b>struct</b> <a href="#0x0_RegisteredCurrencies_T">T</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>currency_codes: vector&lt;vector&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_RegisteredCurrencies_RegistrationCapability"></a>

## Struct `RegistrationCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="libra_configs.md#0x0_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;<a href="#0x0_RegisteredCurrencies_T">RegisteredCurrencies::T</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_RegisteredCurrencies_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_RegisteredCurrencies_initialize">initialize</a>(): <a href="#0x0_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_RegisteredCurrencies_initialize">initialize</a>(): <a href="#0x0_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a> {
    // enforce that this is only going <b>to</b> one specific address,
    Transaction::assert(Transaction::sender() == <a href="#0x0_RegisteredCurrencies_singleton_address">singleton_address</a>(), 0);
    <b>let</b> cap = <a href="libra_configs.md#0x0_LibraConfig_publish_new_config_with_capability">LibraConfig::publish_new_config_with_capability</a>(<a href="#0x0_RegisteredCurrencies_empty">empty</a>());

    <a href="#0x0_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a>{ cap }
}
</code></pre>



</details>

<a name="0x0_RegisteredCurrencies_empty"></a>

## Function `empty`



<pre><code><b>fun</b> <a href="#0x0_RegisteredCurrencies_empty">empty</a>(): <a href="#0x0_RegisteredCurrencies_T">RegisteredCurrencies::T</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_RegisteredCurrencies_empty">empty</a>(): <a href="#0x0_RegisteredCurrencies_T">T</a> {
    <a href="#0x0_RegisteredCurrencies_T">T</a> { currency_codes: <a href="vector.md#0x0_Vector_empty">Vector::empty</a>() }
}
</code></pre>



</details>

<a name="0x0_RegisteredCurrencies_add_currency_code"></a>

## Function `add_currency_code`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_RegisteredCurrencies_add_currency_code">add_currency_code</a>(currency_code: vector&lt;u8&gt;, cap: &<a href="#0x0_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_RegisteredCurrencies_add_currency_code">add_currency_code</a>(
    currency_code: vector&lt;u8&gt;,
    cap: &<a href="#0x0_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a>,
) {
    <b>let</b> config = <a href="libra_configs.md#0x0_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x0_RegisteredCurrencies_T">T</a>&gt;();
    <a href="vector.md#0x0_Vector_push_back">Vector::push_back</a>(&<b>mut</b> config.currency_codes, currency_code);
    <a href="libra_configs.md#0x0_LibraConfig_set_with_capability">LibraConfig::set_with_capability</a>(&cap.cap, config);
}
</code></pre>



</details>

<a name="0x0_RegisteredCurrencies_singleton_address"></a>

## Function `singleton_address`



<pre><code><b>fun</b> <a href="#0x0_RegisteredCurrencies_singleton_address">singleton_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_RegisteredCurrencies_singleton_address">singleton_address</a>(): address {
    <a href="libra_configs.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>()
}
</code></pre>



</details>
