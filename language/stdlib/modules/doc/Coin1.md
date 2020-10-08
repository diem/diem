
<a name="0x1_Coin1"></a>

# Module `0x1::Coin1`

This module defines the coin type Coin1 and its initialization function.


-  [Struct `Coin1`](#0x1_Coin1_Coin1)
-  [Function `initialize`](#0x1_Coin1_initialize)
-  [Module Specification](#@Module_Specification_0)
    -  [Persistence of Resources](#@Persistence_of_Resources_1)


<pre><code><b>use</b> <a href="AccountLimits.md#0x1_AccountLimits">0x1::AccountLimits</a>;
<b>use</b> <a href="FixedPoint32.md#0x1_FixedPoint32">0x1::FixedPoint32</a>;
<b>use</b> <a href="Libra.md#0x1_Libra">0x1::Libra</a>;
<b>use</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp">0x1::LibraTimestamp</a>;
</code></pre>



<a name="0x1_Coin1_Coin1"></a>

## Struct `Coin1`

The type tag representing the <code><a href="Coin1.md#0x1_Coin1">Coin1</a></code> currency on-chain.


<pre><code><b>struct</b> <a href="Coin1.md#0x1_Coin1">Coin1</a>
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

Registers the <code><a href="Coin1.md#0x1_Coin1">Coin1</a></code> cointype. This can only be called from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="Coin1.md#0x1_Coin1_initialize">initialize</a>(lr_account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Coin1.md#0x1_Coin1_initialize">initialize</a>(
    lr_account: &signer,
    tc_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Libra.md#0x1_Libra_register_SCS_currency">Libra::register_SCS_currency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(
        lr_account,
        tc_account,
        <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 1), // exchange rate <b>to</b> <a href="LBR.md#0x1_LBR">LBR</a>
        1000000, // scaling_factor = 10^6
        100,     // fractional_part = 10^2
        b"<a href="Coin1.md#0x1_Coin1">Coin1</a>"
    );
    <a href="AccountLimits.md#0x1_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(lr_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="Libra.md#0x1_Libra_RegisterSCSCurrencyAbortsIf">Libra::RegisterSCSCurrencyAbortsIf</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;{
    currency_code: b"<a href="Coin1.md#0x1_Coin1">Coin1</a>",
    scaling_factor: 1000000
};
<b>include</b> <a href="AccountLimits.md#0x1_AccountLimits_PublishUnrestrictedLimitsAbortsIf">AccountLimits::PublishUnrestrictedLimitsAbortsIf</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;{publish_account: lr_account};
<b>include</b> <a href="Libra.md#0x1_Libra_RegisterSCSCurrencyEnsures">Libra::RegisterSCSCurrencyEnsures</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;;
<b>include</b> <a href="AccountLimits.md#0x1_AccountLimits_PublishUnrestrictedLimitsEnsures">AccountLimits::PublishUnrestrictedLimitsEnsures</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;{publish_account: lr_account};
</code></pre>


Registering Coin1 can only be done in genesis.


<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
</code></pre>


Only the LibraRoot account can register a new currency [[H8]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
</code></pre>


Only a TreasuryCompliance account can have the MintCapability [[H1]][PERMISSION].
Moreover, only a TreasuryCompliance account can have the BurnCapability [[H3]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: tc_account};
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



<a name="@Persistence_of_Resources_1"></a>

### Persistence of Resources


After genesis, Coin1 is registered.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="Libra.md#0x1_Libra_is_currency">Libra::is_currency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;();
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/libra/lip/blob/master/lips/lip-2.md
[ROLE]: https://github.com/libra/lip/blob/master/lips/lip-2.md#roles
[PERMISSION]: https://github.com/libra/lip/blob/master/lips/lip-2.md#permissions
