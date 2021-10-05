
<a name="0x1_CRSN"></a>

# Module `0x1::CRSN`

A module implementing conflict-resistant sequence numbers (CRSNs).
The specification, and formal description of the acceptance and rejection
criteria, force expiration and window shifting of CRSNs are described in DIP-168.


-  [Resource `CRSN`](#0x1_CRSN_CRSN)
-  [Struct `ForceShiftEvent`](#0x1_CRSN_ForceShiftEvent)
-  [Resource `CRSNsAllowed`](#0x1_CRSN_CRSNsAllowed)
-  [Constants](#@Constants_0)
-  [Function `allow_crsns`](#0x1_CRSN_allow_crsns)
-  [Function `publish`](#0x1_CRSN_publish)
-  [Function `record`](#0x1_CRSN_record)
-  [Function `check`](#0x1_CRSN_check)
-  [Function `force_expire`](#0x1_CRSN_force_expire)
-  [Function `has_crsn`](#0x1_CRSN_has_crsn)
-  [Function `shift_window_right`](#0x1_CRSN_shift_window_right)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector">0x1::BitVector</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
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
<code>size: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>slots: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_BitVector">BitVector::BitVector</a></code>
</dt>
<dd>

</dd>
<dt>
<code>force_shift_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="CRSN.md#0x1_CRSN_ForceShiftEvent">CRSN::ForceShiftEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_CRSN_ForceShiftEvent"></a>

## Struct `ForceShiftEvent`

Whenever a force shift is performed a <code><a href="CRSN.md#0x1_CRSN_ForceShiftEvent">ForceShiftEvent</a></code> is emitted.
This is used to prove the absence of a transaction at a specific sequence nonce.


<pre><code><b>struct</b> <a href="CRSN.md#0x1_CRSN_ForceShiftEvent">ForceShiftEvent</a> has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>current_min_nonce: u64</code>
</dt>
<dd>
 current LHS of the CRSN state
</dd>
<dt>
<code>shift_amount: u64</code>
</dt>
<dd>
 The amount the window is being shifted
</dd>
<dt>
<code>bits_at_shift: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_BitVector">BitVector::BitVector</a></code>
</dt>
<dd>
 The state of the bitvector just before the shift. The state of
 the CRSN's bitvector is needed at the time of the shift to prove
 that a CRSNs nonce was expired, and not already used by a transaction
 in the past. This can be used to prove that a transaction can't
 exist from an account because the slot was expired and not used.
 Note: the sequence  nonce of the shifting transaction will not be set.
</dd>
</dl>


</details>

<a name="0x1_CRSN_CRSNsAllowed"></a>

## Resource `CRSNsAllowed`

Flag stored in memory to turn on CRSNs


<pre><code><b>struct</b> <a href="CRSN.md#0x1_CRSN_CRSNsAllowed">CRSNsAllowed</a> has key
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_CRSN_EALREADY_INITIALIZED"></a>

CRSNs were already initialized


<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_EALREADY_INITIALIZED">EALREADY_INITIALIZED</a>: u64 = 6;
</code></pre>



<a name="0x1_CRSN_ECRSN_SIZE_TOO_LARGE"></a>

The size given to the CRSN at the time of publishing was larger than the largest allowed CRSN size


<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_ECRSN_SIZE_TOO_LARGE">ECRSN_SIZE_TOO_LARGE</a>: u64 = 3;
</code></pre>



<a name="0x1_CRSN_EHAS_CRSN"></a>

A CRSN resource wasn't expected, but one was found


<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_EHAS_CRSN">EHAS_CRSN</a>: u64 = 1;
</code></pre>



<a name="0x1_CRSN_EINVALID_SHIFT"></a>

the amount to shift the CRSN window was zero


<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_EINVALID_SHIFT">EINVALID_SHIFT</a>: u64 = 4;
</code></pre>



<a name="0x1_CRSN_ENOT_INITIALIZED"></a>

CRSNs are not yet permitted in the network


<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_ENOT_INITIALIZED">ENOT_INITIALIZED</a>: u64 = 5;
</code></pre>



<a name="0x1_CRSN_ENO_CRSN"></a>

No CRSN resource exists


<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_ENO_CRSN">ENO_CRSN</a>: u64 = 0;
</code></pre>



<a name="0x1_CRSN_EZERO_SIZE_CRSN"></a>

The size given to the CRSN at the time of publishing was zero, which is not supported


<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_EZERO_SIZE_CRSN">EZERO_SIZE_CRSN</a>: u64 = 2;
</code></pre>



<a name="0x1_CRSN_MAX_CRSN_SIZE"></a>



<pre><code><b>const</b> <a href="CRSN.md#0x1_CRSN_MAX_CRSN_SIZE">MAX_CRSN_SIZE</a>: u64 = 256;
</code></pre>



<a name="0x1_CRSN_allow_crsns"></a>

## Function `allow_crsns`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="CRSN.md#0x1_CRSN_allow_crsns">allow_crsns</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="CRSN.md#0x1_CRSN_allow_crsns">allow_crsns</a>(account: &signer) {
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(account);
    <b>assert</b>!(!<b>exists</b>&lt;<a href="CRSN.md#0x1_CRSN_CRSNsAllowed">CRSNsAllowed</a>&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="CRSN.md#0x1_CRSN_EALREADY_INITIALIZED">EALREADY_INITIALIZED</a>));
    move_to(account, <a href="CRSN.md#0x1_CRSN_CRSNsAllowed">CRSNsAllowed</a> { })
}
</code></pre>



</details>

<a name="0x1_CRSN_publish"></a>

## Function `publish`

Publish a DSN under <code>account</code>. Cannot already have a DSN published.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="CRSN.md#0x1_CRSN_publish">publish</a>(account: &signer, min_nonce: u64, size: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="CRSN.md#0x1_CRSN_publish">publish</a>(account: &signer, min_nonce: u64, size: u64) {
    <b>assert</b>!(!<a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="CRSN.md#0x1_CRSN_EHAS_CRSN">EHAS_CRSN</a>));
    <b>assert</b>!(size &gt; 0, <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="CRSN.md#0x1_CRSN_EZERO_SIZE_CRSN">EZERO_SIZE_CRSN</a>));
    <b>assert</b>!(size &lt;= <a href="CRSN.md#0x1_CRSN_MAX_CRSN_SIZE">MAX_CRSN_SIZE</a>, <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="CRSN.md#0x1_CRSN_ECRSN_SIZE_TOO_LARGE">ECRSN_SIZE_TOO_LARGE</a>));
    <b>assert</b>!(<b>exists</b>&lt;<a href="CRSN.md#0x1_CRSN_CRSNsAllowed">CRSNsAllowed</a>&gt;(@DiemRoot), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="CRSN.md#0x1_CRSN_ENOT_INITIALIZED">ENOT_INITIALIZED</a>));
    move_to(account, <a href="CRSN.md#0x1_CRSN">CRSN</a> {
        min_nonce,
        size,
        slots: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_new">BitVector::new</a>(size),
        force_shift_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="CRSN.md#0x1_CRSN_ForceShiftEvent">ForceShiftEvent</a>&gt;(account),
    })
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> BitVector::NewAbortsIf{length: size};
<b>aborts_if</b> !<b>exists</b>&lt;<a href="CRSN.md#0x1_CRSN_CRSNsAllowed">CRSNsAllowed</a>&gt;(@DiemRoot) <b>with</b> Errors::INVALID_STATE;
<b>aborts_if</b> <a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)) <b>with</b> Errors::INVALID_STATE;
<b>aborts_if</b> size == 0 <b>with</b> Errors::INVALID_ARGUMENT;
<b>aborts_if</b> size &gt; <a href="CRSN.md#0x1_CRSN_MAX_CRSN_SIZE">MAX_CRSN_SIZE</a> <b>with</b> Errors::INVALID_ARGUMENT;
<b>ensures</b> <b>exists</b>&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
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
    <b>let</b> addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>if</b> (<a href="CRSN.md#0x1_CRSN_check">check</a>(account, sequence_nonce)) {
        // <a href="CRSN.md#0x1_CRSN">CRSN</a> <b>exists</b> by `check`.
        <b>let</b> crsn = borrow_global_mut&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(addr);
        // accept nonce
        <b>let</b> scaled_nonce = sequence_nonce - crsn.min_nonce;
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_set">BitVector::set</a>(&<b>mut</b> crsn.slots, scaled_nonce);
        <a href="CRSN.md#0x1_CRSN_shift_window_right">shift_window_right</a>(crsn);
        <b>return</b> <b>true</b>
    } <b>else</b> <b>if</b> (<b>exists</b>&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(addr)) { // window was force shifted in this transaction
        <b>let</b> crsn = borrow_global&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(addr);
        <b>if</b> (crsn.min_nonce &gt; sequence_nonce) <b>return</b> <b>true</b>
    };

    <b>false</b>
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
    <b>let</b> addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>!(<a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="CRSN.md#0x1_CRSN_ENO_CRSN">ENO_CRSN</a>));
    <b>let</b> crsn = borrow_global_mut&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(addr);

    // Don't accept <b>if</b> it's outside of the window
    <b>if</b> ((sequence_nonce &lt; crsn.min_nonce) ||
        ((sequence_nonce <b>as</b> u128) &gt;= (crsn.min_nonce <b>as</b> u128) + (<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_length">BitVector::length</a>(&crsn.slots) <b>as</b> u128))) {
        <b>false</b>
    } <b>else</b> {
        // scaled nonce is the index in the window
        <b>let</b> scaled_nonce = sequence_nonce - crsn.min_nonce;

        // Bit already set, reject, otherwise accept
        !<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_is_index_set">BitVector::is_index_set</a>(&crsn.slots, scaled_nonce)
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="CRSN.md#0x1_CRSN_CheckAbortsIf">CheckAbortsIf</a>{addr: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)};
</code></pre>




<a name="0x1_CRSN_CheckAbortsIf"></a>


<pre><code><b>schema</b> <a href="CRSN.md#0x1_CRSN_CheckAbortsIf">CheckAbortsIf</a> {
    addr: address;
    sequence_nonce: u64;
    <b>let</b> crsn = <b>global</b>&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(addr);
    <b>let</b> scaled_nonce = sequence_nonce - crsn.min_nonce;
    <b>aborts_if</b> !<a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(addr) <b>with</b> Errors::INVALID_STATE;
    <b>include</b> <a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(addr) &&
            (sequence_nonce &gt;= crsn.min_nonce) &&
            (sequence_nonce + crsn.min_nonce &lt; <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_length">BitVector::length</a>(crsn.slots))
    ==&gt; BitVector::IsIndexSetAbortsIf{bitvector: crsn.slots, bit_index: scaled_nonce };
}
</code></pre>




<a name="0x1_CRSN_spec_check"></a>


<pre><code><b>fun</b> <a href="CRSN.md#0x1_CRSN_spec_check">spec_check</a>(addr: address, sequence_nonce: u64): bool {
   <b>let</b> crsn = <b>global</b>&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(addr);
   <b>if</b> ((sequence_nonce &lt; crsn.min_nonce) ||
       (sequence_nonce &gt;= crsn.min_nonce + <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_length">BitVector::length</a>(crsn.slots))) {
       <b>false</b>
   } <b>else</b> {
       <b>let</b> scaled_nonce = sequence_nonce - crsn.min_nonce;
       !BitVector::spec_is_index_set(crsn.slots, scaled_nonce)
   }
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
    <b>assert</b>!(shift_amount &gt; 0, <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="CRSN.md#0x1_CRSN_EINVALID_SHIFT">EINVALID_SHIFT</a>));
    <b>let</b> addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>!(<a href="CRSN.md#0x1_CRSN_has_crsn">has_crsn</a>(addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="CRSN.md#0x1_CRSN_ENO_CRSN">ENO_CRSN</a>));
    <b>let</b> crsn = borrow_global_mut&lt;<a href="CRSN.md#0x1_CRSN">CRSN</a>&gt;(addr);

    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> crsn.force_shift_events, <a href="CRSN.md#0x1_CRSN_ForceShiftEvent">ForceShiftEvent</a> {
        current_min_nonce: crsn.min_nonce,
        shift_amount: shift_amount,
        bits_at_shift: *&crsn.slots,
    });

    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_shift_left">BitVector::shift_left</a>(&<b>mut</b> crsn.slots, shift_amount);

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
    <b>let</b> index = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_longest_set_sequence_starting_at">BitVector::longest_set_sequence_starting_at</a>(&crsn.slots, 0);

    // <b>if</b> there is no run of set bits <b>return</b> early
    <b>if</b> (index == 0) <b>return</b>;
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/BitVector.md#0x1_BitVector_shift_left">BitVector::shift_left</a>(&<b>mut</b> crsn.slots, index);
    crsn.min_nonce = crsn.min_nonce + index;
}
</code></pre>



</details>
