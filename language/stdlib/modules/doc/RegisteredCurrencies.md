
<a name="0x1_RegisteredCurrencies"></a>

# Module `0x1::RegisteredCurrencies`

### Table of Contents

-  [Struct `RegisteredCurrencies`](#0x1_RegisteredCurrencies_RegisteredCurrencies)
-  [Struct `RegistrationCapability`](#0x1_RegisteredCurrencies_RegistrationCapability)
-  [Function `initialize`](#0x1_RegisteredCurrencies_initialize)
-  [Function `empty`](#0x1_RegisteredCurrencies_empty)
-  [Function `add_currency_code`](#0x1_RegisteredCurrencies_add_currency_code)
-  [Function `singleton_address`](#0x1_RegisteredCurrencies_singleton_address)
-  [Specification](#0x1_RegisteredCurrencies_Specification)
    -  [Module specifications](#0x1_RegisteredCurrencies_@Module_specifications)
    -  [Function `initialize`](#0x1_RegisteredCurrencies_Specification_initialize)
        -  [Initialization](#0x1_RegisteredCurrencies_@Initialization)
        -  [Uniqueness of the RegisteredCurrencies config.](#0x1_RegisteredCurrencies_@Uniqueness_of_the_RegisteredCurrencies_config.)
        -  [Currency codes](#0x1_RegisteredCurrencies_@Currency_codes)



<a name="0x1_RegisteredCurrencies_RegisteredCurrencies"></a>

## Struct `RegisteredCurrencies`



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

<a name="0x1_RegisteredCurrencies_RegistrationCapability"></a>

## Struct `RegistrationCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="LibraConfig.md#0x1_LibraConfig_ModifyConfigCapability">LibraConfig::ModifyConfigCapability</a>&lt;<a href="#0x1_RegisteredCurrencies_RegisteredCurrencies">RegisteredCurrencies::RegisteredCurrencies</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_RegisteredCurrencies_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer): <a href="#0x1_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer): <a href="#0x1_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a> {
    // enforce that this is only going <b>to</b> one specific address,
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account) == <a href="#0x1_RegisteredCurrencies_singleton_address">singleton_address</a>(),
        0
    );
    <b>let</b> cap = <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config_with_capability">LibraConfig::publish_new_config_with_capability</a>(config_account, <a href="#0x1_RegisteredCurrencies_empty">empty</a>());

    <a href="#0x1_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a> { cap }
}
</code></pre>



</details>

<a name="0x1_RegisteredCurrencies_empty"></a>

## Function `empty`



<pre><code><b>fun</b> <a href="#0x1_RegisteredCurrencies_empty">empty</a>(): <a href="#0x1_RegisteredCurrencies_RegisteredCurrencies">RegisteredCurrencies::RegisteredCurrencies</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_RegisteredCurrencies_empty">empty</a>(): <a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a> {
    <a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a> { currency_codes: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>() }
}
</code></pre>



</details>

<a name="0x1_RegisteredCurrencies_add_currency_code"></a>

## Function `add_currency_code`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_add_currency_code">add_currency_code</a>(currency_code: vector&lt;u8&gt;, cap: &<a href="#0x1_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_add_currency_code">add_currency_code</a>(
    currency_code: vector&lt;u8&gt;,
    cap: &<a href="#0x1_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a>,
) {
    <b>let</b> config = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;();
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> config.currency_codes, currency_code);
    <a href="LibraConfig.md#0x1_LibraConfig_set_with_capability">LibraConfig::set_with_capability</a>(&cap.cap, config);
}
</code></pre>



</details>

<a name="0x1_RegisteredCurrencies_singleton_address"></a>

## Function `singleton_address`

**Q:** Do we need this function, instead of using default_config_address directly?


<pre><code><b>fun</b> <a href="#0x1_RegisteredCurrencies_singleton_address">singleton_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_RegisteredCurrencies_singleton_address">singleton_address</a>(): address {
    <a href="CoreAddresses.md#0x1_CoreAddresses_DEFAULT_CONFIG_ADDRESS">CoreAddresses::DEFAULT_CONFIG_ADDRESS</a>()
}
</code></pre>



</details>

<a name="0x1_RegisteredCurrencies_Specification"></a>

## Specification


<a name="0x1_RegisteredCurrencies_@Module_specifications"></a>

### Module specifications



<pre><code>pragma verify = <b>true</b>;
<a name="0x1_RegisteredCurrencies_spec_singleton_address"></a>
<b>define</b> <a href="#0x1_RegisteredCurrencies_spec_singleton_address">spec_singleton_address</a>(): address { 0xF1A95 }
<a name="0x1_RegisteredCurrencies_spec_is_initialized"></a>
<b>define</b> <a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>():bool {
    <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;(<a href="#0x1_RegisteredCurrencies_spec_singleton_address">spec_singleton_address</a>())
}
</code></pre>



<a name="0x1_RegisteredCurrencies_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer): <a href="#0x1_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a>
</code></pre>



<a name="0x1_RegisteredCurrencies_@Initialization"></a>

#### Initialization


After
<code>initialize</code> is called, the module is initialized.


<pre><code>pragma aborts_if_is_partial = <b>true</b>;
<b>aborts_if</b> <a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>();
<b>ensures</b> <a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>();
</code></pre>




<a name="0x1_RegisteredCurrencies_InitializationPersists"></a>

*Informally:* Once initialize is run, the module continues to be
initialized, forever.


<pre><code><b>schema</b> <a href="#0x1_RegisteredCurrencies_InitializationPersists">InitializationPersists</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>()) ==&gt; <a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>();
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_RegisteredCurrencies_InitializationPersists">InitializationPersists</a> <b>to</b> *;
</code></pre>



<a name="0x1_RegisteredCurrencies_@Uniqueness_of_the_RegisteredCurrencies_config."></a>

#### Uniqueness of the RegisteredCurrencies config.



<a name="0x1_RegisteredCurrencies_OnlySingletonHasRegisteredCurrencies"></a>


<pre><code><b>schema</b> <a href="#0x1_RegisteredCurrencies_OnlySingletonHasRegisteredCurrencies">OnlySingletonHasRegisteredCurrencies</a> {
    <b>invariant</b> !<a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>()
        ==&gt; (forall addr: address: !<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;(addr));
    <b>invariant</b> <a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>()
        ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;(<a href="#0x1_RegisteredCurrencies_spec_singleton_address">spec_singleton_address</a>())
            && (forall addr: address:
                   <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;(addr)
                              ==&gt; addr == <a href="#0x1_RegisteredCurrencies_spec_singleton_address">spec_singleton_address</a>());
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_RegisteredCurrencies_OnlySingletonHasRegisteredCurrencies">OnlySingletonHasRegisteredCurrencies</a> <b>to</b> *;
</code></pre>



<a name="0x1_RegisteredCurrencies_@Currency_codes"></a>

#### Currency codes

Attempting to specify that only
<code>add_currency</code> changes the currency_codes
vector.
**Confused:** I think
<code>initialize</code> should violate this property unless it
checks whether the module is already initialized, because it can be
called a second time, overwriting existing currency_codes.


<a name="0x1_RegisteredCurrencies_OnlyAddCurrencyChangesT"></a>


<pre><code><b>schema</b> <a href="#0x1_RegisteredCurrencies_OnlyAddCurrencyChangesT">OnlyAddCurrencyChangesT</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>())
                 ==&gt; <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_get">LibraConfig::spec_get</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;().currency_codes)
                      == <a href="LibraConfig.md#0x1_LibraConfig_spec_get">LibraConfig::spec_get</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;().currency_codes;
}
</code></pre>




<code>add_currency_code</code> and
<code>initialize</code> change the currency_code vector.


<pre><code><b>apply</b> <a href="#0x1_RegisteredCurrencies_OnlyAddCurrencyChangesT">OnlyAddCurrencyChangesT</a> <b>to</b> * <b>except</b> add_currency_code;
</code></pre>
