
<a name="0x1_CRSN"></a>

# Module `0x1::CRSN`

A module implementing conflict-resistant sequence numbers (CRSNs).
The specification, and formal description of the acceptance and rejection
criteria, force expiration and window shifting of CRSNs are described in DIP-168.


-  [Resource `CRSN`](#0x1_CRSN_CRSN)
-  [Constants](#@Constants_0)
-  [Function `publish`](#0x1_CRSN_publish)
-  [Function `record`](#0x1_CRSN_record)
-  [Function `check`](#0x1_CRSN_check)
-  [Function `force_expire`](#0x1_CRSN_force_expire)
-  [Function `has_crsn`](#0x1_CRSN_has_crsn)
-  [Function `shift_window_right`](#0x1_CRSN_shift_window_right)


<pre><code><b>use</b> <a href="../../../../../../move-stdlib/docs/BitVector.md#0x1_BitVector">0x1::BitVector</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_CRSN_CRSN"></a>

## Resource `CRSN`

A CRSN  represents a finite slice or window of an "infinite" bitvector
starting at zero with a size <code>k</code> defined dynamically at the time of
publication of CRSN resource. The <code>min_nonce</code> defines the left-hand
side of the slice, and the slice's state is held in <code>slots</code> and is of size <code>k</code>.
Diagrammatically:
```
1111...000000100001000000...0100001000000...0000...
^             ...                ^
|____..._____slots______...______|
min_nonce                       min_nonce + k - 1
```


<pre><code><b>struct</b> <a href="CRSN.md#0x1_CRSN">CRSN</a> has key
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
<code>slots: <a href="../../../../../../move-stdlib/docs/BitVector.md#0x1_BitVector_BitVector">BitVector::BitVector</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_CRSN_EHAS_CRSN"></a>

A CRSN resource wasn't expected, but one was found


<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_EHAS_CRSN">EHAS_CRSN</a>: u64 = 1;
</code></pre>



<a name="0x1_CRSN_ENO_CRSN"></a>

No CRSN resource exists


<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_ENO_CRSN">ENO_CRSN</a>: u64 = 0;
</code></pre>



<a name="0x1_CRSN_EZERO_SIZE_CRSN"></a>

The size given to the CRSN at the time of publishing was zero, which is not supported


<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_EZERO_SIZE_CRSN">EZERO_SIZE_CRSN</a>: u64 = 2;
</code></pre>



<a name="0x1_CRSN_publish"></a>

## Function `publish`

Publish a DSN under <code>account</code>. Cannot already have a DSN published.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="CRSN.md#0x1_CRSN_publish">publish</a>(account: &signer, min_nonce: u64, size: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="CRSN.md#0x1_CRSN_publish">publish</a>(account: &signer, min_nonce: u64, size: u64) {
    <b>assert</b>(!<a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="CRSN.md#0x1_CRSN_EHAS_CRSN">EHAS_CRSN</a>));
    <b>assert</b>(size &gt; 0, <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="CRSN.md#0x1_CRSN_EZERO_SIZE_CRSN">EZERO_SIZE_CRSN</a>));
    move_to(account, <a href="CRSN.md#0x1_CRSN">CRSN</a> {
        min_nonce,
        slots: <a href="../../../../../../move-stdlib/docs/BitVector.md#0x1_BitVector_new">BitVector::new</a>(size),
    })
}
</code></pre>



</details>

<a name="0x1_CRSN_record"></a>

## Function `record`

Record <code>sequence_nonce</code> under the <code>account</code>. Returns true if
<code>sequence_nonce</code> is accepted, returns false if the <code>sequence_nonce</code> is rejected.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="CRSN.md#0x1_CRSN_record">record</a>(account: &signer, sequence_nonce: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="CRSN.md#0x1_CRSN_record">record</a>(account: &signer, sequence_nonce: u64): bool
<b>acquires</b> <a href="CRSN.md#0x1_CRSN">CRSN</a> {
    <b>if</b> (<a href="CRSN.md#0x1_CRSN_check">check</a>(account, sequence_nonce)) {
        // <a href="CRSN.md#0x1_CRSN">CRSN</a> <b>exists</b> by `check`.
        <b>let</b> crsn = borrow_global_mut&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(<a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
        // accept nonce
        <b>let</b> scaled_nonce = sequence_nonce - crsn.min_nonce;
        <a href="../../../../../../move-stdlib/docs/BitVector.md#0x1_BitVector_set">BitVector::set</a>(&<b>mut</b> crsn.slots, scaled_nonce);
        <a href="CRSN.md#0x1_CRSN_shift_window_right">shift_window_right</a>(crsn);
        <b>true</b>
    } <b>else</b> {
        <b>false</b>
    }
}
</code></pre>



</details>

<a name="0x1_CRSN_check"></a>

## Function `check`

A stateless version of <code>record</code>: returns <code><b>true</b></code> if the <code>sequence_nonce</code>
will be accepted, and <code><b>false</b></code> otherwise.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="CRSN.md#0x1_CRSN_check">check</a>(account: &signer, sequence_nonce: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="CRSN.md#0x1_CRSN_check">check</a>(account: &signer, sequence_nonce: u64): bool
<b>acquires</b> <a href="CRSN.md#0x1_CRSN">CRSN</a> {
    <b>let</b> addr = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(<a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(addr), <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="CRSN.md#0x1_CRSN_ENO_CRSN">ENO_CRSN</a>));
    <b>let</b> crsn = borrow_global_mut&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(addr);

    // Don't accept <b>if</b> it's outside of the window
    <b>if</b> ((sequence_nonce &lt; crsn.min_nonce) ||
        (sequence_nonce &gt;= crsn.min_nonce + <a href="../../../../../../move-stdlib/docs/BitVector.md#0x1_BitVector_length">BitVector::length</a>(&crsn.slots))) {
        <b>return</b> <b>false</b>
    };

    // scaled nonce is the index in the window
    <b>let</b> scaled_nonce = sequence_nonce - crsn.min_nonce;

    // Bit already set, reject
    <b>if</b> (<a href="../../../../../../move-stdlib/docs/BitVector.md#0x1_BitVector_is_index_set">BitVector::is_index_set</a>(&crsn.slots, scaled_nonce)) <b>return</b> <b>false</b>;

    // otherwise, accept
    <b>true</b>

}
</code></pre>



</details>

<a name="0x1_CRSN_force_expire"></a>

## Function `force_expire`

Force expire transactions by forcibly shifting the window by
<code>shift_amount</code>. After the window has been shifted by <code>shift_amount</code> it is
then shifted over set bits as define by the <code>shift_window_right</code> function.


<pre><code><b>public</b> <b>fun</b> <a href="CRSN.md#0x1_CRSN_force_expire">force_expire</a>(account: &signer, shift_amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CRSN.md#0x1_CRSN_force_expire">force_expire</a>(account: &signer, shift_amount: u64)
<b>acquires</b> <a href="CRSN.md#0x1_CRSN">CRSN</a> {
    <b>if</b> (shift_amount == 0) <b>return</b>;
    <b>let</b> addr = <a href="../../../../../../move-stdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(<a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(addr), <a href="../../../../../../move-stdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="CRSN.md#0x1_CRSN_ENO_CRSN">ENO_CRSN</a>));
    <b>let</b> crsn = borrow_global_mut&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(addr);

    <a href="../../../../../../move-stdlib/docs/BitVector.md#0x1_BitVector_shift_left">BitVector::shift_left</a>(&<b>mut</b> crsn.slots, shift_amount);

    crsn.min_nonce = crsn.min_nonce + shift_amount;
    // shift over any set bits
    <a href="CRSN.md#0x1_CRSN_shift_window_right">shift_window_right</a>(crsn);
}
</code></pre>



</details>

<a name="0x1_CRSN_has_crsn"></a>

## Function `has_crsn`

Return whether this address has a CRSN resource published under it.


<pre><code><b>public</b> <b>fun</b> <a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(addr: address): bool {
    <b>exists</b>&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_CRSN_shift_window_right"></a>

## Function `shift_window_right`



<pre><code><b>fun</b> <a href="CRSN.md#0x1_CRSN_shift_window_right">shift_window_right</a>(crsn: &<b>mut</b> <a href="CRSN.md#0x1_CRSN_CRSN">CRSN::CRSN</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="CRSN.md#0x1_CRSN_shift_window_right">shift_window_right</a>(crsn: &<b>mut</b> <a href="CRSN.md#0x1_CRSN">CRSN</a>) {
    <b>let</b> index = <a href="../../../../../../move-stdlib/docs/BitVector.md#0x1_BitVector_longest_set_sequence_starting_at">BitVector::longest_set_sequence_starting_at</a>(&crsn.slots, 0);

    // <b>if</b> there is no run of set bits <b>return</b> early
    <b>if</b> (index == 0) <b>return</b>;
    <a href="../../../../../../move-stdlib/docs/BitVector.md#0x1_BitVector_shift_left">BitVector::shift_left</a>(&<b>mut</b> crsn.slots, index);
    crsn.min_nonce = crsn.min_nonce + index;
}
</code></pre>



</details>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/main/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/main/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/main/dips/dip-2.md#permissions
