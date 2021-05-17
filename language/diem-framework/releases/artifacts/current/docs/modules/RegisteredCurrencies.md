
<a name="0x1_RegisteredCurrencies"></a>

# Module `0x1::RegisteredCurrencies`

Module for registering currencies in Diem. Basically, this means adding a
string (vector<u8>) for the currency name to vector of names in DiemConfig.


-  [Struct `RegisteredCurrencies`](#0x1_RegisteredCurrencies_RegisteredCurrencies)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_RegisteredCurrencies_initialize)
-  [Function `add_currency_code`](#0x1_RegisteredCurrencies_add_currency_code)
-  [Module Specification](#@Module_Specification_1)
    -  [Initialization](#@Initialization_2)
    -  [Helper Functions](#@Helper_Functions_3)


<pre><code><b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_RegisteredCurrencies_RegisteredCurrencies"></a>

## Struct `RegisteredCurrencies`

A DiemConfig config holding all of the currency codes for registered
currencies. The inner vector<u8>'s are string representations of
currency names.


<pre><code><b>struct</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a> has <b>copy</b>, drop, store
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_RegisteredCurrencies_ECURRENCY_CODE_ALREADY_TAKEN"></a>

Attempted to add a currency code that is already in use


<pre><code><b>const</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_ECURRENCY_CODE_ALREADY_TAKEN">ECURRENCY_CODE_ALREADY_TAKEN</a>: u64 = 0;
</code></pre>



<a name="0x1_RegisteredCurrencies_initialize"></a>

## Function `initialize`

Initializes this module. Can only be called from genesis, with
a Diem root signer.


<pre><code><b>public</b> <b>fun</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_initialize">initialize</a>(dr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_initialize">initialize</a>(dr_account: &signer) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);
    <a href="DiemConfig.md#0x1_DiemConfig_publish_new_config">DiemConfig::publish_new_config</a>(
        dr_account,
        <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a> { currency_codes: <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>() }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_InitializeAbortsIf">InitializeAbortsIf</a>;
<b>include</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_InitializeEnsures">InitializeEnsures</a>;
</code></pre>




<a name="0x1_RegisteredCurrencies_InitializeAbortsIf"></a>


<pre><code><b>schema</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_InitializeAbortsIf">InitializeAbortsIf</a> {
    dr_account: signer;
    <b>include</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_AbortsIfNotGenesis">DiemTimestamp::AbortsIfNotGenesis</a>;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
    <b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_PublishNewConfigAbortsIf">DiemConfig::PublishNewConfigAbortsIf</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;;
}
</code></pre>




<a name="0x1_RegisteredCurrencies_InitializeEnsures"></a>


<pre><code><b>schema</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_InitializeEnsures">InitializeEnsures</a> {
    <b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_PublishNewConfigEnsures">DiemConfig::PublishNewConfigEnsures</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;{
        payload: <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a> { currency_codes: <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>() }
    };
    <b>ensures</b> len(<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>()) == 0;
}
</code></pre>



</details>

<a name="0x1_RegisteredCurrencies_add_currency_code"></a>

## Function `add_currency_code`

Adds a new currency code. The currency code must not yet exist.


<pre><code><b>public</b> <b>fun</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_add_currency_code">add_currency_code</a>(dr_account: &signer, currency_code: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_add_currency_code">add_currency_code</a>(
    dr_account: &signer,
    currency_code: vector&lt;u8&gt;,
) {
    <b>let</b> config = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;();
    <b>assert</b>(
        !<a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_contains">Vector::contains</a>(&config.currency_codes, &currency_code),
        <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_ECURRENCY_CODE_ALREADY_TAKEN">ECURRENCY_CODE_ALREADY_TAKEN</a>)
    );
    <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> config.currency_codes, currency_code);
    <a href="DiemConfig.md#0x1_DiemConfig_set">DiemConfig::set</a>(dr_account, config);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf">AddCurrencyCodeAbortsIf</a>;
<b>include</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_AddCurrencyCodeEnsures">AddCurrencyCodeEnsures</a>;
</code></pre>




<a name="0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf"></a>


<pre><code><b>schema</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf">AddCurrencyCodeAbortsIf</a> {
    dr_account: &signer;
    currency_code: vector&lt;u8&gt;;
    <b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_SetAbortsIf">DiemConfig::SetAbortsIf</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;{ account: dr_account };
}
</code></pre>


The same currency code can be only added once.


<pre><code><b>schema</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf">AddCurrencyCodeAbortsIf</a> {
    <b>aborts_if</b> contains(
        <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;().currency_codes,
        currency_code
    ) <b>with</b> <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>




<a name="0x1_RegisteredCurrencies_AddCurrencyCodeEnsures"></a>


<pre><code><b>schema</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_AddCurrencyCodeEnsures">AddCurrencyCodeEnsures</a> {
    currency_code: vector&lt;u8&gt;;
    <b>ensures</b> <a href="../../../../../../move-stdlib/docs/Vector.md#0x1_Vector_eq_push_back">Vector::eq_push_back</a>(<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>(), <b>old</b>(<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>()), currency_code);
    <b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_SetEnsures">DiemConfig::SetEnsures</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt; {payload: <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;()};
}
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



<a name="@Initialization_2"></a>

### Initialization


Global invariant that currency config is always available after genesis.


<pre><code><b>invariant</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>() ==&gt; <a href="DiemConfig.md#0x1_DiemConfig_spec_is_published">DiemConfig::spec_is_published</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;();
</code></pre>



<a name="@Helper_Functions_3"></a>

### Helper Functions


Helper to get the currency code vector.


<a name="0x1_RegisteredCurrencies_get_currency_codes"></a>


<pre><code><b>fun</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>(): vector&lt;vector&lt;u8&gt;&gt; {
    <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;().currency_codes
}
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
