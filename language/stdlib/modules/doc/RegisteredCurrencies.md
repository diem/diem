
<a name="0x1_RegisteredCurrencies"></a>

# Module `0x1::RegisteredCurrencies`

### Table of Contents

-  [Struct `RegisteredCurrencies`](#0x1_RegisteredCurrencies_RegisteredCurrencies)
-  [Resource `RegistrationCapability`](#0x1_RegisteredCurrencies_RegistrationCapability)
-  [Function `initialize`](#0x1_RegisteredCurrencies_initialize)
-  [Function `empty`](#0x1_RegisteredCurrencies_empty)
-  [Function `add_currency_code`](#0x1_RegisteredCurrencies_add_currency_code)
-  [Specification](#0x1_RegisteredCurrencies_Specification)
    -  [Module specifications](#0x1_RegisteredCurrencies_@Module_specifications)
    -  [Function `initialize`](#0x1_RegisteredCurrencies_Specification_initialize)
        -  [Initialization](#0x1_RegisteredCurrencies_@Initialization)
        -  [Uniqueness of the RegisteredCurrencies config.](#0x1_RegisteredCurrencies_@Uniqueness_of_the_RegisteredCurrencies_config.)
        -  [Currency Codes](#0x1_RegisteredCurrencies_@Currency_Codes)



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

## Resource `RegistrationCapability`



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



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer, create_config_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="LibraConfig.md#0x1_LibraConfig_CreateOnChainConfig">LibraConfig::CreateOnChainConfig</a>&gt;): <a href="#0x1_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_initialize">initialize</a>(
    config_account: &signer,
    create_config_capability: &Capability&lt;CreateOnChainConfig&gt;,
): <a href="#0x1_RegisteredCurrencies_RegistrationCapability">RegistrationCapability</a> {
    // enforce that this is only going <b>to</b> one specific address,
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(config_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>(),
        0
    );
    <b>let</b> cap = <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config_with_capability">LibraConfig::publish_new_config_with_capability</a>(
        config_account,
        create_config_capability,
        <a href="#0x1_RegisteredCurrencies_empty">empty</a>()
    );

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

<a name="0x1_RegisteredCurrencies_Specification"></a>

## Specification


<a name="0x1_RegisteredCurrencies_@Module_specifications"></a>

### Module specifications



<pre><code>pragma verify = <b>true</b>;
</code></pre>


Returns true iff initialize has been called.


<a name="0x1_RegisteredCurrencies_spec_is_initialized"></a>


<pre><code><b>define</b> <a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>(): bool {
    <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::SPEC_ASSOCIATION_ROOT_ADDRESS</a>())
}
</code></pre>



<a name="0x1_RegisteredCurrencies_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RegisteredCurrencies_initialize">initialize</a>(config_account: &signer, create_config_capability: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="LibraConfig.md#0x1_LibraConfig_CreateOnChainConfig">LibraConfig::CreateOnChainConfig</a>&gt;): <a href="#0x1_RegisteredCurrencies_RegistrationCapability">RegisteredCurrencies::RegistrationCapability</a>
</code></pre>



<a name="0x1_RegisteredCurrencies_@Initialization"></a>

#### Initialization



<pre><code>pragma aborts_if_is_partial = <b>true</b>;
</code></pre>


After
<code>initialize</code> is called, the module is initialized.
<code>initialize</code> aborts if already initialized


<pre><code><b>aborts_if</b> <a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>();
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



<a name="0x1_RegisteredCurrencies_OnlyConfigAddressHasRegisteredCurrencies"></a>

There is no address with a RegisteredCurrencies value before initialization.


<pre><code><b>schema</b> <a href="#0x1_RegisteredCurrencies_OnlyConfigAddressHasRegisteredCurrencies">OnlyConfigAddressHasRegisteredCurrencies</a> {
    <b>invariant</b> !<a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>()
        ==&gt; (forall addr: address: !<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;(addr));
}
</code></pre>


*Informally:* After initialization, only singleton_address() has a RegisteredCurrencies value.


<pre><code><b>schema</b> <a href="#0x1_RegisteredCurrencies_OnlyConfigAddressHasRegisteredCurrencies">OnlyConfigAddressHasRegisteredCurrencies</a> {
    <b>invariant</b> <a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>()
        ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::SPEC_ASSOCIATION_ROOT_ADDRESS</a>())
            && (forall addr: address:
                   <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;(addr)
                              ==&gt; addr == <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::SPEC_ASSOCIATION_ROOT_ADDRESS</a>());
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_RegisteredCurrencies_OnlyConfigAddressHasRegisteredCurrencies">OnlyConfigAddressHasRegisteredCurrencies</a> <b>to</b> *;
</code></pre>



<a name="0x1_RegisteredCurrencies_@Currency_Codes"></a>

#### Currency Codes

> TODO: currency_code vector is a set (no dups).  (Not satisfied now.)
> TODO: add_currency just pushes one thing.
Only
<code>Self::add_currency</code> changes the currency_codes vector.


<a name="0x1_RegisteredCurrencies_OnlyAddCurrencyChangesRegistration"></a>


<pre><code><b>schema</b> <a href="#0x1_RegisteredCurrencies_OnlyAddCurrencyChangesRegistration">OnlyAddCurrencyChangesRegistration</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x1_RegisteredCurrencies_spec_is_initialized">spec_is_initialized</a>())
                 ==&gt; <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_get">LibraConfig::spec_get</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;().currency_codes)
                      == <a href="LibraConfig.md#0x1_LibraConfig_spec_get">LibraConfig::spec_get</a>&lt;<a href="#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;().currency_codes;
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_RegisteredCurrencies_OnlyAddCurrencyChangesRegistration">OnlyAddCurrencyChangesRegistration</a> <b>to</b> * <b>except</b> add_currency_code;
</code></pre>
