
<a name="0x1_Offer"></a>

# Module `0x1::Offer`



-  [Resource `Offer`](#0x1_Offer_Offer)
-  [Const `EOFFER_DNE_FOR_ACCOUNT`](#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT)
-  [Function `create`](#0x1_Offer_create)
-  [Function `redeem`](#0x1_Offer_redeem)
-  [Function `exists_at`](#0x1_Offer_exists_at)
-  [Function `address_of`](#0x1_Offer_address_of)
-  [Module Specification](#@Module_Specification_0)
    -  [Module specification](#@Module_specification_1)
        -  [Creation of Offers](#@Creation_of_Offers_2)
        -  [Removal of Offers](#@Removal_of_Offers_3)


<a name="0x1_Offer_Offer"></a>

## Resource `Offer`



<pre><code><b>resource</b> <b>struct</b> <a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;
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

<a name="0x1_Offer_EOFFER_DNE_FOR_ACCOUNT"></a>

## Const `EOFFER_DNE_FOR_ACCOUNT`

An offer of the specified type for the account does not exist


<pre><code><b>const</b> <a href="Offer.md#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>: u64 = 0;
</code></pre>



<a name="0x1_Offer_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_create">create</a>&lt;Offered&gt;(account: &signer, offered: Offered, for: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_create">create</a>&lt;Offered&gt;(account: &signer, offered: Offered, for: address) {
  move_to(account, <a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt; { offered, for });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Offer a struct to the account under address <code>for</code> by
placing the offer under the signer's address


<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>global</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) == <a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt; { offered: offered, for: for };
</code></pre>



</details>

<a name="0x1_Offer_redeem"></a>

## Function `redeem`



<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_redeem">redeem</a>&lt;Offered&gt;(account: &signer, offer_address: address): Offered
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_redeem">redeem</a>&lt;Offered&gt;(account: &signer, offer_address: address): Offered <b>acquires</b> <a href="Offer.md#0x1_Offer">Offer</a> {
  <b>let</b> <a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt; { offered, for } = move_from&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
  <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
  <b>assert</b>(sender == for || sender == offer_address, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Offer.md#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>));
  offered
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Aborts if there is no offer under <code>offer_address</code> or if the account
cannot redeem the offer.
Ensures that the offered struct under <code>offer_address</code> is removed is returned.


<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
<b>aborts_if</b> !<a href="Offer.md#0x1_Offer_is_allowed_recipient">is_allowed_recipient</a>&lt;Offered&gt;(offer_address, <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>old</b>(<b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address)) && !<b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
<b>ensures</b> result == <b>old</b>(<b>global</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).offered);
</code></pre>



</details>

<a name="0x1_Offer_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_exists_at">exists_at</a>&lt;Offered&gt;(offer_address: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_exists_at">exists_at</a>&lt;Offered&gt;(offer_address: address): bool {
  <b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Returns whether or not an <code><a href="Offer.md#0x1_Offer">Offer</a></code> resource is under the given address <code>offer_address</code>.


<pre><code><b>ensures</b> result == <b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
</code></pre>



</details>

<a name="0x1_Offer_address_of"></a>

## Function `address_of`



<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_address_of">address_of</a>&lt;Offered&gt;(offer_address: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_address_of">address_of</a>&lt;Offered&gt;(offer_address: address): address <b>acquires</b> <a href="Offer.md#0x1_Offer">Offer</a> {
  borrow_global&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).for
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Aborts is there is no offer resource <code><a href="Offer.md#0x1_Offer">Offer</a></code> at the <code>offer_address</code>.
Returns the address of the intended recipient of the Offer
under the <code>offer_address</code>.


<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
<b>ensures</b> result == <b>global</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).for;
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



<a name="@Module_specification_1"></a>

### Module specification


This module defines a resource <code><a href="Offer.md#0x1_Offer">Offer</a></code> that is used as a permissioned trading scheme between accounts.
It defines two main functions for creating and retrieving a struct offered by some user
inside the resource <code><a href="Offer.md#0x1_Offer">Offer</a></code> under the offerer's account.

Currently, the only other module that depends on this module is LibraConfig, where it's used to
pass a capability to an account that allows it to modify a config.


Helper function that returns whether or not the <code>recipient</code> is an intended
recipient of the offered struct in the <code><a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;</code> resource at the address <code>offer_address</code>
Returns true if the recipient is allowed to redeem <code><a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;</code> at <code>offer_address</code>
and false otherwise.

TODO (dd): this is undefined if the offer does not exist. Should this be anded with
"exists_at"?


<a name="0x1_Offer_is_allowed_recipient"></a>


<pre><code><b>define</b> <a href="Offer.md#0x1_Offer_is_allowed_recipient">is_allowed_recipient</a>&lt;Offered&gt;(offer_addr: address, recipient: address): bool {
  recipient == <b>global</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_addr).for || recipient == offer_addr
}
</code></pre>


Mirrors the Move function exists_at<Offered>, above.


<a name="0x1_Offer_spec_exists_at"></a>


<pre><code><b>define</b> <a href="Offer.md#0x1_Offer_spec_exists_at">spec_exists_at</a>&lt;Offered&gt;(offer_addr: address): bool {
    <b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_addr)
}
</code></pre>



<a name="@Creation_of_Offers_2"></a>

#### Creation of Offers



<a name="0x1_Offer_OnlyCreateCanCreateOffer"></a>

Only <code><a href="Offer.md#0x1_Offer_create">Self::create</a></code> can create a resource <code><a href="Offer.md#0x1_Offer">Offer</a></code> under an address.

**Informally:** No function to which this is applied can create an offer.
If there didn't exist an offer under some <code>addr</code>, then it continues
not to have one.


<pre><code><b>schema</b> <a href="Offer.md#0x1_Offer_OnlyCreateCanCreateOffer">OnlyCreateCanCreateOffer</a> {
    <b>ensures</b> <b>forall</b> ty: type, addr: address <b>where</b> !<b>old</b>(<b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;ty&gt;&gt;(addr)) : !<b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;ty&gt;&gt;(addr);
}
</code></pre>



Apply OnlyCreateCanCreateOffer


<pre><code><b>apply</b> <a href="Offer.md#0x1_Offer_OnlyCreateCanCreateOffer">OnlyCreateCanCreateOffer</a> <b>to</b> *&lt;Offered&gt;, * <b>except</b> create;
</code></pre>




<a name="@Removal_of_Offers_3"></a>

#### Removal of Offers



<a name="0x1_Offer_OnlyRedeemCanRemoveOffer"></a>

Only <code><a href="Offer.md#0x1_Offer_redeem">Self::redeem</a></code> can remove the <code><a href="Offer.md#0x1_Offer">Offer</a></code> resource from an account.

**Informally:** No other function except for <code>redeem</code> can remove an offer from an account.


<pre><code><b>schema</b> <a href="Offer.md#0x1_Offer_OnlyRedeemCanRemoveOffer">OnlyRedeemCanRemoveOffer</a> {
    <b>ensures</b> <b>forall</b> ty: type, addr: address <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;ty&gt;&gt;(addr)) :
              (<b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;ty&gt;&gt;(addr) && <b>global</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;ty&gt;&gt;(addr) == <b>old</b>(<b>global</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;ty&gt;&gt;(addr)));
}
</code></pre>



Enforce that every function except <code><a href="Offer.md#0x1_Offer_redeem">Self::redeem</a></code> can remove an offer from the global store.


<pre><code><b>apply</b> <a href="Offer.md#0x1_Offer_OnlyRedeemCanRemoveOffer">OnlyRedeemCanRemoveOffer</a> <b>to</b> *&lt;Offered&gt;, * <b>except</b> redeem;
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
