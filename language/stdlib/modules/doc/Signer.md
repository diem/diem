
<a name="0x0_Signer"></a>

# Module `0x0::Signer`

### Table of Contents

-  [Function `borrow_address`](#0x0_Signer_borrow_address)
-  [Function `address_of`](#0x0_Signer_address_of)
-  [Specification](#0x0_Signer_Specification)



<a name="0x0_Signer_borrow_address"></a>

## Function `borrow_address`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Signer_borrow_address">borrow_address</a>(s: &signer): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="#0x0_Signer_borrow_address">borrow_address</a>(s: &signer): &address;
</code></pre>



</details>

<a name="0x0_Signer_address_of"></a>

## Function `address_of`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Signer_address_of">address_of</a>(s: &signer): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Signer_address_of">address_of</a>(s: &signer): address {
    *<a href="#0x0_Signer_borrow_address">borrow_address</a>(s)
}
</code></pre>



</details>

<a name="0x0_Signer_Specification"></a>

## Specification



<a name="0x0_Signer_get_address"></a>


<pre><code><b>native</b> <b>define</b> <a href="#0x0_Signer_get_address">get_address</a>(account: signer): address;
</code></pre>
