
<a name="0x1_CoreAddresses"></a>

# Module `0x1::CoreAddresses`



-  [Const <code><a href="CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">ELIBRA_ROOT</a></code>](#0x1_CoreAddresses_ELIBRA_ROOT)
-  [Const <code><a href="CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">ETREASURY_COMPLIANCE</a></code>](#0x1_CoreAddresses_ETREASURY_COMPLIANCE)
-  [Const <code><a href="CoreAddresses.md#0x1_CoreAddresses_EVM">EVM</a></code>](#0x1_CoreAddresses_EVM)
-  [Const <code><a href="CoreAddresses.md#0x1_CoreAddresses_ECURRENCY_INFO">ECURRENCY_INFO</a></code>](#0x1_CoreAddresses_ECURRENCY_INFO)
-  [Function <code>LIBRA_ROOT_ADDRESS</code>](#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS)
-  [Function <code>CURRENCY_INFO_ADDRESS</code>](#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS)
-  [Function <code>TREASURY_COMPLIANCE_ADDRESS</code>](#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS)
-  [Function <code>VM_RESERVED_ADDRESS</code>](#0x1_CoreAddresses_VM_RESERVED_ADDRESS)
-  [Function <code>assert_libra_root</code>](#0x1_CoreAddresses_assert_libra_root)
-  [Function <code>assert_treasury_compliance</code>](#0x1_CoreAddresses_assert_treasury_compliance)
-  [Function <code>assert_vm</code>](#0x1_CoreAddresses_assert_vm)
-  [Function <code>assert_currency_info</code>](#0x1_CoreAddresses_assert_currency_info)


<a name="0x1_CoreAddresses_ELIBRA_ROOT"></a>

## Const `ELIBRA_ROOT`

The operation can only be performed by the account at 0xA550C18 (Libra Root)


<pre><code><b>const</b> <a href="CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">ELIBRA_ROOT</a>: u64 = 0;
</code></pre>



<a name="0x1_CoreAddresses_ETREASURY_COMPLIANCE"></a>

## Const `ETREASURY_COMPLIANCE`

The operation can only be performed by the account at 0xB1E55ED (Treasury & Compliance)


<pre><code><b>const</b> <a href="CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">ETREASURY_COMPLIANCE</a>: u64 = 1;
</code></pre>



<a name="0x1_CoreAddresses_EVM"></a>

## Const `EVM`

The operation can only be performed by the VM


<pre><code><b>const</b> <a href="CoreAddresses.md#0x1_CoreAddresses_EVM">EVM</a>: u64 = 2;
</code></pre>



<a name="0x1_CoreAddresses_ECURRENCY_INFO"></a>

## Const `ECURRENCY_INFO`

The operation can only be performed by the account where currencies are registered


<pre><code><b>const</b> <a href="CoreAddresses.md#0x1_CoreAddresses_ECURRENCY_INFO">ECURRENCY_INFO</a>: u64 = 4;
</code></pre>



<a name="0x1_CoreAddresses_LIBRA_ROOT_ADDRESS"></a>

## Function `LIBRA_ROOT_ADDRESS`

The address of the libra root account. This account is
created in genesis, and cannot be changed. This address has
ultimate authority over the permissions granted (or removed) from
accounts on-chain.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_CURRENCY_INFO_ADDRESS"></a>

## Function `CURRENCY_INFO_ADDRESS`

The (singleton) address under which the <code><a href="Libra.md#0x1_Libra_CurrencyInfo">0x1::Libra::CurrencyInfo</a></code> resource for
every registered currency is published. This is the same as the
<code>LIBRA_ROOT_ADDRESS</code> but there is no requirement that it must
be this from an operational viewpoint, so this is why this is separated out.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CURRENCY_INFO_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CURRENCY_INFO_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS"></a>

## Function `TREASURY_COMPLIANCE_ADDRESS`

The account address of the treasury and compliance account in
charge of minting/burning and other day-to-day but privileged
operations. The account at this address is created in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>(): address {
    0xB1E55ED
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_VM_RESERVED_ADDRESS"></a>

## Function `VM_RESERVED_ADDRESS`

The reserved address for transactions inserted by the VM into blocks (e.g.
block metadata transactions). Because the transaction is sent from
the VM, an account _cannot_ exist at the <code>0x0</code> address since there
is no signer for the transaction.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(): address {
    0x0
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_assert_libra_root"></a>

## Function `assert_libra_root`

Assert that the account is the libra root address.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">assert_libra_root</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">assert_libra_root</a>(account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_ELIBRA_ROOT">ELIBRA_ROOT</a>))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>;
</code></pre>


Specifies that a function aborts if the account has not the Libra root address.


<a name="0x1_CoreAddresses_AbortsIfNotLibraRoot"></a>


<pre><code><b>schema</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a> {
    account: signer;
    <b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>()
        <b>with</b> <a href="Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>;
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_assert_treasury_compliance"></a>

## Function `assert_treasury_compliance`

Assert that the signer has the treasury compliance address.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_treasury_compliance">assert_treasury_compliance</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_treasury_compliance">assert_treasury_compliance</a>(account: &signer) {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>(),
        <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_ETREASURY_COMPLIANCE">ETREASURY_COMPLIANCE</a>)
    )
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotTreasuryCompliance">AbortsIfNotTreasuryCompliance</a>;
</code></pre>


Specifies that a function aborts if the account has not the treasury compliance address.


<a name="0x1_CoreAddresses_AbortsIfNotTreasuryCompliance"></a>


<pre><code><b>schema</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotTreasuryCompliance">AbortsIfNotTreasuryCompliance</a> {
    account: signer;
    <b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>()
        <b>with</b> <a href="Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>;
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_assert_vm"></a>

## Function `assert_vm`

Assert that the signer has the VM reserved address.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_vm">assert_vm</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_vm">assert_vm</a>(account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_EVM">EVM</a>))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotVM">AbortsIfNotVM</a>;
</code></pre>


Specifies that a function aborts if the account has not the VM reserved address.


<a name="0x1_CoreAddresses_AbortsIfNotVM"></a>


<pre><code><b>schema</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotVM">AbortsIfNotVM</a> {
    account: signer;
    <b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>()
        <b>with</b> <a href="Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>;
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_assert_currency_info"></a>

## Function `assert_currency_info`

Assert that the signer has the currency info address.


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_currency_info">assert_currency_info</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreAddresses.md#0x1_CoreAddresses_assert_currency_info">assert_currency_info</a>(account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CURRENCY_INFO_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_ECURRENCY_INFO">ECURRENCY_INFO</a>))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotCurrencyInfo">AbortsIfNotCurrencyInfo</a>;
</code></pre>


Specifies that a function aborts if the account has not the currency info address.


<a name="0x1_CoreAddresses_AbortsIfNotCurrencyInfo"></a>


<pre><code><b>schema</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotCurrencyInfo">AbortsIfNotCurrencyInfo</a> {
    account: signer;
    <b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CURRENCY_INFO_ADDRESS</a>()
        <b>with</b> <a href="Errors.md#0x1_Errors_REQUIRES_ADDRESS">Errors::REQUIRES_ADDRESS</a>;
}
</code></pre>



</details>

[]: # (File containing markdown style reference definitions to be included in each generated doc)
