
<a name="0x1_SlidingNonce"></a>

# Module `0x1::SlidingNonce`



-  [Resource <code><a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code>](#0x1_SlidingNonce_SlidingNonce)
-  [Const <code><a href="SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">ESLIDING_NONCE</a></code>](#0x1_SlidingNonce_ESLIDING_NONCE)
-  [Const <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">ENONCE_TOO_OLD</a></code>](#0x1_SlidingNonce_ENONCE_TOO_OLD)
-  [Const <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">ENONCE_TOO_NEW</a></code>](#0x1_SlidingNonce_ENONCE_TOO_NEW)
-  [Const <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">ENONCE_ALREADY_RECORDED</a></code>](#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED)
-  [Const <code><a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_PUBLISHED">ENONCE_ALREADY_PUBLISHED</a></code>](#0x1_SlidingNonce_ENONCE_ALREADY_PUBLISHED)
-  [Const <code><a href="SlidingNonce.md#0x1_SlidingNonce_NONCE_MASK_SIZE">NONCE_MASK_SIZE</a></code>](#0x1_SlidingNonce_NONCE_MASK_SIZE)
-  [Function <code>record_nonce_or_abort</code>](#0x1_SlidingNonce_record_nonce_or_abort)
-  [Function <code>try_record_nonce</code>](#0x1_SlidingNonce_try_record_nonce)
-  [Function <code>publish</code>](#0x1_SlidingNonce_publish)
-  [Function <code>publish_nonce_resource</code>](#0x1_SlidingNonce_publish_nonce_resource)
-  [Module Specification](#@Module_Specification_0)


<a name="0x1_SlidingNonce_SlidingNonce"></a>

## Resource `SlidingNonce`

This struct keep last 128 nonce values in a bit map nonce_mask
We assume that nonce are generated incrementally, but certain permutation is allowed when nonce are recorded
For example you can record nonce 10 and then record nonce 9
When nonce X is recorded, all nonce lower then X-128 will be rejected with code 10001(see below)
In a nutshell, min_nonce records minimal nonce allowed
And nonce_mask contains a bitmap for nonce in range [min_nonce; min_nonce+127]


<pre><code><b>resource</b> <b>struct</b> <a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>min_nonce: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>nonce_mask: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_SlidingNonce_ESLIDING_NONCE"></a>

## Const `ESLIDING_NONCE`

The <code><a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ESLIDING_NONCE">ESLIDING_NONCE</a>: u64 = 0;
</code></pre>



<a name="0x1_SlidingNonce_ENONCE_TOO_OLD"></a>

## Const `ENONCE_TOO_OLD`

The nonce is too old and impossible to ensure whether it's duplicated or not


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_OLD">ENONCE_TOO_OLD</a>: u64 = 1;
</code></pre>



<a name="0x1_SlidingNonce_ENONCE_TOO_NEW"></a>

## Const `ENONCE_TOO_NEW`

The nonce is too far in the future - this is not allowed to protect against nonce exhaustion


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_TOO_NEW">ENONCE_TOO_NEW</a>: u64 = 2;
</code></pre>



<a name="0x1_SlidingNonce_ENONCE_ALREADY_RECORDED"></a>

## Const `ENONCE_ALREADY_RECORDED`

The nonce was already recorded previously


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_RECORDED">ENONCE_ALREADY_RECORDED</a>: u64 = 3;
</code></pre>



<a name="0x1_SlidingNonce_ENONCE_ALREADY_PUBLISHED"></a>

## Const `ENONCE_ALREADY_PUBLISHED`

The sliding nonce resource was already published


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_PUBLISHED">ENONCE_ALREADY_PUBLISHED</a>: u64 = 4;
</code></pre>



<a name="0x1_SlidingNonce_NONCE_MASK_SIZE"></a>

## Const `NONCE_MASK_SIZE`

Size of SlidingNonce::nonce_mask in bits.


<pre><code><b>const</b> <a href="SlidingNonce.md#0x1_SlidingNonce_NONCE_MASK_SIZE">NONCE_MASK_SIZE</a>: u64 = 128;
</code></pre>



<a name="0x1_SlidingNonce_record_nonce_or_abort"></a>

## Function `record_nonce_or_abort`

Calls try_record_nonce and aborts transaction if returned code is non-0


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


> TODO: turn verify on when we are ready to specify this function.
It is currently assumed that this function raises no arithmetic overflow/underflow.


<pre><code><b>pragma</b> opaque, verify = <b>false</b>;
<b>ensures</b> result == <a href="SlidingNonce.md#0x1_SlidingNonce_spec_try_record_nonce">spec_try_record_nonce</a>(account, seq_nonce);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
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
    <b>assert</b>(!<b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_PUBLISHED">ENONCE_ALREADY_PUBLISHED</a>));
    move_to(account, <a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a> {  min_nonce: 0, nonce_mask: 0 });
}
</code></pre>



</details>

<a name="0x1_SlidingNonce_publish_nonce_resource"></a>

## Function `publish_nonce_resource`

Publishes nonce resource into specific account
Only the libra root account can create this resource for different accounts


<pre><code><b>public</b> <b>fun</b> <a href="SlidingNonce.md#0x1_SlidingNonce_publish_nonce_resource">publish_nonce_resource</a>(lr_account: &signer, account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SlidingNonce.md#0x1_SlidingNonce_publish_nonce_resource">publish_nonce_resource</a>(
    lr_account: &signer,
    account: &signer
) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <b>let</b> new_resource = <a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a> {
        min_nonce: 0,
        nonce_mask: 0,
    };
    <b>assert</b>(!<b>exists</b>&lt;<a href="SlidingNonce.md#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="SlidingNonce.md#0x1_SlidingNonce_ENONCE_ALREADY_PUBLISHED">ENONCE_ALREADY_PUBLISHED</a>));
    move_to(account, new_resource);
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification

Specification version of <code><a href="SlidingNonce.md#0x1_SlidingNonce_try_record_nonce">Self::try_record_nonce</a></code>.


<a name="0x1_SlidingNonce_spec_try_record_nonce"></a>


<pre><code><b>define</b> <a href="SlidingNonce.md#0x1_SlidingNonce_spec_try_record_nonce">spec_try_record_nonce</a>(account: signer, seq_nonce: u64): u64;
</code></pre>
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions
