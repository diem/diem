
<a name="0x1_Coin1"></a>

# Module `0x1::Coin1`

### Table of Contents

-  [Struct `Coin1`](#0x1_Coin1_Coin1)
-  [Function `initialize`](#0x1_Coin1_initialize)
-  [Specification](#0x1_Coin1_Specification)
    -  [Function `initialize`](#0x1_Coin1_Specification_initialize)
    -  [Module Specification](#0x1_Coin1_@Module_Specification)



<a name="0x1_Coin1_Coin1"></a>

## Struct `Coin1`



<pre><code><b>struct</b> <a href="#0x1_Coin1">Coin1</a>
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

<a name="0x1_Coin1_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Coin1_initialize">initialize</a>(lr_account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Coin1_initialize">initialize</a>(
    lr_account: &signer,
    tc_account: &signer,
) {
    // Register the <a href="#0x1_Coin1">Coin1</a> currency.
    <b>let</b> (coin1_mint_cap, coin1_burn_cap) =
        <a href="Libra.md#0x1_Libra_register_currency">Libra::register_currency</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;(
            lr_account,
            <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 2), // exchange rate <b>to</b> <a href="LBR.md#0x1_LBR">LBR</a>
            <b>false</b>,   // is_synthetic
            1000000, // scaling_factor = 10^6
            100,     // fractional_part = 10^2
            b"<a href="#0x1_Coin1">Coin1</a>"
        );
    <a href="Libra.md#0x1_Libra_publish_mint_capability">Libra::publish_mint_capability</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;(tc_account, coin1_mint_cap, tc_account);
    <a href="Libra.md#0x1_Libra_publish_burn_capability">Libra::publish_burn_capability</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;(tc_account, coin1_burn_cap, tc_account);
}
</code></pre>



</details>

<a name="0x1_Coin1_Specification"></a>

## Specification


<a name="0x1_Coin1_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Coin1_initialize">initialize</a>(lr_account: &signer, tc_account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="Libra.md#0x1_Libra_RegisterCurrencyAbortsIf">Libra::RegisterCurrencyAbortsIf</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;;
<b>include</b> <a href="RegisteredCurrencies.md#0x1_RegisteredCurrencies_AddCurrencyCodeAbortsIf">RegisteredCurrencies::AddCurrencyCodeAbortsIf</a>{currency_code: b"<a href="#0x1_Coin1">Coin1</a>"};
<b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_treasury_compliance_role">Roles::spec_has_treasury_compliance_role</a>(tc_account);
<b>aborts_if</b> <a href="Libra.md#0x1_Libra_spec_has_mint_capability">Libra::spec_has_mint_capability</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;(tc_account);
<b>aborts_if</b> <a href="Libra.md#0x1_Libra_spec_has_burn_capability">Libra::spec_has_burn_capability</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;(tc_account);
<b>ensures</b> <a href="Libra.md#0x1_Libra_spec_is_currency">Libra::spec_is_currency</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;();
<b>ensures</b> <a href="Libra.md#0x1_Libra_spec_has_mint_capability">Libra::spec_has_mint_capability</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;(tc_account);
<b>ensures</b> <a href="Libra.md#0x1_Libra_spec_has_burn_capability">Libra::spec_has_burn_capability</a>&lt;<a href="#0x1_Coin1">Coin1</a>&gt;(tc_account);
</code></pre>


**************** MODULE SPECIFICATION ****************

<a name="0x1_Coin1_@Module_Specification"></a>

### Module Specification


Verify all functions in this module.


<pre><code>pragma verify = <b>true</b>;
</code></pre>
