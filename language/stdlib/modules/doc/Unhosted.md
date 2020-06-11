
<a name="0x0_Unhosted"></a>

# Module `0x0::Unhosted`

### Table of Contents

-  [Struct `Unhosted`](#0x0_Unhosted_Unhosted)
-  [Function `publish_global_limits_definition`](#0x0_Unhosted_publish_global_limits_definition)
-  [Function `create`](#0x0_Unhosted_create)
-  [Function `limits_addr`](#0x0_Unhosted_limits_addr)



<a name="0x0_Unhosted_Unhosted"></a>

## Struct `Unhosted`



<pre><code><b>struct</b> <a href="#0x0_Unhosted">Unhosted</a>
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

<a name="0x0_Unhosted_publish_global_limits_definition"></a>

## Function `publish_global_limits_definition`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Unhosted_publish_global_limits_definition">publish_global_limits_definition</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Unhosted_publish_global_limits_definition">publish_global_limits_definition</a>(account: &signer) {
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account) == <a href="#0x0_Unhosted_limits_addr">limits_addr</a>(), 100042);
    // These are limits for testnet _only_.
    <a href="AccountLimits.md#0x0_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>(account);
    /*<a href="AccountLimits.md#0x0_AccountLimits_publish_limits_definition">AccountLimits::publish_limits_definition</a>(
        10000,
        10000,
        50000,
        31540000000000
    );*/
    <a href="AccountLimits.md#0x0_AccountLimits_certify_limits_definition">AccountLimits::certify_limits_definition</a>(account, <a href="#0x0_Unhosted_limits_addr">limits_addr</a>());
}
</code></pre>



</details>

<a name="0x0_Unhosted_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Unhosted_create">create</a>(): <a href="#0x0_Unhosted_Unhosted">Unhosted::Unhosted</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Unhosted_create">create</a>(): <a href="#0x0_Unhosted">Unhosted</a> {
    Transaction::assert(<a href="Testnet.md#0x0_Testnet_is_testnet">Testnet::is_testnet</a>(), 10041);
    <a href="#0x0_Unhosted">Unhosted</a> {  }
}
</code></pre>



</details>

<a name="0x0_Unhosted_limits_addr"></a>

## Function `limits_addr`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Unhosted_limits_addr">limits_addr</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Unhosted_limits_addr">limits_addr</a>(): address {
    <a href="CoreAddresses.md#0x0_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>()
}
</code></pre>



</details>
