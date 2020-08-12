
<a name="0x1_CoreAddresses"></a>

# Module `0x1::CoreAddresses`

### Table of Contents

-  [Const `ELIBRA_ROOT`](#0x1_CoreAddresses_ELIBRA_ROOT)
-  [Const `ETREASURY_COMPLIANCE`](#0x1_CoreAddresses_ETREASURY_COMPLIANCE)
-  [Const `EVM`](#0x1_CoreAddresses_EVM)
-  [Const `ELIBRA_ROOT_OR_TREASURY_COMPLIANCE`](#0x1_CoreAddresses_ELIBRA_ROOT_OR_TREASURY_COMPLIANCE)
-  [Const `ECURRENCY_INFO`](#0x1_CoreAddresses_ECURRENCY_INFO)
-  [Function `LIBRA_ROOT_ADDRESS`](#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS)
-  [Function `CURRENCY_INFO_ADDRESS`](#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS)
-  [Function `TREASURY_COMPLIANCE_ADDRESS`](#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS)
-  [Function `VM_RESERVED_ADDRESS`](#0x1_CoreAddresses_VM_RESERVED_ADDRESS)
-  [Function `assert_libra_root`](#0x1_CoreAddresses_assert_libra_root)
-  [Function `assert_treasury_compliance`](#0x1_CoreAddresses_assert_treasury_compliance)
-  [Function `assert_vm`](#0x1_CoreAddresses_assert_vm)
-  [Function `assert_currency_info`](#0x1_CoreAddresses_assert_currency_info)
-  [Function `assert_libra_root_or_treasury_compliance`](#0x1_CoreAddresses_assert_libra_root_or_treasury_compliance)
-  [Specification](#0x1_CoreAddresses_Specification)
    -  [Function `assert_libra_root`](#0x1_CoreAddresses_Specification_assert_libra_root)
    -  [Function `assert_treasury_compliance`](#0x1_CoreAddresses_Specification_assert_treasury_compliance)
    -  [Function `assert_vm`](#0x1_CoreAddresses_Specification_assert_vm)
    -  [Function `assert_currency_info`](#0x1_CoreAddresses_Specification_assert_currency_info)
    -  [Function `assert_libra_root_or_treasury_compliance`](#0x1_CoreAddresses_Specification_assert_libra_root_or_treasury_compliance)



<a name="0x1_CoreAddresses_ELIBRA_ROOT"></a>

## Const `ELIBRA_ROOT`



<pre><code><b>const</b> ELIBRA_ROOT: u64 = 0;
</code></pre>



<a name="0x1_CoreAddresses_ETREASURY_COMPLIANCE"></a>

## Const `ETREASURY_COMPLIANCE`



<pre><code><b>const</b> ETREASURY_COMPLIANCE: u64 = 1;
</code></pre>



<a name="0x1_CoreAddresses_EVM"></a>

## Const `EVM`



<pre><code><b>const</b> EVM: u64 = 2;
</code></pre>



<a name="0x1_CoreAddresses_ELIBRA_ROOT_OR_TREASURY_COMPLIANCE"></a>

## Const `ELIBRA_ROOT_OR_TREASURY_COMPLIANCE`



<pre><code><b>const</b> ELIBRA_ROOT_OR_TREASURY_COMPLIANCE: u64 = 3;
</code></pre>



<a name="0x1_CoreAddresses_ECURRENCY_INFO"></a>

## Const `ECURRENCY_INFO`



<pre><code><b>const</b> ECURRENCY_INFO: u64 = 4;
</code></pre>



<a name="0x1_CoreAddresses_LIBRA_ROOT_ADDRESS"></a>

## Function `LIBRA_ROOT_ADDRESS`

The address of the libra root account. This account is
created in genesis, and cannot be changed. This address has
ultimate authority over the permissions granted (or removed) from
accounts on-chain.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_CURRENCY_INFO_ADDRESS"></a>

## Function `CURRENCY_INFO_ADDRESS`

The (singleton) address under which the
<code><a href="Libra.md#0x1_Libra_CurrencyInfo">0x1::Libra::CurrencyInfo</a></code> resource for
every registered currency is published. This is the same as the
<code>LIBRA_ROOT_ADDRESS</code> but there is no requirement that it must
be this from an operational viewpoint, so this is why this is separated out.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CURRENCY_INFO_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CURRENCY_INFO_ADDRESS</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS"></a>

## Function `TREASURY_COMPLIANCE_ADDRESS`

The account address of the treasury and compliance account in
charge of minting/burning and other day-to-day but privileged
operations. The account at this address is created in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>(): address {
    0xB1E55ED
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_VM_RESERVED_ADDRESS"></a>

## Function `VM_RESERVED_ADDRESS`

The reserved address for transactions inserted by the VM into blocks (e.g.
block metadata transactions). Because the transaction is sent from
the VM, an account _cannot_ exist at the
<code>0x0</code> address since there
is no signer for the transaction.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(): address {
    0x0
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_assert_libra_root"></a>

## Function `assert_libra_root`

Assert that the account is the libra root address.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_libra_root">assert_libra_root</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_libra_root">assert_libra_root</a>(account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(ELIBRA_ROOT))
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_assert_treasury_compliance"></a>

## Function `assert_treasury_compliance`

Assert that the signer has the treasury compliance address.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_treasury_compliance">assert_treasury_compliance</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_treasury_compliance">assert_treasury_compliance</a>(account: &signer) {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>(),
        <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(ETREASURY_COMPLIANCE)
    )
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_assert_vm"></a>

## Function `assert_vm`

Assert that the signer has the VM reserved address.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_vm">assert_vm</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_vm">assert_vm</a>(account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(EVM))
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_assert_currency_info"></a>

## Function `assert_currency_info`

Assert that the signer has the currency info address.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_currency_info">assert_currency_info</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_currency_info">assert_currency_info</a>(account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CURRENCY_INFO_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(ECURRENCY_INFO))
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_assert_libra_root_or_treasury_compliance"></a>

## Function `assert_libra_root_or_treasury_compliance`

Assert that the signer has the libra root or treasury compliance address.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_libra_root_or_treasury_compliance">assert_libra_root_or_treasury_compliance</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_libra_root_or_treasury_compliance">assert_libra_root_or_treasury_compliance</a>(account: &signer) {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(
        addr == <a href="#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>() || addr == <a href="#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>(),
        <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(ELIBRA_ROOT_OR_TREASURY_COMPLIANCE)
    )
}
</code></pre>



</details>

<a name="0x1_CoreAddresses_Specification"></a>

## Specification


<a name="0x1_CoreAddresses_Specification_assert_libra_root"></a>

### Function `assert_libra_root`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_libra_root">assert_libra_root</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_CoreAddresses_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a>;
</code></pre>


Specifies that a function aborts if the account has not the Libra root address.


<a name="0x1_CoreAddresses_AbortsIfNotLibraRoot"></a>


<pre><code><b>schema</b> <a href="#0x1_CoreAddresses_AbortsIfNotLibraRoot">AbortsIfNotLibraRoot</a> {
    account: signer;
    <b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>()
        with Errors::REQUIRES_ADDRESS;
}
</code></pre>



<a name="0x1_CoreAddresses_Specification_assert_treasury_compliance"></a>

### Function `assert_treasury_compliance`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_treasury_compliance">assert_treasury_compliance</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_CoreAddresses_AbortsIfNotTreasuryCompliance">AbortsIfNotTreasuryCompliance</a>;
</code></pre>


Specifies that a function aborts if the account has not the treasury compliance address.


<a name="0x1_CoreAddresses_AbortsIfNotTreasuryCompliance"></a>


<pre><code><b>schema</b> <a href="#0x1_CoreAddresses_AbortsIfNotTreasuryCompliance">AbortsIfNotTreasuryCompliance</a> {
    account: signer;
    <b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>()
        with Errors::REQUIRES_ADDRESS;
}
</code></pre>



<a name="0x1_CoreAddresses_Specification_assert_vm"></a>

### Function `assert_vm`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_vm">assert_vm</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_CoreAddresses_AbortsIfNotVM">AbortsIfNotVM</a>;
</code></pre>


Specifies that a function aborts if the account has not the VM reserved address.


<a name="0x1_CoreAddresses_AbortsIfNotVM"></a>


<pre><code><b>schema</b> <a href="#0x1_CoreAddresses_AbortsIfNotVM">AbortsIfNotVM</a> {
    account: signer;
    <b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="#0x1_CoreAddresses_VM_RESERVED_ADDRESS">VM_RESERVED_ADDRESS</a>()
        with Errors::REQUIRES_ADDRESS;
}
</code></pre>



<a name="0x1_CoreAddresses_Specification_assert_currency_info"></a>

### Function `assert_currency_info`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_currency_info">assert_currency_info</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_CoreAddresses_AbortsIfNotCurrencyInfo">AbortsIfNotCurrencyInfo</a>;
</code></pre>


Specifies that a function aborts if the account has not the currency info address.


<a name="0x1_CoreAddresses_AbortsIfNotCurrencyInfo"></a>


<pre><code><b>schema</b> <a href="#0x1_CoreAddresses_AbortsIfNotCurrencyInfo">AbortsIfNotCurrencyInfo</a> {
    account: signer;
    <b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CURRENCY_INFO_ADDRESS</a>()
        with Errors::REQUIRES_ADDRESS;
}
</code></pre>



<a name="0x1_CoreAddresses_Specification_assert_libra_root_or_treasury_compliance"></a>

### Function `assert_libra_root_or_treasury_compliance`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreAddresses_assert_libra_root_or_treasury_compliance">assert_libra_root_or_treasury_compliance</a>(account: &signer)
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_CoreAddresses_AbortsIfNotLibraRootOrTreasuryCompliance">AbortsIfNotLibraRootOrTreasuryCompliance</a>;
</code></pre>


Specifies that a function aborts if the account has not either the libra root or the treasury compliance
address.


<a name="0x1_CoreAddresses_AbortsIfNotLibraRootOrTreasuryCompliance"></a>


<pre><code><b>schema</b> <a href="#0x1_CoreAddresses_AbortsIfNotLibraRootOrTreasuryCompliance">AbortsIfNotLibraRootOrTreasuryCompliance</a> {
    account: signer;
    <a name="0x1_CoreAddresses_addr$9"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>aborts_if</b> addr != <a href="#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">LIBRA_ROOT_ADDRESS</a>() && addr != <a href="#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">TREASURY_COMPLIANCE_ADDRESS</a>()
        with Errors::REQUIRES_ADDRESS;
}
</code></pre>
