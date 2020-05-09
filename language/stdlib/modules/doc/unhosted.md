
<a name="0x0_Unhosted"></a>

# Module `0x0::Unhosted`

### Table of Contents

-  [Struct `T`](#0x0_Unhosted_T)
-  [Function `publish_global_limits_definition`](#0x0_Unhosted_publish_global_limits_definition)
-  [Function `create`](#0x0_Unhosted_create)
-  [Function `limits_addr`](#0x0_Unhosted_limits_addr)



<a name="0x0_Unhosted_T"></a>

## Struct `T`



<pre><code><b>struct</b> <a href="#0x0_Unhosted_T">T</a>
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



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Unhosted_publish_global_limits_definition">publish_global_limits_definition</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Unhosted_publish_global_limits_definition">publish_global_limits_definition</a>() {
    Transaction::assert(Transaction::sender() == <a href="#0x0_Unhosted_limits_addr">limits_addr</a>(), 0);
    // These are limits for testnet _only_.
    <a href="account_limits.md#0x0_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>();
    /*<a href="account_limits.md#0x0_AccountLimits_publish_limits_definition">AccountLimits::publish_limits_definition</a>(
        10000,
        10000,
        50000,
        31540000000000
    );*/
    <a href="account_limits.md#0x0_AccountLimits_certify_limits_definition">AccountLimits::certify_limits_definition</a>(<a href="#0x0_Unhosted_limits_addr">limits_addr</a>());
}
</code></pre>



</details>

<a name="0x0_Unhosted_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Unhosted_create">create</a>(): <a href="#0x0_Unhosted_T">Unhosted::T</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Unhosted_create">create</a>(): <a href="#0x0_Unhosted_T">T</a> {
    Transaction::assert(<a href="testnet.md#0x0_Testnet_is_testnet">Testnet::is_testnet</a>(), 10041);
    <a href="#0x0_Unhosted_T">T</a> {  }
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
    0xA550C18
}
</code></pre>



</details>
