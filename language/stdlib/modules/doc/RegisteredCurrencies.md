
<a name="0x1_RegisteredCurrencies"></a>

# Module `0x1::RegisteredCurrencies`

Module managing the registered currencies in the Libra framework.


-  [Struct `RegisteredCurrencies`](#0x1_RegisteredCurrencies_RegisteredCurrencies)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_RegisteredCurrencies_initialize)
-  [Function `add_currency_code`](#0x1_RegisteredCurrencies_add_currency_code)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="LibraConfig.md#0x1_LibraConfig">0x1::LibraConfig</a>;
<b>use</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp">0x1::LibraTimestamp</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_RegisteredCurrencies_RegisteredCurrencies"></a>

## Struct `RegisteredCurrencies`

An on-chain config holding all of the currency codes for registered
currencies. The inner vector<u8>'s are string representations of
currency names.


<pre><code><b>struct</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>
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

Initializes this module. Can only be called from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_initialize">initialize</a>(lr_account: &signer) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>(
        lr_account,
        <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a> { currency_codes: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>() }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigAbortsIf">LibraConfig::PublishNewConfigAbortsIf</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigEnsures">LibraConfig::PublishNewConfigEnsures</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;{
    payload: <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a> { currency_codes: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>() }
};
<b>ensures</b> len(<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>()) == 0;
</code></pre>



</details>

<a name="0x1_RegisteredCurrencies_add_currency_code"></a>

## Function `add_currency_code`

Adds a new currency code. The currency code must not yet exist.


<pre><code><b>public</b> <b>fun</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_add_currency_code">add_currency_code</a>(lr_account: &signer, currency_code: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_add_currency_code">add_currency_code</a>(
    lr_account: &signer,
    currency_code: vector&lt;u8&gt;,
) {
    <b>let</b> config = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;();
    <b>assert</b>(
        !<a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&config.currency_codes, &currency_code),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_ECURRENCY_CODE_ALREADY_TAKEN">ECURRENCY_CODE_ALREADY_TAKEN</a>)
    );
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> config.currency_codes, currency_code);
    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>(lr_account, config);
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
    lr_account: &signer;
    currency_code: vector&lt;u8&gt;;
    <b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetAbortsIf">LibraConfig::SetAbortsIf</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;{ account: lr_account };
}
</code></pre>


The same currency code can be only added once.


<pre><code><b>schema</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf">AddCurrencyCodeAbortsIf</a> {
    <b>aborts_if</b> <a href="Vector.md#0x1_Vector_spec_contains">Vector::spec_contains</a>(
        <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;().currency_codes,
        currency_code
    ) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>




<a name="0x1_RegisteredCurrencies_AddCurrencyCodeEnsures"></a>


<pre><code><b>schema</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_AddCurrencyCodeEnsures">AddCurrencyCodeEnsures</a> {
    currency_code: vector&lt;u8&gt;;
}
</code></pre>


The resulting currency_codes is the one before this function is called, with the new one added to the end.


<pre><code><b>schema</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_AddCurrencyCodeEnsures">AddCurrencyCodeEnsures</a> {
    <b>ensures</b> <a href="Vector.md#0x1_Vector_eq_push_back">Vector::eq_push_back</a>(<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>(), <b>old</b>(<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>()), currency_code);
    <b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetEnsures">LibraConfig::SetEnsures</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt; {payload: <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;()};
}
</code></pre>



Helper to get the currency code vector.


<a name="0x1_RegisteredCurrencies_get_currency_codes"></a>


<pre><code><b>define</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_get_currency_codes">get_currency_codes</a>(): vector&lt;vector&lt;u8&gt;&gt; {
    <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;().currency_codes
}
</code></pre>


Global invariant that currency config is always available after genesis.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies">RegisteredCurrencies</a>&gt;();
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
