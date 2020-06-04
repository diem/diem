
<a name="0x0_RegisteredCurrencies"></a>

# Module `0x0::RegisteredCurrencies`

### Table of Contents

-  [Struct `T`](#0x0_RegisteredCurrencies_T)
-  [Struct `RegistrationCapability`](#0x0_RegisteredCurrencies_RegistrationCapability)
-  [Function `initialize`](#0x0_RegisteredCurrencies_initialize)
-  [Function `empty`](#0x0_RegisteredCurrencies_empty)
-  [Function `add_currency_code`](#0x0_RegisteredCurrencies_add_currency_code)
-  [Function `singleton_address`](#0x0_RegisteredCurrencies_singleton_address)
-  [Specification](#0x0_RegisteredCurrencies_Specification)
    -  [Module specifications](#0x0_RegisteredCurrencies_@Module_specifications)
    -  [Function `initialize`](#0x0_RegisteredCurrencies_Specification_initialize)



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

<code>cap: <a href="LibraConfig.md#0x0_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;<a href="#0x0_RegisteredCurrencies_T">RegisteredCurrencies::T</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_RegisteredCurrencies_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer): <a href="#0x0_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer): <a href="#0x0_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a> {
    // enforce that this is only going <b>to</b> one specific address,
    Transaction::assert(
        <a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(config_account) == <a href="LibraConfig.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>(),
        0
    );
    <b>let</b> cap = <a href="LibraConfig.md#0x0_LibraConfig_publish_new_config_with_capability">LibraConfig::publish_new_config_with_capability</a>(config_account, <a href="#0x0_RegisteredCurrencies_empty">empty</a>());

    <a href="#0x0_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a> { cap }
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
    <a href="#0x0_RegisteredCurrencies_T">T</a> { currency_codes: <a href="Vector.md#0x0_Vector_empty">Vector::empty</a>() }
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
    <b>let</b> config = <a href="LibraConfig.md#0x0_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x0_RegisteredCurrencies_T">T</a>&gt;();
    <a href="Vector.md#0x0_Vector_push_back">Vector::push_back</a>(&<b>mut</b> config.currency_codes, currency_code);
    <a href="LibraConfig.md#0x0_LibraConfig_set_with_capability">LibraConfig::set_with_capability</a>(&cap.cap, config);
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
    <a href="LibraConfig.md#0x0_LibraConfig_default_config_address">LibraConfig::default_config_address</a>()
}
</code></pre>



</details>

<a name="0x0_RegisteredCurrencies_Specification"></a>

## Specification


<a name="0x0_RegisteredCurrencies_@Module_specifications"></a>

### Module specifications



<pre><code>pragma verify = <b>false</b>;
<a name="0x0_RegisteredCurrencies_spec_singleton_address"></a>
<b>define</b> <a href="#0x0_RegisteredCurrencies_spec_singleton_address">spec_singleton_address</a>():address { <a href="LibraConfig.md#0x0_LibraConfig_spec_default_config_address">LibraConfig::spec_default_config_address</a>() }
<a name="0x0_RegisteredCurrencies_spec_is_initialized"></a>
<b>define</b> <a href="#0x0_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>():bool { <a href="LibraConfig.md#0x0_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x0_RegisteredCurrencies_T">T</a>&gt;(<a href="#0x0_RegisteredCurrencies_spec_singleton_address">spec_singleton_address</a>()) }
</code></pre>



<a name="0x0_RegisteredCurrencies_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer): <a href="#0x0_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a>
</code></pre>




<pre><code><b>ensures</b> !<a href="#0x0_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>();
<b>ensures</b> <a href="#0x0_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>();
</code></pre>




<a name="0x0_RegisteredCurrencies_OnlySingletonHasT"></a>


<pre><code><b>schema</b> <a href="#0x0_RegisteredCurrencies_OnlySingletonHasT">OnlySingletonHasT</a> {
    <b>invariant</b> !<a href="#0x0_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>()
        ==&gt; all(domain&lt;address&gt;(), |addr| !<a href="LibraConfig.md#0x0_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x0_RegisteredCurrencies_T">T</a>&gt;(addr));
    <b>invariant</b> <a href="#0x0_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>()
        ==&gt; <a href="LibraConfig.md#0x0_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x0_RegisteredCurrencies_T">T</a>&gt;(sender())
            && all(domain&lt;address&gt;(),
                   |addr| <a href="LibraConfig.md#0x0_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x0_RegisteredCurrencies_T">T</a>&gt;(addr)
                              ==&gt; addr == <a href="#0x0_RegisteredCurrencies_spec_singleton_address">spec_singleton_address</a>());
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x0_RegisteredCurrencies_OnlySingletonHasT">OnlySingletonHasT</a> <b>to</b> *;
</code></pre>




<a name="0x0_RegisteredCurrencies_OnlyAddCurrencyChangesT"></a>


<pre><code><b>schema</b> <a href="#0x0_RegisteredCurrencies_OnlyAddCurrencyChangesT">OnlyAddCurrencyChangesT</a> {
    <b>ensures</b> <a href="#0x0_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>()
                 ==&gt; <b>old</b>(<a href="LibraConfig.md#0x0_LibraConfig_spec_get">LibraConfig::spec_get</a>&lt;<a href="#0x0_RegisteredCurrencies_T">T</a>&gt;().currency_codes)
                      == <a href="LibraConfig.md#0x0_LibraConfig_spec_get">LibraConfig::spec_get</a>&lt;<a href="#0x0_RegisteredCurrencies_T">T</a>&gt;().currency_codes;
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x0_RegisteredCurrencies_OnlyAddCurrencyChangesT">OnlyAddCurrencyChangesT</a> <b>to</b> * <b>except</b> add_currency_code;
</code></pre>
