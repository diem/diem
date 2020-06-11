
<a name="0x0_Testnet"></a>

# Module `0x0::Testnet`

### Table of Contents

-  [Struct `IsTestnet`](#0x0_Testnet_IsTestnet)
-  [Function `initialize`](#0x0_Testnet_initialize)
-  [Function `is_testnet`](#0x0_Testnet_is_testnet)
-  [Function `remove_testnet`](#0x0_Testnet_remove_testnet)
-  [Specification](#0x0_Testnet_Specification)
    -  [Module Specification](#0x0_Testnet_@Module_Specification)
        -  [Management of TestNet](#0x0_Testnet_@Management_of_TestNet)
        -  [Specifications for individual functions](#0x0_Testnet_@Specifications_for_individual_functions)
    -  [Function `initialize`](#0x0_Testnet_Specification_initialize)
    -  [Function `is_testnet`](#0x0_Testnet_Specification_is_testnet)
    -  [Function `remove_testnet`](#0x0_Testnet_Specification_remove_testnet)



<a name="0x0_Testnet_IsTestnet"></a>

## Struct `IsTestnet`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Testnet_IsTestnet">IsTestnet</a>
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

<a name="0x0_Testnet_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Testnet_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Testnet_initialize">initialize</a>(account: &signer) {
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x0_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>(), 0);
    move_to(account, <a href="#0x0_Testnet_IsTestnet">IsTestnet</a>{})
}
</code></pre>



</details>

<a name="0x0_Testnet_is_testnet"></a>

## Function `is_testnet`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Testnet_is_testnet">is_testnet</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Testnet_is_testnet">is_testnet</a>(): bool {
    exists&lt;<a href="#0x0_Testnet_IsTestnet">IsTestnet</a>&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x0_Testnet_remove_testnet"></a>

## Function `remove_testnet`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Testnet_remove_testnet">remove_testnet</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Testnet_remove_testnet">remove_testnet</a>(account: &signer)
<b>acquires</b> <a href="#0x0_Testnet_IsTestnet">IsTestnet</a> {
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x0_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>(), 0);
    <a href="#0x0_Testnet_IsTestnet">IsTestnet</a>{} = move_from&lt;<a href="#0x0_Testnet_IsTestnet">IsTestnet</a>&gt;(<a href="CoreAddresses.md#0x0_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>());
}
</code></pre>



</details>

<a name="0x0_Testnet_Specification"></a>

## Specification


This module is used by Genesis, LibraAccount and Unhosted as part of
the initialization and to check whether the transaction is on the
testnet.
*

<a name="0x0_Testnet_@Module_Specification"></a>

### Module Specification


Verify all functions in this module, including private ones.


<pre><code>pragma verify = <b>true</b>;
</code></pre>


Returns the association root address.


<a name="0x0_Testnet_spec_root_address"></a>


<pre><code><b>define</b> <a href="#0x0_Testnet_spec_root_address">spec_root_address</a>(): address { 0xA550C18 }
</code></pre>


Returns true if the testnet has been initialized.
Initialization can be reversed by function
<code>remove_testnet</code>.


<a name="0x0_Testnet_spec_is_initialized"></a>


<pre><code><b>define</b> <a href="#0x0_Testnet_spec_is_initialized">spec_is_initialized</a>(): bool {
    exists&lt;<a href="#0x0_Testnet_IsTestnet">IsTestnet</a>&gt;(<a href="#0x0_Testnet_spec_root_address">spec_root_address</a>())
}
</code></pre>



<a name="0x0_Testnet_@Management_of_TestNet"></a>

#### Management of TestNet


Returns true if no address has IsTestNet resource.


<a name="0x0_Testnet_spec_no_addr_has_testnet"></a>


<pre><code><b>define</b> <a href="#0x0_Testnet_spec_no_addr_has_testnet">spec_no_addr_has_testnet</a>(): bool {
    all(domain&lt;address&gt;(), |addr| !exists&lt;<a href="#0x0_Testnet_IsTestnet">IsTestnet</a>&gt;(addr))
}
</code></pre>


Returns true if only the root address has an IsTestNet resource.


<a name="0x0_Testnet_spec_only_root_addr_has_testnet"></a>


<pre><code><b>define</b> <a href="#0x0_Testnet_spec_only_root_addr_has_testnet">spec_only_root_addr_has_testnet</a>(): bool {
    all(domain&lt;address&gt;(), |addr|
        exists&lt;<a href="#0x0_Testnet_IsTestnet">IsTestnet</a>&gt;(addr)
            ==&gt; addr == <a href="#0x0_Testnet_spec_root_address">spec_root_address</a>())
}
</code></pre>




<a name="0x0_Testnet_OnlyRootAddressHasTestNet"></a>

Base case of the induction before the association address is
initialized with IsTestnet.


<pre><code><b>schema</b> <a href="#0x0_Testnet_OnlyRootAddressHasTestNet">OnlyRootAddressHasTestNet</a> {
    <b>invariant</b> <b>module</b> !<a href="#0x0_Testnet_spec_is_initialized">spec_is_initialized</a>()
                        ==&gt; <a href="#0x0_Testnet_spec_no_addr_has_testnet">spec_no_addr_has_testnet</a>();
}
</code></pre>


Inductive step after the initialization is complete.


<pre><code><b>schema</b> <a href="#0x0_Testnet_OnlyRootAddressHasTestNet">OnlyRootAddressHasTestNet</a> {
    <b>invariant</b> <b>module</b> <a href="#0x0_Testnet_spec_is_initialized">spec_is_initialized</a>()
                        ==&gt; <a href="#0x0_Testnet_spec_only_root_addr_has_testnet">spec_only_root_addr_has_testnet</a>();
}
</code></pre>




<a name="0x0_Testnet_TestNetStaysInitialized"></a>


<pre><code><b>schema</b> <a href="#0x0_Testnet_TestNetStaysInitialized">TestNetStaysInitialized</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x0_Testnet_spec_is_initialized">spec_is_initialized</a>()) ==&gt; <a href="#0x0_Testnet_spec_is_initialized">spec_is_initialized</a>();
}
</code></pre>



Apply 'OnlyRootAddressHasTestNet' to all functions.


<pre><code><b>apply</b> <a href="#0x0_Testnet_OnlyRootAddressHasTestNet">OnlyRootAddressHasTestNet</a> <b>to</b> *;
</code></pre>


Apply 'TestNetStaysInitialized' to all functions except
<code>remove_testnet</code>


<pre><code><b>apply</b> <a href="#0x0_Testnet_TestNetStaysInitialized">TestNetStaysInitialized</a> <b>to</b> * <b>except</b> remove_testnet;
</code></pre>



<a name="0x0_Testnet_@Specifications_for_individual_functions"></a>

#### Specifications for individual functions


Returns true if IsTestNet exists at the address of the account.


<a name="0x0_Testnet_spec_exists_testnet"></a>


<pre><code><b>define</b> <a href="#0x0_Testnet_spec_exists_testnet">spec_exists_testnet</a>(account: signer): bool {
    exists&lt;<a href="#0x0_Testnet_IsTestnet">IsTestnet</a>&gt;(<a href="Signer.md#0x0_Signer_get_address">Signer::get_address</a>(account))
}
</code></pre>



<a name="0x0_Testnet_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Testnet_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x0_Signer_get_address">Signer::get_address</a>(account) != <a href="#0x0_Testnet_spec_root_address">spec_root_address</a>();
<b>aborts_if</b> <a href="#0x0_Testnet_spec_exists_testnet">spec_exists_testnet</a>(account);
<b>ensures</b> <a href="#0x0_Testnet_spec_exists_testnet">spec_exists_testnet</a>(account);
</code></pre>



<a name="0x0_Testnet_Specification_is_testnet"></a>

### Function `is_testnet`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Testnet_is_testnet">is_testnet</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == exists&lt;<a href="#0x0_Testnet_IsTestnet">IsTestnet</a>&gt;(<a href="#0x0_Testnet_spec_root_address">spec_root_address</a>());
</code></pre>



<a name="0x0_Testnet_Specification_remove_testnet"></a>

### Function `remove_testnet`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Testnet_remove_testnet">remove_testnet</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x0_Signer_get_address">Signer::get_address</a>(account) != <a href="#0x0_Testnet_spec_root_address">spec_root_address</a>();
<b>aborts_if</b> !<a href="#0x0_Testnet_spec_exists_testnet">spec_exists_testnet</a>(account);
<b>ensures</b> !<a href="#0x0_Testnet_spec_exists_testnet">spec_exists_testnet</a>(account);
</code></pre>
