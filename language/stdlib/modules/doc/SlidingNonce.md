
<a name="0x1_SlidingNonce"></a>

# Module `0x1::SlidingNonce`

Allows transactions to be executed out-of-order while ensuring that they are executed at most once.
Nonces are assigned to transactions off-chain by clients submitting the transactions.
It maintains a sliding window bitvector of 128 flags.  A flag of 0 indicates that the transaction
with that nonce has not yet been executed.
When nonce X is recorded, all transactions with nonces lower then X-128 will abort.


-  [Resource `SlidingNonce`](#0x1_SlidingNonce_SlidingNonce)
-  [Constants](#@Constants_0)
-  [Function `record_nonce_or_abort`](#0x1_SlidingNonce_record_nonce_or_abort)
-  [Function `try_record_nonce`](#0x1_SlidingNonce_try_record_nonce)
-  [Function `publish`](#0x1_SlidingNonce_publish)
-  [Module Specification](#@Module_Specification_1)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_SlidingNonce_SlidingNonce"></a>

## Resource `SlidingNonce`



<pre><code><b>resource</b> <b>struct</b> <a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>min_nonce: u64</code>
</dt>
<dd>
 Minimum nonce in sliding window. All transactions with smaller
 nonces will be automatically rejected, since the window cannot
 tell whether they have been executed or not.
</dd>
<dt>
<code>nonce_mask: u128</code>
</dt>
<dd>
 Bit-vector of window of nonce values
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_SlidingNonce_ENONCE_ALREADY_PUBLISHED"></a>

The sliding nonce resource was already published


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_PUBLISHED">ENONCE_ALREADY_PUBLISHED</a>: u64 = 4;
</code></pre>



<a name="0x1_SlidingNonce_ENONCE_ALREADY_RECORDED"></a>

The nonce was already recorded previously


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">ENONCE_ALREADY_RECORDED</a>: u64 = 3;
</code></pre>



<a name="0x1_SlidingNonce_ENONCE_TOO_NEW"></a>

The nonce is too large - this protects against nonce exhaustion


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">ENONCE_TOO_NEW</a>: u64 = 2;
</code></pre>



<a name="0x1_SlidingNonce_ENONCE_TOO_OLD"></a>

The nonce aborted because it's too old (nonce smaller than <code>min_nonce</code>)


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">ENONCE_TOO_OLD</a>: u64 = 1;
</code></pre>



<a name="0x1_SlidingNonce_ESLIDING_NONCE"></a>

The <code><a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">ESLIDING_NONCE</a>: u64 = 0;
</code></pre>



<a name="0x1_SlidingNonce_NONCE_MASK_SIZE"></a>

Size of SlidingNonce::nonce_mask in bits.


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_NONCE_MASK_SIZE">NONCE_MASK_SIZE</a>: u64 = 128;
</code></pre>



<a name="0x1_SlidingNonce_record_nonce_or_abort"></a>

## Function `record_nonce_or_abort`

Calls <code>try_record_nonce</code> and aborts transaction if returned code is non-0


<pre><code><b>public</b> <b>fun</b> <a href="SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">record_nonce_or_abort</a>(account: &signer, seq_nonce: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">record_nonce_or_abort</a>(account: &signer, seq_nonce: u64) <b>acquires</b> <a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a> {
    <b>let</b> code = <a href="SlidingNonce.md#0x1_SlidingNonce_try_record_nonce">try_record_nonce</a>(account, seq_nonce);
    <b>assert</b>(code == 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(code));
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">RecordNonceAbortsIf</a>;
</code></pre>




<a name="0x1_SlidingNonce_RecordNonceAbortsIf"></a>


<pre><code><b>schema</b> <a href="SlidingNonce.md#0x1_SlidingNonce_RecordNonceAbortsIf">RecordNonceAbortsIf</a> {
    account: signer;
    seq_nonce: u64;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
    <b>aborts_if</b> <a href="SlidingNonce.md#0x1_SlidingNonce_spec_try_record_nonce">spec_try_record_nonce</a>(account, seq_nonce) != 0 <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>



</details>

<a name="0x1_SlidingNonce_try_record_nonce"></a>

## Function `try_record_nonce`

Tries to record this nonce in the account.
Returns 0 if a nonce was recorded and non-0 otherwise


<pre><code><b>public</b> <b>fun</b> <a href="SlidingNonce.md#0x1_SlidingNonce_try_record_nonce">try_record_nonce</a>(account: &signer, seq_nonce: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SlidingNonce.md#0x1_SlidingNonce_try_record_nonce">try_record_nonce</a>(account: &signer, seq_nonce: u64): u64 <b>acquires</b> <a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a> {
    <b>if</b> (seq_nonce == 0) {
        <b>return</b> 0
    };
    <b>assert</b>(<b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">ESLIDING_NONCE</a>));
    <b>let</b> t = borrow_global_mut&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>if</b> (t.min_nonce &gt; seq_nonce) {
        <b>return</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">ENONCE_TOO_OLD</a>
    };
    <b>let</b> jump_limit = 10000; // Don't allow giant leaps in nonce <b>to</b> protect against nonce exhaustion
    <b>if</b> (t.min_nonce + jump_limit &lt;= seq_nonce) {
        <b>return</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">ENONCE_TOO_NEW</a>
    };
    <b>let</b> bit_pos = seq_nonce - t.min_nonce;
    <b>if</b> (bit_pos &gt;= <a href="SlidingNonce.md#0x1_SlidingNonce_NONCE_MASK_SIZE">NONCE_MASK_SIZE</a>) {
        <b>let</b> shift = (bit_pos - <a href="SlidingNonce.md#0x1_SlidingNonce_NONCE_MASK_SIZE">NONCE_MASK_SIZE</a> + 1);
        <b>if</b>(shift &gt;= <a href="SlidingNonce.md#0x1_SlidingNonce_NONCE_MASK_SIZE">NONCE_MASK_SIZE</a>) {
            t.nonce_mask = 0;
            t.min_nonce = seq_nonce + 1 - <a href="SlidingNonce.md#0x1_SlidingNonce_NONCE_MASK_SIZE">NONCE_MASK_SIZE</a>;
        } <b>else</b> {
            t.nonce_mask = t.nonce_mask &gt;&gt; (shift <b>as</b> u8);
            t.min_nonce = t.min_nonce + shift;
        }
    };
    <b>let</b> bit_pos = seq_nonce - t.min_nonce;
    <b>let</b> set = 1u128 &lt;&lt; (bit_pos <b>as</b> u8);
    <b>if</b> (t.nonce_mask & set != 0) {
        <b>return</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">ENONCE_ALREADY_RECORDED</a>
    };
    t.nonce_mask = t.nonce_mask | set;
    0
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


It is currently assumed that this function raises no arithmetic overflow/underflow.
>Note: Verification is turned off. For verifying callers, this is effectively abstracted into a function
that returns arbitrary results because <code>spec_try_record_nonce</code> is uninterpreted.


<pre><code><b>pragma</b> opaque, verify = <b>false</b>;
<b>ensures</b> result == <a href="SlidingNonce.md#0x1_SlidingNonce_spec_try_record_nonce">spec_try_record_nonce</a>(account, seq_nonce);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>


Specification version of <code><a href="SlidingNonce.md#0x1_SlidingNonce_try_record_nonce">Self::try_record_nonce</a></code>.


<a name="0x1_SlidingNonce_spec_try_record_nonce"></a>


<pre><code><b>define</b> <a href="SlidingNonce.md#0x1_SlidingNonce_spec_try_record_nonce">spec_try_record_nonce</a>(account: signer, seq_nonce: u64): u64;
</code></pre>



</details>

<a name="0x1_SlidingNonce_publish"></a>

## Function `publish`

Publishes nonce resource for <code>account</code>
This is required before other functions in this module can be called for `account


<pre><code><b>public</b> <b>fun</b> <a href="SlidingNonce.md#0x1_SlidingNonce_publish">publish</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SlidingNonce.md#0x1_SlidingNonce_publish">publish</a>(account: &signer) {
    <b>assert</b>(!<b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_PUBLISHED">ENONCE_ALREADY_PUBLISHED</a>));
    move_to(account, <a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a> {  min_nonce: 0, nonce_mask: 0 });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> opaque;
<b>modifies</b> <b>global</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
<b>ensures</b> <b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



Sliding nonces are initialized at Diem root and treasury compliance addresses


<pre><code><b>invariant</b> [<b>global</b>] <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>()
    ==&gt; <b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>());
<b>invariant</b> [<b>global</b>] <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>()
    ==&gt; <b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>());
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/master/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/master/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/master/dips/dip-2.md#permissions
