## Table of Content
-  [Module `0x0::LibraDocTest`](#0x0_LibraDocTest)
    -  [Struct `T`](#0x0_LibraDocTest_T)
    -  [Struct `MintCapability`](#0x0_LibraDocTest_MintCapability)
    -  [Struct `Info`](#0x0_LibraDocTest_Info)
    -  [Struct `Preburn`](#0x0_LibraDocTest_Preburn)
    -  [Function `register`](#0x0_LibraDocTest_register)
    -  [Function `assert_is_registered`](#0x0_LibraDocTest_assert_is_registered)
    -  [Function `mint`](#0x0_LibraDocTest_mint)
    -  [Function `burn`](#0x0_LibraDocTest_burn)
    -  [Function `cancel_burn`](#0x0_LibraDocTest_cancel_burn)
    -  [Function `new_preburn`](#0x0_LibraDocTest_new_preburn)
    -  [Function `mint_with_capability`](#0x0_LibraDocTest_mint_with_capability)
    -  [Function `preburn`](#0x0_LibraDocTest_preburn)
    -  [Function `preburn_to_sender`](#0x0_LibraDocTest_preburn_to_sender)
    -  [Function `burn_with_capability`](#0x0_LibraDocTest_burn_with_capability)
    -  [Function `cancel_burn_with_capability`](#0x0_LibraDocTest_cancel_burn_with_capability)
    -  [Function `publish_preburn`](#0x0_LibraDocTest_publish_preburn)
    -  [Function `remove_preburn`](#0x0_LibraDocTest_remove_preburn)
    -  [Function `destroy_preburn`](#0x0_LibraDocTest_destroy_preburn)
    -  [Function `publish_mint_capability`](#0x0_LibraDocTest_publish_mint_capability)
    -  [Function `remove_mint_capability`](#0x0_LibraDocTest_remove_mint_capability)
    -  [Function `market_cap`](#0x0_LibraDocTest_market_cap)
    -  [Function `preburn_value`](#0x0_LibraDocTest_preburn_value)
    -  [Function `zero`](#0x0_LibraDocTest_zero)
    -  [Function `value`](#0x0_LibraDocTest_value)
    -  [Function `split`](#0x0_LibraDocTest_split)
    -  [Function `withdraw`](#0x0_LibraDocTest_withdraw)
    -  [Function `join`](#0x0_LibraDocTest_join)
    -  [Function `deposit`](#0x0_LibraDocTest_deposit)
    -  [Function `destroy_zero`](#0x0_LibraDocTest_destroy_zero)

<a name="0x0_LibraDocTest"></a>

## Module `0x0::LibraDocTest`

The Libra module defines basic functionality around coins.

### Note

This is not a consistent and up-to-date implementation, specification, or documentation
of the
<code>Libra</code> module. It is rather a playground for testing the documentation generator.

> We use block quotes like this to mark documentation text which is specific to docgen testing.
>
> We can refer to a module like in
<code><a href="#0x0_LibraDocTest">LibraDocTest</a></code> -- if it is unambiguous -- or like
<code><a href="#0x0_LibraDocTest">0x0::LibraDocTest</a></code>.

### Settings for Verification

> Spec module blocks which are associated with the module do not have an implicit header and
> are directly included in the module doc (or Specification sub-section if
<code>--doc-<b>spec</b>-inline=<b>false</b></code>).
> To get a header, one needs to be explicitly assigned like we do here.

Verify also private functions.


<pre><code>pragma verify = <b>true</b>;
</code></pre>



<a name="0x0_LibraDocTest_T"></a>

### Struct `T`

A resource representing a fungible token


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;
</code></pre>



##### Fields


<dl>
<dt>

<code>value: u64</code>
</dt>
<dd>
 The value of the token. May be zero
</dd>
</dl>


##### Specification

Maintains sum_of_token


<pre><code><b>invariant</b> <b>pack</b> sum_of_token_values&lt;Token&gt; = sum_of_token_values&lt;Token&gt; + value;
<b>invariant</b> <b>unpack</b> sum_of_token_values&lt;Token&gt; = sum_of_token_values&lt;Token&gt; - value;
</code></pre>



This ghost variable is defined to have the true sum values of all instances of Token


<pre><code><b>global</b> sum_of_token_values&lt;Token&gt;: num;
</code></pre>




<pre><code><b>schema</b> SumRemainsSame&lt;Token&gt; {
    <b>ensures</b> sum_of_token_values&lt;Token&gt; == <b>old</b>(sum_of_token_values&lt;Token&gt;);
}
</code></pre>



Only mint/burn & with capability versions can change the total amount of currency.
skip mint, burn, mint_with_capability, burn_with_capability.
Burn always aborts because Preburn.is_approved is always false.


<pre><code><b>apply</b> SumRemainsSame&lt;Token&gt; <b>to</b> *&lt;Token&gt; <b>except</b> mint*&lt;Token&gt;, burn*&lt;Token&gt;;
</code></pre>



SPEC: MarketCap == Sum of all the instances of a particular token.
Note: this works even though the verifier does not
know that each existing token.value must be <= sum_of_token_values. I assume that
total_value is constrained to be non-negative, so it all works. I should check this out.
Note: What is the value of a ghost variable in the genesis state?
State machine with two states (not registered/registered), so write as two invariants.


<pre><code><b>schema</b> SumOfTokenValuesInvariant&lt;Token&gt; {
    <b>invariant</b> <b>module</b> !token_is_registered&lt;Token&gt;() ==&gt; sum_of_token_values&lt;Token&gt; == 0;
    <b>invariant</b> <b>module</b> token_is_registered&lt;Token&gt;()
                   ==&gt; sum_of_token_values&lt;Token&gt; == <b>global</b>&lt;<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;&gt;(0xA550C18).total_value;
}
</code></pre>




<pre><code><b>apply</b> SumOfTokenValuesInvariant&lt;Token&gt; <b>to</b> <b>public</b> *&lt;Token&gt;;
</code></pre>



<a name="0x0_LibraDocTest_MintCapability"></a>

### Struct `MintCapability`

A singleton resource that grants access to
<code><a href="#0x0_LibraDocTest_mint">LibraDocTest::mint</a></code>. Only the Association has one.

> Instead of
<code><a href="#0x0_LibraDocTest_mint">LibraDocTest::mint</a></code> we can also write
<code><a href="#0x0_LibraDocTest_mint">0x0::LibraDocTest::mint</a></code>,
<code><a href="#0x0_LibraDocTest_mint">Self::mint</a></code>, or just
<code><a href="#0x0_LibraDocTest_mint">mint</a>()</code>
> (for functions from enclosing module) to get a hyper link in documentation text.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;
</code></pre>



##### Fields


<dl>
<dt>

<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


##### Specification

Maintain mint_capability_count


<pre><code><b>invariant</b> <b>pack</b> mint_capability_count&lt;Token&gt; = mint_capability_count&lt;Token&gt; + 1;
<b>invariant</b> <b>unpack</b> mint_capability_count&lt;Token&gt; = mint_capability_count&lt;Token&gt; - 1;
</code></pre>



Ghost variable representing the total number of MintCapability instances for Token (0 or 1).


<pre><code><b>global</b> mint_capability_count&lt;Token&gt;: num;
</code></pre>


Helper to check whether sender has MintCapability.


<pre><code>define exists_sender_mint_capability&lt;Token&gt;(): bool { exists&lt;<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;&gt;(sender()) }
</code></pre>


There is a MintCapability for Token iff the token is registered
> For the below schema, we have documentation comments on members. In the case of module/function/struct
> spec blocks, we can just flatten all spec block members with possibly
> attached member documentation into the section. For schemas this is not possible.
> So we repeat a schema declaration multiple times, "extending" it.

If token is not registered, there can be no capability.


<pre><code><b>schema</b> MintCapabilityCountInvariant&lt;Token&gt; {
    <b>invariant</b> <b>module</b> !token_is_registered&lt;Token&gt;() ==&gt; mint_capability_count&lt;Token&gt; == 0;
}
</code></pre>


If token is registered, there is exactly one capability.


<pre><code><b>schema</b> MintCapabilityCountInvariant&lt;Token&gt; {
    <b>invariant</b> <b>module</b> token_is_registered&lt;Token&gt;() ==&gt; mint_capability_count&lt;Token&gt; == 1;
}
</code></pre>




<pre><code><b>apply</b> MintCapabilityCountInvariant&lt;Token&gt; <b>to</b> <b>public</b> *&lt;Token&gt;;
</code></pre>



<a name="0x0_LibraDocTest_Info"></a>

### Struct `Info`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;
</code></pre>



##### Fields


<dl>
<dt>

<code>total_value: u128</code>
</dt>
<dd>
 The sum of the values of all
<code><a href="#0x0_LibraDocTest_T">LibraDocTest::T</a></code> resources in the system
</dd>
<dt>

<code>preburn_value: u64</code>
</dt>
<dd>
 Value of funds that are in the process of being burned
</dd>
</dl>


##### Specification


Specifications helpers for working with Info<Token> at association address.


<pre><code>define association_address(): address { 0xA550C18 }
define token_is_registered&lt;Token&gt;(): bool { exists&lt;<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;&gt;(association_address()) }
define info&lt;Token&gt;(): <a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt; { <b>global</b>&lt;<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;&gt;(association_address()) }
</code></pre>


Once registered, a token stays registered forever.


<pre><code><b>schema</b> RegistrationPersists&lt;Token&gt; {
    <b>ensures</b> <b>old</b>(token_is_registered&lt;Token&gt;()) ==&gt; token_is_registered&lt;Token&gt;();
}
</code></pre>




<pre><code><b>apply</b> RegistrationPersists&lt;Token&gt; <b>to</b> *&lt;Token&gt;;
</code></pre>



<a name="0x0_LibraDocTest_Preburn"></a>

### Struct `Preburn`

A holding area where funds that will subsequently be burned wait while their underyling
assets are sold off-chain.
This resource can only be created by the holder of the MintCapability. An account that
contains this address has the authority to initiate a burn request. A burn request can be
resolved by the holder of the MintCapability by either (1) burning the funds, or (2)
returning the funds to the account that initiated the burn request.
This design supports multiple preburn requests in flight at the same time, including multiple
burn requests from the same account. However, burn requests from the same account must be
resolved in FIFO order.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;
</code></pre>



##### Fields


<dl>
<dt>

<code>requests: vector&lt;<a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;&gt;</code>
</dt>
<dd>
 Queue of pending burn requests
</dd>
<dt>

<code>is_approved: bool</code>
</dt>
<dd>
 Boolean that is true if the holder of the MintCapability has approved this account as a
 preburner
</dd>
</dl>


<a name="0x0_LibraDocTest_register"></a>

### Function `register`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_register">register</a>&lt;Token&gt;()
</code></pre>



##### Specification



<pre><code><b>include</b> RegisterAbortsIf&lt;Token&gt;;
<b>ensures</b> exists_sender_mint_capability&lt;Token&gt;();
<b>ensures</b> token_is_registered&lt;Token&gt;();
<b>ensures</b> info&lt;Token&gt;().total_value == 0;
<b>ensures</b> info&lt;Token&gt;().preburn_value == 0;
</code></pre>




<pre><code><b>schema</b> RegisterAbortsIf&lt;Token&gt; {
    <b>aborts_if</b> sender() != association_address();
    <b>aborts_if</b> exists_sender_mint_capability&lt;Token&gt;();
    <b>aborts_if</b> token_is_registered&lt;Token&gt;();
}
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_register">register</a>&lt;Token&gt;() {
    // Only callable by the Association address
    Transaction::assert(Transaction::sender() == 0xA550C18, 1);
    move_to_sender(<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;{ });
    move_to_sender(<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt; { total_value: 0u128, preburn_value: 0 });
}
</code></pre>



<a name="0x0_LibraDocTest_assert_is_registered"></a>

### Function `assert_is_registered`



<pre><code><b>fun</b> <a href="#0x0_LibraDocTest_assert_is_registered">assert_is_registered</a>&lt;Token&gt;()
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> !token_is_registered&lt;Token&gt;();
</code></pre>



##### Implementation


<pre><code><b>fun</b> <a href="#0x0_LibraDocTest_assert_is_registered">assert_is_registered</a>&lt;Token&gt;() {
    Transaction::assert(exists&lt;<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;&gt;(0xA550C18), 12);
}
</code></pre>



<a name="0x0_LibraDocTest_mint"></a>

### Function `mint`

Return
<code>amount</code> coins.
Fails if the sender does not have a published MintCapability.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_mint">mint</a>&lt;Token&gt;(amount: u64): <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;
</code></pre>



##### Specification



<pre><code><b>include</b> MintAbortsIf&lt;Token&gt;;
<b>aborts_if</b> !exists_sender_mint_capability&lt;Token&gt;();
<b>include</b> MintEnsures&lt;Token&gt;;
</code></pre>




<pre><code><b>schema</b> MintAbortsIf&lt;Token&gt; {
    amount: u64;
    <b>aborts_if</b> !token_is_registered&lt;Token&gt;();
    <b>aborts_if</b> amount &gt; 1000000000 * 1000000;
    <b>aborts_if</b> info&lt;Token&gt;().total_value + amount &gt; max_u128();
}
</code></pre>




<pre><code><b>schema</b> MintEnsures&lt;Token&gt; {
    amount: u64;
    result: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;;
    <b>ensures</b> info&lt;Token&gt;().total_value == <b>old</b>(info&lt;Token&gt;().total_value) + amount;
    <b>ensures</b> result.value == amount;
}
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_mint">mint</a>&lt;Token&gt;(amount: u64): <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_LibraDocTest_Info">Info</a>, <a href="#0x0_LibraDocTest_MintCapability">MintCapability</a> {
    <a href="#0x0_LibraDocTest_mint_with_capability">mint_with_capability</a>(amount, borrow_global&lt;<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;&gt;(Transaction::sender()))
}
</code></pre>



<a name="0x0_LibraDocTest_burn"></a>

### Function `burn`

Burn the coins currently held in the preburn holding area under
<code>preburn_address</code>.
Fails if the sender does not have a published MintCapability.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_burn">burn</a>&lt;Token&gt;(preburn_address: address)
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> !exists_sender_mint_capability&lt;Token&gt;();
<b>include</b> BurnAbortsIf&lt;Token&gt;;
<b>include</b> BurnEnsures&lt;Token&gt;;
</code></pre>


Properties applying both to burn and to burn_cancel functions.


<pre><code><b>schema</b> BasicBurnAbortsIf&lt;Token&gt; {
    preburn_address: address;
    <b>aborts_if</b> !exists&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address);
    <b>aborts_if</b> len(<b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests) == 0;
    <b>aborts_if</b> !token_is_registered&lt;Token&gt;();
    <b>aborts_if</b> info&lt;Token&gt;().<a href="#0x0_LibraDocTest_preburn_value">preburn_value</a> &lt; <b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests[0].value;
}
</code></pre>




<pre><code><b>schema</b> BurnAbortsIf&lt;Token&gt; {
    <b>include</b> BasicBurnAbortsIf&lt;Token&gt;;
    <b>aborts_if</b> info&lt;Token&gt;().total_value &lt; <b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests[0].value;
}
</code></pre>




<pre><code><b>schema</b> BurnEnsures&lt;Token&gt; {
    preburn_address: address;
    <b>ensures</b> Vector::eq_pop_front(
                <b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests,
                <b>old</b>(<b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests)
            );
    <b>ensures</b> info&lt;Token&gt;().total_value ==
                <b>old</b>(info&lt;Token&gt;().total_value)
                    - <b>old</b>(<b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests[0].value);
    <b>ensures</b> info&lt;Token&gt;().preburn_value ==
                <b>old</b>(info&lt;Token&gt;().preburn_value)
                    - <b>old</b>(<b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests[0].value);
}
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_burn">burn</a>&lt;Token&gt;(
    preburn_address: address
) <b>acquires</b> <a href="#0x0_LibraDocTest_Info">Info</a>, <a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>, <a href="#0x0_LibraDocTest_Preburn">Preburn</a> {
    <a href="#0x0_LibraDocTest_burn_with_capability">burn_with_capability</a>(
        preburn_address,
        borrow_global&lt;<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;&gt;(Transaction::sender())
    )
}
</code></pre>



<a name="0x0_LibraDocTest_cancel_burn"></a>

### Function `cancel_burn`

Cancel the oldest burn request from
<code>preburn_address</code>
Fails if the sender does not have a published MintCapability.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_cancel_burn">cancel_burn</a>&lt;Token&gt;(preburn_address: address): <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> !exists_sender_mint_capability&lt;Token&gt;();
<b>include</b> BasicBurnAbortsIf&lt;Token&gt;;
<b>include</b> CancelBurnEnsures&lt;Token&gt;;
</code></pre>




<pre><code><b>schema</b> CancelBurnEnsures&lt;Token&gt; {
    preburn_address: address;
    result: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;;
    <b>ensures</b> Vector::eq_pop_front(
                <b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests,
                <b>old</b>(<b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests)
            );
    <b>ensures</b> info&lt;Token&gt;().preburn_value ==
                <b>old</b>(info&lt;Token&gt;().preburn_value) - <b>old</b>(<b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests[0].value);
    <b>ensures</b> result == <b>old</b>(<b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests[0]);
}
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_cancel_burn">cancel_burn</a>&lt;Token&gt;(
    preburn_address: address
): <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_LibraDocTest_Info">Info</a>, <a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>, <a href="#0x0_LibraDocTest_Preburn">Preburn</a> {
    <a href="#0x0_LibraDocTest_cancel_burn_with_capability">cancel_burn_with_capability</a>(
        preburn_address,
        borrow_global&lt;<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;&gt;(Transaction::sender())
    )
}
</code></pre>



<a name="0x0_LibraDocTest_new_preburn"></a>

### Function `new_preburn`

Create a new Preburn resource


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_new_preburn">new_preburn</a>&lt;Token&gt;(): <a href="#0x0_LibraDocTest_Preburn">LibraDocTest::Preburn</a>&lt;Token&gt;
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> !token_is_registered&lt;Token&gt;();
<b>ensures</b> len(result.requests) == 0;
<b>ensures</b> result.is_approved == <b>false</b>;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_new_preburn">new_preburn</a>&lt;Token&gt;(): <a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt; {
    <a href="#0x0_LibraDocTest_assert_is_registered">assert_is_registered</a>&lt;Token&gt;();
    <a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt; { requests: Vector::empty(), is_approved: <b>false</b>, }
}
</code></pre>



<a name="0x0_LibraDocTest_mint_with_capability"></a>

### Function `mint_with_capability`

Mint a new
<code><a href="#0x0_LibraDocTest_T">LibraDocTest::T</a></code> worth
<code>value</code>. The caller must have a reference to a MintCapability.
Only the Association account can acquire such a reference, and it can do so only via
<code>borrow_sender_mint_capability</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_mint_with_capability">mint_with_capability</a>&lt;Token&gt;(value: u64, _capability: &<a href="#0x0_LibraDocTest_MintCapability">LibraDocTest::MintCapability</a>&lt;Token&gt;): <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;
</code></pre>



##### Specification



<pre><code><b>include</b> MintAbortsIf&lt;Token&gt;{amount: value};
<b>include</b> MintEnsures&lt;Token&gt;{amount: value};
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_mint_with_capability">mint_with_capability</a>&lt;Token&gt;(
    value: u64,
    _capability: &<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;
): <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_LibraDocTest_Info">Info</a> {
    <a href="#0x0_LibraDocTest_assert_is_registered">assert_is_registered</a>&lt;Token&gt;();
    // TODO: temporary measure for testnet only: limit minting <b>to</b> 1B Libra at a time.
    // this is <b>to</b> prevent the market cap's total value from hitting u64_max due <b>to</b> excessive
    // minting. This will not be a problem in the production Libra system because coins will
    // be backed with real-world assets, and thus minting will be correspondingly rarer.
    // * 1000000 here because the unit is microlibra
    Transaction::assert(<a href="#0x0_LibraDocTest_value">value</a> &lt;= 1000000000 * 1000000, 11);
    // <b>update</b> market cap <b>resource</b> <b>to</b> reflect minting
    <b>let</b> market_cap = borrow_global_mut&lt;<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;&gt;(0xA550C18);
    market_cap.total_value = market_cap.total_value + (value <b>as</b> u128);

    <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt; { value }
}
</code></pre>



<a name="0x0_LibraDocTest_preburn"></a>

### Function `preburn`

Send coin to the preburn holding area
<code>preburn_ref</code>, where it will wait to be burned.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_preburn">preburn</a>&lt;Token&gt;(preburn_ref: &<b>mut</b> <a href="#0x0_LibraDocTest_Preburn">LibraDocTest::Preburn</a>&lt;Token&gt;, coin: <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;)
</code></pre>



##### Specification



<pre><code><b>include</b> PreburnAbortsIf&lt;Token&gt;;
<b>include</b> PreburnEnsures&lt;Token&gt;;
</code></pre>




<pre><code><b>schema</b> PreburnAbortsIf&lt;Token&gt; {
    coin: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;;
    <b>aborts_if</b> !token_is_registered&lt;Token&gt;();
    <b>aborts_if</b> info&lt;Token&gt;().preburn_value + coin.value &gt; max_u64();
}
</code></pre>




<pre><code><b>schema</b> PreburnEnsures&lt;Token&gt; {
    preburn_ref: &<b>mut</b> <a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;;
    coin: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;;
    <b>ensures</b> info&lt;Token&gt;().preburn_value == <b>old</b>(info&lt;Token&gt;().preburn_value) + coin.value;
    <b>ensures</b> Vector::eq_push_back(preburn_ref.requests, <b>old</b>(preburn_ref.requests), coin);
}
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_preburn">preburn</a>&lt;Token&gt;(
    preburn_ref: &<b>mut</b> <a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;,
    coin: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;
) <b>acquires</b> <a href="#0x0_LibraDocTest_Info">Info</a> {
    // TODO: bring this back once we can automate approvals in testnet
    // Transaction::assert(preburn_ref.is_approved, 13);
    <b>let</b> coin_value = <a href="#0x0_LibraDocTest_value">value</a>(&coin);
    Vector::push_back(
        &<b>mut</b> preburn_ref.requests,
        coin
    );
    <b>let</b> market_cap = borrow_global_mut&lt;<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;&gt;(0xA550C18);
    market_cap.preburn_value = market_cap.preburn_value + coin_value
}
</code></pre>



<a name="0x0_LibraDocTest_preburn_to_sender"></a>

### Function `preburn_to_sender`

Send coin to the preburn holding area, where it will wait to be burned.
Fails if the sender does not have a published Preburn resource


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_preburn_to_sender">preburn_to_sender</a>&lt;Token&gt;(coin: <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;)
</code></pre>



##### Specification



<pre><code><b>include</b> PreburnAbortsIf&lt;Token&gt;;
<b>aborts_if</b> !exists&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(sender());
<b>include</b> PreburnEnsures&lt;Token&gt;{preburn_ref: <b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(sender())};
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_preburn_to_sender">preburn_to_sender</a>&lt;Token&gt;(coin: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;) <b>acquires</b> <a href="#0x0_LibraDocTest_Info">Info</a>, <a href="#0x0_LibraDocTest_Preburn">Preburn</a> {
    <a href="#0x0_LibraDocTest_preburn">preburn</a>(borrow_global_mut&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(Transaction::sender()), coin)
}
</code></pre>



<a name="0x0_LibraDocTest_burn_with_capability"></a>

### Function `burn_with_capability`

Permanently remove the coins held in the
<code><a href="#0x0_LibraDocTest_Preburn">Preburn</a></code> resource stored at
<code>preburn_address</code> and
update the market cap accordingly. If there are multiple preburn requests in progress, this
will remove the oldest one.
Can only be invoked by the holder of the MintCapability. Fails if the there is no
<code><a href="#0x0_LibraDocTest_Preburn">Preburn</a></code>
resource under
<code>preburn_address</code> or has one with no pending burn requests.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_burn_with_capability">burn_with_capability</a>&lt;Token&gt;(preburn_address: address, _capability: &<a href="#0x0_LibraDocTest_MintCapability">LibraDocTest::MintCapability</a>&lt;Token&gt;)
</code></pre>



##### Specification



<pre><code><b>include</b> BurnAbortsIf&lt;Token&gt;;
<b>aborts_if</b> info&lt;Token&gt;().total_value &lt; <b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address).requests[0].value;
<b>include</b> BurnEnsures&lt;Token&gt;;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_burn_with_capability">burn_with_capability</a>&lt;Token&gt;(
    preburn_address: address,
    _capability: &<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;
) <b>acquires</b> <a href="#0x0_LibraDocTest_Info">Info</a>, <a href="#0x0_LibraDocTest_Preburn">Preburn</a> {
    // destroy the coin at the head of the preburn queue
    <b>let</b> preburn = borrow_global_mut&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address);
    <b>let</b> <a href="#0x0_LibraDocTest_T">T</a> { value } = Vector::remove(&<b>mut</b> preburn.requests, 0);
    // <b>update</b> the market cap
    <b>let</b> market_cap = borrow_global_mut&lt;<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;&gt;(0xA550C18);
    market_cap.total_value = market_cap.total_value - (value <b>as</b> u128);
    market_cap.preburn_value = market_cap.preburn_value - value
}
</code></pre>



<a name="0x0_LibraDocTest_cancel_burn_with_capability"></a>

### Function `cancel_burn_with_capability`

Cancel the burn request in the
<code><a href="#0x0_LibraDocTest_Preburn">Preburn</a></code> resource stored at
<code>preburn_address</code> and
return the coins to the caller.
If there are multiple preburn requests in progress, this will cancel the oldest one.
Can only be invoked by the holder of the MintCapability. Fails if the transaction sender
does not have a published Preburn resource or has one with no pending burn requests.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;Token&gt;(preburn_address: address, _capability: &<a href="#0x0_LibraDocTest_MintCapability">LibraDocTest::MintCapability</a>&lt;Token&gt;): <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;
</code></pre>



##### Specification



<pre><code><b>include</b> BasicBurnAbortsIf&lt;Token&gt;;
<b>include</b> CancelBurnEnsures&lt;Token&gt;;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_cancel_burn_with_capability">cancel_burn_with_capability</a>&lt;Token&gt;(
    preburn_address: address,
    _capability: &<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;
): <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_LibraDocTest_Info">Info</a>, <a href="#0x0_LibraDocTest_Preburn">Preburn</a> {
    // destroy the coin at the head of the preburn queue
    <b>let</b> preburn = borrow_global_mut&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(preburn_address);
    <b>let</b> coin = Vector::remove(&<b>mut</b> preburn.requests, 0);
    // <b>update</b> the market cap
    <b>let</b> market_cap = borrow_global_mut&lt;<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;&gt;(0xA550C18);
    market_cap.preburn_value = market_cap.preburn_value - <a href="#0x0_LibraDocTest_value">value</a>(&coin);

    coin
}
</code></pre>



<a name="0x0_LibraDocTest_publish_preburn"></a>

### Function `publish_preburn`

Publish
<code>preburn</code> under the sender's account


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_publish_preburn">publish_preburn</a>&lt;Token&gt;(preburn: <a href="#0x0_LibraDocTest_Preburn">LibraDocTest::Preburn</a>&lt;Token&gt;)
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> exists&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(sender());
<b>ensures</b> exists&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(sender());
<b>ensures</b> <b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(sender()) == preburn;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_publish_preburn">publish_preburn</a>&lt;Token&gt;(preburn: <a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;) {
    move_to_sender(preburn)
}
</code></pre>



<a name="0x0_LibraDocTest_remove_preburn"></a>

### Function `remove_preburn`

Remove and return the
<code><a href="#0x0_LibraDocTest_Preburn">Preburn</a></code> resource under the sender's account


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_remove_preburn">remove_preburn</a>&lt;Token&gt;(): <a href="#0x0_LibraDocTest_Preburn">LibraDocTest::Preburn</a>&lt;Token&gt;
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(sender());
<b>ensures</b> !exists&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(sender());
<b>ensures</b> result == <b>old</b>(<b>global</b>&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(sender()));
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_remove_preburn">remove_preburn</a>&lt;Token&gt;(): <a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_LibraDocTest_Preburn">Preburn</a> {
    move_from&lt;<a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;&gt;(Transaction::sender())
}
</code></pre>



<a name="0x0_LibraDocTest_destroy_preburn"></a>

### Function `destroy_preburn`

Destroys the given preburn resource.
Aborts if
<code>requests</code> is non-empty


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_destroy_preburn">destroy_preburn</a>&lt;Token&gt;(preburn: <a href="#0x0_LibraDocTest_Preburn">LibraDocTest::Preburn</a>&lt;Token&gt;)
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> len(preburn.requests) &gt; 0;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_destroy_preburn">destroy_preburn</a>&lt;Token&gt;(preburn: <a href="#0x0_LibraDocTest_Preburn">Preburn</a>&lt;Token&gt;) {
    <b>let</b> <a href="#0x0_LibraDocTest_Preburn">Preburn</a> { requests, is_approved: _ } = preburn;
    Vector::destroy_empty(requests)
}
</code></pre>



<a name="0x0_LibraDocTest_publish_mint_capability"></a>

### Function `publish_mint_capability`

Publish
<code>capability</code> under the sender's account


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_publish_mint_capability">publish_mint_capability</a>&lt;Token&gt;(capability: <a href="#0x0_LibraDocTest_MintCapability">LibraDocTest::MintCapability</a>&lt;Token&gt;)
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> exists_sender_mint_capability&lt;Token&gt;();
<b>ensures</b> exists_sender_mint_capability&lt;Token&gt;();
<b>ensures</b> capability == <b>global</b>&lt;<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;&gt;(sender());
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_publish_mint_capability">publish_mint_capability</a>&lt;Token&gt;(capability: <a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;) {
    move_to_sender(capability)
}
</code></pre>



<a name="0x0_LibraDocTest_remove_mint_capability"></a>

### Function `remove_mint_capability`

Remove and return the MintCapability from the sender's account. Fails if the sender does
not have a published MintCapability


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_remove_mint_capability">remove_mint_capability</a>&lt;Token&gt;(): <a href="#0x0_LibraDocTest_MintCapability">LibraDocTest::MintCapability</a>&lt;Token&gt;
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> !exists_sender_mint_capability&lt;Token&gt;();
<b>ensures</b> !exists_sender_mint_capability&lt;Token&gt;();
<b>ensures</b> result == <b>old</b>(<b>global</b>&lt;<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;&gt;(sender()));
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_remove_mint_capability">remove_mint_capability</a>&lt;Token&gt;(): <a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt; <b>acquires</b> <a href="#0x0_LibraDocTest_MintCapability">MintCapability</a> {
    move_from&lt;<a href="#0x0_LibraDocTest_MintCapability">MintCapability</a>&lt;Token&gt;&gt;(Transaction::sender())
}
</code></pre>



<a name="0x0_LibraDocTest_market_cap"></a>

### Function `market_cap`

Return the total value of all Libra in the system


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_market_cap">market_cap</a>&lt;Token&gt;(): u128
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> !token_is_registered&lt;Token&gt;();
<b>ensures</b> result == info&lt;Token&gt;().total_value;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_market_cap">market_cap</a>&lt;Token&gt;(): u128 <b>acquires</b> <a href="#0x0_LibraDocTest_Info">Info</a> {
    borrow_global&lt;<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;&gt;(0xA550C18).total_value
}
</code></pre>



<a name="0x0_LibraDocTest_preburn_value"></a>

### Function `preburn_value`

Return the total value of Libra to be burned


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_preburn_value">preburn_value</a>&lt;Token&gt;(): u64
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> !token_is_registered&lt;Token&gt;();
<b>ensures</b> result == info&lt;Token&gt;().preburn_value;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_preburn_value">preburn_value</a>&lt;Token&gt;(): u64 <b>acquires</b> <a href="#0x0_LibraDocTest_Info">Info</a> {
    borrow_global&lt;<a href="#0x0_LibraDocTest_Info">Info</a>&lt;Token&gt;&gt;(0xA550C18).preburn_value
}
</code></pre>



<a name="0x0_LibraDocTest_zero"></a>

### Function `zero`

Create a new
<code><a href="#0x0_LibraDocTest_T">LibraDocTest::T</a></code> with a value of 0


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_zero">zero</a>&lt;Token&gt;(): <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> !token_is_registered&lt;Token&gt;();
<b>ensures</b> result.value == 0;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_zero">zero</a>&lt;Token&gt;(): <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt; {
    // prevent silly coin types (e.g., Libra&lt;bool&gt;) from being created
    <a href="#0x0_LibraDocTest_assert_is_registered">assert_is_registered</a>&lt;Token&gt;();
    <a href="#0x0_LibraDocTest_T">T</a> { value: 0 }
}
</code></pre>



<a name="0x0_LibraDocTest_value"></a>

### Function `value`

Public accessor for the value of a coin


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_value">value</a>&lt;Token&gt;(coin_ref: &<a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;): u64
</code></pre>



##### Specification



<pre><code><b>ensures</b> result == coin_ref.value;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_value">value</a>&lt;Token&gt;(coin_ref: &<a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;): u64 {
    coin_ref.value
}
</code></pre>



<a name="0x0_LibraDocTest_split"></a>

### Function `split`

Splits the given coin into two and returns them both
It leverages
<code>withdraw</code> for any verifications of the values


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_split">split</a>&lt;Token&gt;(coin: <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;, amount: u64): (<a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;, <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;)
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> coin.<a href="#0x0_LibraDocTest_value">value</a> &lt; amount;
<b>ensures</b> result_1.value == coin.value - amount;
<b>ensures</b> result_2.value == amount;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_split">split</a>&lt;Token&gt;(coin: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;, amount: u64): (<a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;, <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;) {
    <b>let</b> other = <a href="#0x0_LibraDocTest_withdraw">withdraw</a>(&<b>mut</b> coin, amount);
    (coin, other)
}
</code></pre>



<a name="0x0_LibraDocTest_withdraw"></a>

### Function `withdraw`

"Divides" the given coin into two, where original coin is modified in place
The original coin will have value = original value -
<code>value</code>
The new coin will have a value =
<code>value</code>
Fails if the coins value is less than
<code>value</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_withdraw">withdraw</a>&lt;Token&gt;(coin_ref: &<b>mut</b> <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;, value: u64): <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> coin_ref.<a href="#0x0_LibraDocTest_value">value</a> &lt; value;
<b>ensures</b> coin_ref.value == <b>old</b>(coin_ref.value) - value;
<b>ensures</b> result.value == value;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_withdraw">withdraw</a>&lt;Token&gt;(coin_ref: &<b>mut</b> <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;, value: u64): <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt; {
    // Check that `amount` is less than the coin's value
    Transaction::assert(coin_ref.value &gt;= value, 10);

    // Split the coin
    coin_ref.value = coin_ref.value - value;
    <a href="#0x0_LibraDocTest_T">T</a> { value }
}
</code></pre>



<a name="0x0_LibraDocTest_join"></a>

### Function `join`

Merges two coins and returns a new coin whose value is equal to the sum of the two inputs


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_join">join</a>&lt;Token&gt;(coin1: <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;, coin2: <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;): <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> coin1.value + coin2.value &gt; max_u64();
<b>ensures</b> result.value == coin1.value + coin2.value;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_join">join</a>&lt;Token&gt;(coin1: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;, coin2: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;): <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;  {
    <a href="#0x0_LibraDocTest_deposit">deposit</a>(&<b>mut</b> coin1, coin2);
    coin1
}
</code></pre>



<a name="0x0_LibraDocTest_deposit"></a>

### Function `deposit`

"Merges" the two coins
The coin passed in by reference will have a value equal to the sum of the two coins
The
<code>check</code> coin is consumed in the process


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_deposit">deposit</a>&lt;Token&gt;(coin_ref: &<b>mut</b> <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;, check: <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;)
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> coin_ref.value + check.value &gt; max_u64();
<b>ensures</b> coin_ref.value == <b>old</b>(coin_ref.value) + check.value;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_deposit">deposit</a>&lt;Token&gt;(coin_ref: &<b>mut</b> <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;, check: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;) {
    <b>let</b> <a href="#0x0_LibraDocTest_T">T</a> { value } = check;
    coin_ref.value= coin_ref.value + value;
}
</code></pre>



<a name="0x0_LibraDocTest_destroy_zero"></a>

### Function `destroy_zero`

Destroy a coin
Fails if the value is non-zero
The amount of
<code><a href="#0x0_LibraDocTest_T">LibraDocTest::T</a></code> in the system is a tightly controlled property,
so you cannot "burn" any non-zero amount of
<code><a href="#0x0_LibraDocTest_T">LibraDocTest::T</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_destroy_zero">destroy_zero</a>&lt;Token&gt;(coin: <a href="#0x0_LibraDocTest_T">LibraDocTest::T</a>&lt;Token&gt;)
</code></pre>



##### Specification



<pre><code><b>aborts_if</b> coin.value &gt; 0;
</code></pre>



##### Implementation


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraDocTest_destroy_zero">destroy_zero</a>&lt;Token&gt;(coin: <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt;) {
    <b>let</b> <a href="#0x0_LibraDocTest_T">T</a>&lt;Token&gt; { value } = coin;
    Transaction::assert(value == 0, 11);
}
</code></pre>
