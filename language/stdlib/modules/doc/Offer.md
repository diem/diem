
<a name="0x0_Offer"></a>

# Module `0x0::Offer`

### Table of Contents

-  [Struct `Offer`](#0x0_Offer_Offer)
-  [Function `create`](#0x0_Offer_create)
-  [Function `redeem`](#0x0_Offer_redeem)
-  [Function `exists_at`](#0x0_Offer_exists_at)
-  [Function `address_of`](#0x0_Offer_address_of)



<a name="0x0_Offer_Offer"></a>

## Struct `Offer`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Offer">Offer</a>&lt;Offered&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>offered: Offered</code>
</dt>
<dd>

</dd>
<dt>

<code>for: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_Offer_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Offer_create">create</a>&lt;Offered&gt;(account: &signer, offered: Offered, for: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Offer_create">create</a>&lt;Offered&gt;(account: &signer, offered: Offered, for: address) {
  move_to(account, <a href="#0x0_Offer">Offer</a>&lt;Offered&gt; { offered, for });
}
</code></pre>



</details>

<a name="0x0_Offer_redeem"></a>

## Function `redeem`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Offer_redeem">redeem</a>&lt;Offered&gt;(account: &signer, offer_address: address): Offered
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Offer_redeem">redeem</a>&lt;Offered&gt;(account: &signer, offer_address: address): Offered <b>acquires</b> <a href="#0x0_Offer">Offer</a> {
  <b>let</b> <a href="#0x0_Offer">Offer</a>&lt;Offered&gt; { offered, for } = move_from&lt;<a href="#0x0_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
  <b>let</b> sender = <a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account);
  // fail with INSUFFICIENT_PRIVILEGES
  Transaction::assert(sender == for || sender == offer_address, 11);
  offered
}
</code></pre>



</details>

<a name="0x0_Offer_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Offer_exists_at">exists_at</a>&lt;Offered&gt;(offer_address: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Offer_exists_at">exists_at</a>&lt;Offered&gt;(offer_address: address): bool {
  exists&lt;<a href="#0x0_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address)
}
</code></pre>



</details>

<a name="0x0_Offer_address_of"></a>

## Function `address_of`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Offer_address_of">address_of</a>&lt;Offered&gt;(offer_address: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Offer_address_of">address_of</a>&lt;Offered&gt;(offer_address: address): address <b>acquires</b> <a href="#0x0_Offer">Offer</a> {
  borrow_global&lt;<a href="#0x0_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).for
}
</code></pre>



</details>
