
<a name="0x1_RegisteredCurrencies"></a>

# Module `0x1::RegisteredCurrencies`

### Table of Contents

-  [Struct `RegisteredCurrencies`](#0x1_RegisteredCurrencies_RegisteredCurrencies)
-  [Function `initialize`](#0x1_RegisteredCurrencies_initialize)
-  [Function `add_currency_code`](#0x1_RegisteredCurrencies_add_currency_code)
-  [Specification](#0x1_RegisteredCurrencies_Specification)
    -  [Function `initialize`](#0x1_RegisteredCurrencies_Specification_initialize)
    -  [Function `add_currency_code`](#0x1_RegisteredCurrencies_Specification_add_currency_code)

Module managing the registered currencies in the Libra framework.


<a name="0x1_RegisteredCurrencies_RegisteredCurrencies"></a>

## Struct `RegisteredCurrencies`

An on-chain config holding all of the currency codes for registered
currencies. The inner vector<u8>'s are string representations of
currency names.


<pre><code><b>struct</b> <a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>
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

<a name="0x1_RegisteredCurrencies_initialize"></a>

## Function `initialize`

Initializes this module. Can only be called from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);

    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(),
        EINVALID_SINGLETON_ADDRESS
    );
    <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>(
        config_account,
        <a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a> { currency_codes: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>() }
    );


}
</code></pre>



</details>

<a name="0x1_RegisteredCurrencies_add_currency_code"></a>

## Function `add_currency_code`

Adds a new currency code. The currency code must not yet exist.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_add_currency_code">add_currency_code</a>(lr_account: &signer, currency_code: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_add_currency_code">add_currency_code</a>(
    lr_account: &signer,
    currency_code: vector&lt;u8&gt;,
) {
    <b>let</b> config = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;();
    <b>assert</b>(
        !<a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&config.currency_codes, &currency_code),
        ECURRENCY_CODE_ALREADY_TAKEN
    );
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> config.currency_codes, currency_code);
    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>(lr_account, config);
}
</code></pre>



</details>

<a name="0x1_RegisteredCurrencies_Specification"></a>

## Specification


<a name="0x1_RegisteredCurrencies_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer)
</code></pre>




<pre><code>pragma aborts_if_is_partial;
</code></pre>


Function aborts if already initialized or not in genesis.


<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(config_account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>();
<b>aborts_if</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;();
<b>aborts_if</b> !spec_is_genesis();
<b>ensures</b> <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;();
<b>ensures</b> len(<a href="#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>()) == 0;
</code></pre>



<a name="0x1_RegisteredCurrencies_Specification_add_currency_code"></a>

### Function `add_currency_code`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_add_currency_code">add_currency_code</a>(lr_account: &signer, currency_code: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf">AddCurrencyCodeAbortsIf</a>;
</code></pre>


The resulting currency_codes is the one before this function is called, with the new one added to the end.


<pre><code><b>ensures</b> <a href="Vector.md#0x1_Vector_eq_push_back">Vector::eq_push_back</a>(<a href="#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>(), <b>old</b>(<a href="#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>()), currency_code);
</code></pre>




<a name="0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf">AddCurrencyCodeAbortsIf</a> {
    lr_account: &signer;
    currency_code: vector&lt;u8&gt;;
    <b>aborts_if</b> !<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;();
    <b>aborts_if</b> !exists&lt;<a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(lr_account));
}
</code></pre>


The same currency code can be only added once.


<pre><code><b>schema</b> <a href="#0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf">AddCurrencyCodeAbortsIf</a> {
    <b>aborts_if</b> <a href="Vector.md#0x1_Vector_spec_contains">Vector::spec_contains</a>(
        <a href="LibraConfig.md#0x1_LibraConfig_spec_get">LibraConfig::spec_get</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;().currency_codes,
        currency_code
    );
}
</code></pre>




<pre><code>pragma verify = <b>true</b>;
</code></pre>


Helper to get the currency code vector.


<a name="0x1_RegisteredCurrencies_get_currency_codes"></a>


<pre><code><b>define</b> <a href="#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>(): vector&lt;vector&lt;u8&gt;&gt; {
    <a href="LibraConfig.md#0x1_LibraConfig_spec_get">LibraConfig::spec_get</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;().currency_codes
}
</code></pre>


Global invariant that currency config is always available after genesis.


<pre><code><b>invariant</b> [<b>global</b>] !spec_is_genesis() ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;();
</code></pre>


Global invariant that only LIBRA_ROOT can have a currency registration.


<pre><code><b>invariant</b> [<b>global</b>] !spec_is_genesis() ==&gt; (
    forall holder: address where exists&lt;<a href="LibraConfig.md#0x1_LibraConfig_LibraConfig">LibraConfig::LibraConfig</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;&gt;(holder):
        holder == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()
);
</code></pre>
