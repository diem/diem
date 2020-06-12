
<a name="0x1_SlidingNonce"></a>

# Module `0x1::SlidingNonce`

### Table of Contents

-  [Struct `CreateSlidingNonce`](#0x1_SlidingNonce_CreateSlidingNonce)
-  [Struct `SlidingNonce`](#0x1_SlidingNonce_SlidingNonce)
-  [Function `grant_privileges`](#0x1_SlidingNonce_grant_privileges)
-  [Function `record_nonce_or_abort`](#0x1_SlidingNonce_record_nonce_or_abort)
-  [Function `try_record_nonce`](#0x1_SlidingNonce_try_record_nonce)
-  [Function `publish`](#0x1_SlidingNonce_publish)
-  [Function `publish_nonce_resource`](#0x1_SlidingNonce_publish_nonce_resource)



<a name="0x1_SlidingNonce_CreateSlidingNonce"></a>

## Struct `CreateSlidingNonce`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_SlidingNonce_CreateSlidingNonce">CreateSlidingNonce</a>
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

<a name="0x1_SlidingNonce_SlidingNonce"></a>

## Struct `SlidingNonce`

This struct keep last 128 nonce values in a bit map nonce_mask
We assume that nonce are generated incrementally, but certain permutation is allowed when nonce are recorded
For example you can record nonce 10 and then record nonce 9
When nonce X is recorded, all nonce lower then X-128 will be rejected with code 10001(see below)
In a nutshell, min_nonce records minimal nonce allowed
And nonce_mask contains a bitmap for nonce in range [min_nonce; min_nonce+127]


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_SlidingNonce">SlidingNonce</a>
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

<a name="0x1_SlidingNonce_grant_privileges"></a>

## Function `grant_privileges`

Grants the
<code><a href="#0x1_SlidingNonce_CreateSlidingNonce">CreateSlidingNonce</a></code> privilege to the calling
<code>account</code>.
Aborts if the calling account does not have the association root role.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SlidingNonce_grant_privileges">grant_privileges</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SlidingNonce_grant_privileges">grant_privileges</a>(account: &signer) {
    <a href="Roles.md#0x1_Roles_add_privilege_to_account_association_root_role">Roles::add_privilege_to_account_association_root_role</a>(account, <a href="#0x1_SlidingNonce_CreateSlidingNonce">CreateSlidingNonce</a>{});
}
</code></pre>



</details>

<a name="0x1_SlidingNonce_record_nonce_or_abort"></a>

## Function `record_nonce_or_abort`

Calls try_record_nonce and aborts transaction if returned code is non-0


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SlidingNonce_record_nonce_or_abort">record_nonce_or_abort</a>(account: &signer, seq_nonce: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SlidingNonce_record_nonce_or_abort">record_nonce_or_abort</a>(account: &signer, seq_nonce: u64) <b>acquires</b> <a href="#0x1_SlidingNonce">SlidingNonce</a> {
    <b>let</b> code = <a href="#0x1_SlidingNonce_try_record_nonce">try_record_nonce</a>(account, seq_nonce);
    <b>assert</b>(code == 0, code);
}
</code></pre>



</details>

<a name="0x1_SlidingNonce_try_record_nonce"></a>

## Function `try_record_nonce`

Tries to record this nonce in the account.
Returns 0 if a nonce was recorded and non-0 otherwise
Reasons for nonce to be rejected:
* code 10001: This nonce is too old and impossible to ensure whether it's duplicated or not
* code 10002: This nonce is too far in the future - this is not allowed to protect against nonce exhaustion
* code 10003: This nonce was already recorded previously


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SlidingNonce_try_record_nonce">try_record_nonce</a>(account: &signer, seq_nonce: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SlidingNonce_try_record_nonce">try_record_nonce</a>(account: &signer, seq_nonce: u64): u64 <b>acquires</b> <a href="#0x1_SlidingNonce">SlidingNonce</a> {
    <b>if</b> (seq_nonce == 0) {
        <b>return</b> 0
    };
    <b>let</b> t = borrow_global_mut&lt;<a href="#0x1_SlidingNonce">SlidingNonce</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>if</b> (t.min_nonce &gt; seq_nonce) {
        <b>return</b> 10001
    };
    <b>let</b> jump_limit = 10000; // Don't allow giant leaps in nonce <b>to</b> protect against nonce exhaustion
    <b>if</b> (t.min_nonce + jump_limit &lt;= seq_nonce) {
        <b>return</b> 10002
    };
    <b>let</b> bit_pos = seq_nonce - t.min_nonce;
    <b>let</b> nonce_mask_size = 128; // size of SlidingNonce::nonce_mask in bits. no constants in <b>move</b>?
    <b>if</b> (bit_pos &gt;= nonce_mask_size) {
        <b>let</b> shift = (bit_pos - nonce_mask_size + 1);
        <b>if</b>(shift &gt;= nonce_mask_size) {
            t.nonce_mask = 0;
            t.min_nonce = seq_nonce + 1 - nonce_mask_size;
        } <b>else</b> {
            t.nonce_mask = t.nonce_mask &gt;&gt; (shift <b>as</b> u8);
            t.min_nonce = t.min_nonce + shift;
        }
    };
    <b>let</b> bit_pos = seq_nonce - t.min_nonce;
    <b>let</b> set = 1u128 &lt;&lt; (bit_pos <b>as</b> u8);
    <b>if</b> (t.nonce_mask & set != 0) {
        <b>return</b> 10003
    };
    t.nonce_mask = t.nonce_mask | set;
    0
}
</code></pre>



</details>

<a name="0x1_SlidingNonce_publish"></a>

## Function `publish`

Publishes nonce resource for
<code>account</code>
This is required before other functions in this module can be called for `account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SlidingNonce_publish">publish</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SlidingNonce_publish">publish</a>(account: &signer) {
    move_to(account, <a href="#0x1_SlidingNonce">SlidingNonce</a> {  min_nonce: 0, nonce_mask: 0 });
}
</code></pre>



</details>

<a name="0x1_SlidingNonce_publish_nonce_resource"></a>

## Function `publish_nonce_resource`

Publishes nonce resource into specific account
Only association can create this resource for different account
Alternative is publish_nonce_resource_for_user that publishes resource into current account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SlidingNonce_publish_nonce_resource">publish_nonce_resource</a>(_: &<a href="Roles.md#0x1_Roles_Capability">Roles::Capability</a>&lt;<a href="#0x1_SlidingNonce_CreateSlidingNonce">SlidingNonce::CreateSlidingNonce</a>&gt;, account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_SlidingNonce_publish_nonce_resource">publish_nonce_resource</a>(_: &Capability&lt;<a href="#0x1_SlidingNonce_CreateSlidingNonce">CreateSlidingNonce</a>&gt;, account: &signer) {
    <b>let</b> new_resource = <a href="#0x1_SlidingNonce">SlidingNonce</a> {
        min_nonce: 0,
        nonce_mask: 0,
    };
    move_to(account, new_resource)
}
</code></pre>



</details>
