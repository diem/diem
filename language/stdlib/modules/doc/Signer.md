
<a name="0x1_Signer"></a>

# Module `0x1::Signer`



-  [Function <code>borrow_address</code>](#0x1_Signer_borrow_address)
-  [Function <code>address_of</code>](#0x1_Signer_address_of)


<a name="0x1_Signer_borrow_address"></a>

## Function `borrow_address`



<pre><code><b>public</b> <b>fun</b> <a href="Signer.md#0x1_Signer_borrow_address">borrow_address</a>(s: &signer): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Signer.md#0x1_Signer_borrow_address">borrow_address</a>(s: &signer): &address;
</code></pre>



</details>

<a name="0x1_Signer_address_of"></a>

## Function `address_of`



<pre><code><b>public</b> <b>fun</b> <a href="Signer.md#0x1_Signer_address_of">address_of</a>(s: &signer): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Signer.md#0x1_Signer_address_of">address_of</a>(s: &signer): address {
    *<a href="Signer.md#0x1_Signer_borrow_address">borrow_address</a>(s)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="Signer.md#0x1_Signer_spec_address_of">spec_address_of</a>(s);
</code></pre>



Specification version of <code><a href="Signer.md#0x1_Signer_address_of">Self::address_of</a></code>.


<a name="0x1_Signer_spec_address_of"></a>


<pre><code><b>native</b> <b>define</b> <a href="Signer.md#0x1_Signer_spec_address_of">spec_address_of</a>(account: signer): address;
</code></pre>



</details>

[]: # (File containing markdown style reference definitions to be included in each generated doc)
