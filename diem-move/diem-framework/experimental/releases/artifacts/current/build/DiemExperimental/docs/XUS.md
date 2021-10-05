
<a name="0x1_XUS"></a>

# Module `0x1::XUS`

This module defines the coin type XUS and its initialization function.


-  [Struct `XUS`](#0x1_XUS_XUS)
-  [Function `initialize`](#0x1_XUS_initialize)
-  [Module Specification](#@Module_Specification_0)
    -  [Persistence of Resources](#@Persistence_of_Resources_1)
    -  [Access Control](#@Access_Control_2)
        -  [Minting](#@Minting_3)
        -  [Burning](#@Burning_4)
        -  [Preburning](#@Preburning_5)


<pre><code><b>use</b> <a href="AccountLimits.md#0x1_AccountLimits">0x1::AccountLimits</a>;
<b>use</b> <a href="Diem.md#0x1_Diem">0x1::Diem</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32">0x1::FixedPoint32</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
</code></pre>



<a name="0x1_XUS_XUS"></a>

## Struct `XUS`

The type tag representing the <code><a href="XUS.md#0x1_XUS">XUS</a></code> currency on-chain.


<pre><code><b>struct</b> <a href="XUS.md#0x1_XUS">XUS</a>
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

<a name="0x1_XUS_initialize"></a>

## Function `initialize`

Registers the <code><a href="XUS.md#0x1_XUS">XUS</a></code> cointype. This can only be called from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="XUS.md#0x1_XUS_initialize">initialize</a>(dr_account: &signer, tc_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="XUS.md#0x1_XUS_initialize">initialize</a>(
    dr_account: &signer,
    tc_account: &signer,
) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_treasury_compliance">Roles::assert_treasury_compliance</a>(tc_account);
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);
    <a href="Diem.md#0x1_Diem_register_SCS_currency">Diem::register_SCS_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(
        dr_account,
        tc_account,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/FixedPoint32.md#0x1_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(1, 1), // exchange rate <b>to</b> <a href="XDX.md#0x1_XDX">XDX</a>
        1000000, // scaling_factor = 10^6
        100,     // fractional_part = 10^2
        b"<a href="XUS.md#0x1_XUS">XUS</a>"
    );
    <a href="AccountLimits.md#0x1_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(dr_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="Diem.md#0x1_Diem_RegisterSCSCurrencyAbortsIf">Diem::RegisterSCSCurrencyAbortsIf</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;{
    currency_code: b"<a href="XUS.md#0x1_XUS">XUS</a>",
    scaling_factor: 1000000
};
<b>include</b> <a href="AccountLimits.md#0x1_AccountLimits_PublishUnrestrictedLimitsAbortsIf">AccountLimits::PublishUnrestrictedLimitsAbortsIf</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;{publish_account: dr_account};
<b>include</b> <a href="Diem.md#0x1_Diem_RegisterSCSCurrencyEnsures">Diem::RegisterSCSCurrencyEnsures</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;;
<b>include</b> <a href="AccountLimits.md#0x1_AccountLimits_PublishUnrestrictedLimitsEnsures">AccountLimits::PublishUnrestrictedLimitsEnsures</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;{publish_account: dr_account};
</code></pre>


Registering XUS can only be done in genesis.


<pre><code><b>include</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_AbortsIfNotGenesis">DiemTimestamp::AbortsIfNotGenesis</a>;
</code></pre>


Only the DiemRoot account can register a new currency [[H8]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
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


After genesis, XUS is registered.


<pre><code><b>invariant</b> [suspendable] <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>() ==&gt; <a href="Diem.md#0x1_Diem_is_currency">Diem::is_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;();
</code></pre>


After genesis, <code>LimitsDefinition&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;</code> is published at Diem root. It's published by
AccountLimits::publish_unrestricted_limits, but we can't prove the condition there because
it does not hold for all types (but does hold for XUS).


<pre><code><b>invariant</b> [suspendable] <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>()
    ==&gt; <b>exists</b>&lt;<a href="AccountLimits.md#0x1_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;&gt;(@DiemRoot);
</code></pre>


<code>LimitsDefinition&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;</code> is not published at any other address


<pre><code><b>invariant</b> [suspendable] <b>forall</b> addr: address <b>where</b> <b>exists</b>&lt;<a href="AccountLimits.md#0x1_AccountLimits_LimitsDefinition">AccountLimits::LimitsDefinition</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;&gt;(addr):
    addr == @DiemRoot;
</code></pre>


After genesis, XUS is always a non-synthetic currency.


<pre><code><b>invariant</b> [suspendable] <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>()
    ==&gt; !<a href="Diem.md#0x1_Diem_is_synthetic_currency">Diem::is_synthetic_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;();
</code></pre>


After genesis, the scaling factor for XUS is always equal to 1,000,000.


<pre><code><b>invariant</b> [suspendable] <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>()
    ==&gt; <a href="Diem.md#0x1_Diem_spec_currency_info">Diem::spec_currency_info</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;().scaling_factor == 1000000;
</code></pre>


After genesis, the fractional part for XUS is always equal to 100.


<pre><code><b>invariant</b> [suspendable] <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>()
    ==&gt; <a href="Diem.md#0x1_Diem_spec_currency_info">Diem::spec_currency_info</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;().fractional_part == 100;
</code></pre>


After genesis, the currency code for XUS is always "XUS".


<pre><code><b>invariant</b> [suspendable] <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>()
    ==&gt; <a href="Diem.md#0x1_Diem_spec_currency_code">Diem::spec_currency_code</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;() == b"<a href="XUS.md#0x1_XUS">XUS</a>";
</code></pre>



<a name="@Access_Control_2"></a>

### Access Control


<a name="@Minting_3"></a>

#### Minting


Only TreasuryCompliance can have MintCapability<XUS> [[H1]][PERMISSION].
If an account has MintCapability<XUS>, it is a TreasuryCompliance account.


<pre><code><b>invariant</b>
    <b>forall</b> a: address:
        <a href="Diem.md#0x1_Diem_spec_has_mint_capability">Diem::spec_has_mint_capability</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(a) ==&gt;
            <a href="Roles.md#0x1_Roles_spec_has_treasury_compliance_role_addr">Roles::spec_has_treasury_compliance_role_addr</a>(a);
</code></pre>


Only the owner of MintCapability<XUS> can mint XUS [[H1]][PERMISSION].
If the <code>total_value</code> for XUS is increased, the transaction should be
signed by the owner of MintCapability<XUS>.


<pre><code><b>invariant</b> <b>update</b> [suspendable] (
        <b>old</b>(<a href="Diem.md#0x1_Diem_spec_is_currency">Diem::spec_is_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;()) &&
        <a href="Diem.md#0x1_Diem_spec_is_currency">Diem::spec_is_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;() &&
        <b>old</b>(<a href="Diem.md#0x1_Diem_spec_currency_info">Diem::spec_currency_info</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;().total_value) &lt; <a href="Diem.md#0x1_Diem_spec_currency_info">Diem::spec_currency_info</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;().total_value
    ) ==&gt; <a href="Diem.md#0x1_Diem_spec_signed_by_mint_capability_owner">Diem::spec_signed_by_mint_capability_owner</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;();
</code></pre>


The permission to mint XUS is unique [[I1]][PERMISSION].
At most one address has a MintCapability<XUS>.


<pre><code><b>invariant</b>
    <b>forall</b> a1: address, a2: address:
        (<a href="Diem.md#0x1_Diem_spec_has_mint_capability">Diem::spec_has_mint_capability</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(a1) && <a href="Diem.md#0x1_Diem_spec_has_mint_capability">Diem::spec_has_mint_capability</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(a2)) ==&gt; a1 == a2;
</code></pre>


MintCapability<XUS> is not transferrable [[J1]][PERMISSION].
MintCapability<XUS> is not copiable, and once it's published, it's not removed.


<pre><code><b>invariant</b> <b>update</b>
    <b>forall</b> a: address
        <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="Diem.md#0x1_Diem_MintCapability">Diem::MintCapability</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;&gt;(a)):
            <b>exists</b>&lt;<a href="Diem.md#0x1_Diem_MintCapability">Diem::MintCapability</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;&gt;(a);
</code></pre>



<a name="@Burning_4"></a>

#### Burning


Only TreasuryCompliance can have BurnCapability [[H3]][PERMISSION].
If an account has BurnCapability<XUS>, it is a TreasuryCompliance account.


<pre><code><b>invariant</b>
    <b>forall</b> a: address:
        <a href="Diem.md#0x1_Diem_spec_has_burn_capability">Diem::spec_has_burn_capability</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(a) ==&gt;
            <a href="Roles.md#0x1_Roles_spec_has_treasury_compliance_role_addr">Roles::spec_has_treasury_compliance_role_addr</a>(a);
</code></pre>


Only the owner of BurnCapability<XUS> can burn XUS [[H3]][PERMISSION].
If the <code>total_value</code> or <code>preburn_value</code> for XUS is decreased, the
transaction should be signed by the owner of BurnCapability<XUS>.


<pre><code><b>invariant</b> <b>update</b> [suspendable] (
        <b>old</b>(<a href="Diem.md#0x1_Diem_spec_is_currency">Diem::spec_is_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;()) &&
        <a href="Diem.md#0x1_Diem_spec_is_currency">Diem::spec_is_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;() &&
        <b>old</b>(<a href="Diem.md#0x1_Diem_spec_currency_info">Diem::spec_currency_info</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;().total_value) &gt; <a href="Diem.md#0x1_Diem_spec_currency_info">Diem::spec_currency_info</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;().total_value
    ) ==&gt; <a href="Diem.md#0x1_Diem_spec_signed_by_burn_capability_owner">Diem::spec_signed_by_burn_capability_owner</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;();
<b>invariant</b> <b>update</b> [suspendable] (
        <b>old</b>(<a href="Diem.md#0x1_Diem_spec_is_currency">Diem::spec_is_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;()) &&
        <a href="Diem.md#0x1_Diem_spec_is_currency">Diem::spec_is_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;() &&
        <b>old</b>(<a href="Diem.md#0x1_Diem_spec_currency_info">Diem::spec_currency_info</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;().preburn_value) &gt; <a href="Diem.md#0x1_Diem_spec_currency_info">Diem::spec_currency_info</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;().preburn_value
    ) ==&gt; <a href="Diem.md#0x1_Diem_spec_signed_by_burn_capability_owner">Diem::spec_signed_by_burn_capability_owner</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;();
</code></pre>


The permission to burn XUS is unique [[I3]][PERMISSION].
At most one address has a BurnCapability<XUS>.


<pre><code><b>invariant</b>
    <b>forall</b> a1: address, a2: address:
        (<a href="Diem.md#0x1_Diem_spec_has_burn_capability">Diem::spec_has_burn_capability</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(a1) && <a href="Diem.md#0x1_Diem_spec_has_burn_capability">Diem::spec_has_burn_capability</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(a2)) ==&gt; a1 == a2;
</code></pre>


BurnCapability<XUS> is not transferrable [[J3]][PERMISSION].
BurnCapability<XUS> is not copiable, and once it's published, it's not removed.


<pre><code><b>invariant</b> <b>update</b> [suspendable]
    <b>forall</b> a: address
        <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="Diem.md#0x1_Diem_BurnCapability">Diem::BurnCapability</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;&gt;(a)):
            <b>exists</b>&lt;<a href="Diem.md#0x1_Diem_BurnCapability">Diem::BurnCapability</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;&gt;(a);
</code></pre>



<a name="@Preburning_5"></a>

#### Preburning


Only DesignatedDealer can has the "preburn" permission [[H4]][PERMISSION].
If an account has PreburnQueue<XUS> or Preburn<XUS>, it is a DesignatedDealer account.


<pre><code><b>invariant</b>
    <b>forall</b> a: address:
        (<a href="Diem.md#0x1_Diem_spec_has_preburn_queue">Diem::spec_has_preburn_queue</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(a) || <a href="Diem.md#0x1_Diem_spec_has_preburn">Diem::spec_has_preburn</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;(a)) ==&gt;
            <a href="Roles.md#0x1_Roles_spec_has_designated_dealer_role_addr">Roles::spec_has_designated_dealer_role_addr</a>(a);
</code></pre>


Only the owner of PreburnQueue<XUS> can preburn XUS [[H4]][PERMISSION].
If the <code>preburn_value</code> for XUS is increased, the transaction should be
signed by the owner of PreburnQueue<XUS> or Preburn<XUS>.


<pre><code><b>invariant</b> <b>update</b> [suspendable] (
        <b>old</b>(<a href="Diem.md#0x1_Diem_spec_is_currency">Diem::spec_is_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;()) &&
        <a href="Diem.md#0x1_Diem_spec_is_currency">Diem::spec_is_currency</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;() &&
        <b>old</b>(<a href="Diem.md#0x1_Diem_spec_currency_info">Diem::spec_currency_info</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;().preburn_value) &lt; <a href="Diem.md#0x1_Diem_spec_currency_info">Diem::spec_currency_info</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;().preburn_value
    ) ==&gt; (<a href="Diem.md#0x1_Diem_spec_signed_by_preburn_queue_owner">Diem::spec_signed_by_preburn_queue_owner</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;() || <a href="Diem.md#0x1_Diem_spec_signed_by_preburn_owner">Diem::spec_signed_by_preburn_owner</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;());
</code></pre>


PreburnQueue<XUS> is not transferrable [[J4]][PERMISSION].
PreburnQueue<XUS> is not copiable, and once it's published, it's not removed.


<pre><code><b>invariant</b> <b>update</b> [suspendable]
    <b>forall</b> a: address
        <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="Diem.md#0x1_Diem_PreburnQueue">Diem::PreburnQueue</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;&gt;(a)):
            <b>exists</b>&lt;<a href="Diem.md#0x1_Diem_PreburnQueue">Diem::PreburnQueue</a>&lt;<a href="XUS.md#0x1_XUS">XUS</a>&gt;&gt;(a);
</code></pre>
