
<a name="0x1_SystemAddresses"></a>

# Module `0x1::SystemAddresses`



-  [Constants](#@Constants_0)
-  [Function `assert_core_resource`](#0x1_SystemAddresses_assert_core_resource)
-  [Function `assert_core_resource_address`](#0x1_SystemAddresses_assert_core_resource_address)
-  [Function `is_core_resource_address`](#0x1_SystemAddresses_is_core_resource_address)
-  [Function `assert_vm`](#0x1_SystemAddresses_assert_vm)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_SystemAddresses_ENOT_CORE_RESOURCE_ADDRESS"></a>

The address/account did not correspond to the core resource address


<pre><code><b>const</b> <a href="SystemAddresses.md#0x1_SystemAddresses_ENOT_CORE_RESOURCE_ADDRESS">ENOT_CORE_RESOURCE_ADDRESS</a>: u64 = 0;
</code></pre>



<a name="0x1_SystemAddresses_EVM"></a>

The operation can only be performed by the VM


<pre><code><b>const</b> <a href="SystemAddresses.md#0x1_SystemAddresses_EVM">EVM</a>: u64 = 1;
</code></pre>



<a name="0x1_SystemAddresses_assert_core_resource"></a>

## Function `assert_core_resource`



<pre><code><b>public</b> <b>fun</b> <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">assert_core_resource</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">assert_core_resource</a>(account: &signer) {
    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource_address">assert_core_resource_address</a>(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="SystemAddresses.md#0x1_SystemAddresses_AbortsIfNotCoreResource">AbortsIfNotCoreResource</a> {addr: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) };
</code></pre>



</details>

<a name="0x1_SystemAddresses_assert_core_resource_address"></a>

## Function `assert_core_resource_address`



<pre><code><b>public</b> <b>fun</b> <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource_address">assert_core_resource_address</a>(addr: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource_address">assert_core_resource_address</a>(addr: <b>address</b>) {
    <b>assert</b>!(<a href="SystemAddresses.md#0x1_SystemAddresses_is_core_resource_address">is_core_resource_address</a>(addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="SystemAddresses.md#0x1_SystemAddresses_ENOT_CORE_RESOURCE_ADDRESS">ENOT_CORE_RESOURCE_ADDRESS</a>))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="SystemAddresses.md#0x1_SystemAddresses_AbortsIfNotCoreResource">AbortsIfNotCoreResource</a>;
</code></pre>


Specifies that a function aborts if the account does not have the Diem root address.


<a name="0x1_SystemAddresses_AbortsIfNotCoreResource"></a>


<pre><code><b>schema</b> <a href="SystemAddresses.md#0x1_SystemAddresses_AbortsIfNotCoreResource">AbortsIfNotCoreResource</a> {
    addr: <b>address</b>;
    <b>aborts_if</b> addr != @CoreResources <b>with</b> Errors::REQUIRES_ADDRESS;
}
</code></pre>



</details>

<a name="0x1_SystemAddresses_is_core_resource_address"></a>

## Function `is_core_resource_address`



<pre><code><b>public</b> <b>fun</b> <a href="SystemAddresses.md#0x1_SystemAddresses_is_core_resource_address">is_core_resource_address</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SystemAddresses.md#0x1_SystemAddresses_is_core_resource_address">is_core_resource_address</a>(addr: <b>address</b>): bool {
    addr == @CoreResources
}
</code></pre>



</details>

<a name="0x1_SystemAddresses_assert_vm"></a>

## Function `assert_vm`

Assert that the signer has the VM reserved address.


<pre><code><b>public</b> <b>fun</b> <a href="SystemAddresses.md#0x1_SystemAddresses_assert_vm">assert_vm</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SystemAddresses.md#0x1_SystemAddresses_assert_vm">assert_vm</a>(account: &signer) {
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == @VMReserved, <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="SystemAddresses.md#0x1_SystemAddresses_EVM">EVM</a>))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>include</b> <a href="SystemAddresses.md#0x1_SystemAddresses_AbortsIfNotVM">AbortsIfNotVM</a>;
</code></pre>


Specifies that a function aborts if the account does not have the VM reserved address.


<a name="0x1_SystemAddresses_AbortsIfNotVM"></a>


<pre><code><b>schema</b> <a href="SystemAddresses.md#0x1_SystemAddresses_AbortsIfNotVM">AbortsIfNotVM</a> {
    account: signer;
    <b>aborts_if</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != @VMReserved <b>with</b> Errors::REQUIRES_ADDRESS;
}
</code></pre>



</details>
