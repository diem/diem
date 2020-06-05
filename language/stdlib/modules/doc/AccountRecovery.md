
<a name="0x0_VASPAccountRecovery"></a>

# Module `0x0::VASPAccountRecovery`

### Table of Contents

-  [Struct `VASPAccountRecovery`](#0x0_VASPAccountRecovery_VASPAccountRecovery)
-  [Function `publish`](#0x0_VASPAccountRecovery_publish)
-  [Function `rotate`](#0x0_VASPAccountRecovery_rotate)



<a name="0x0_VASPAccountRecovery_VASPAccountRecovery"></a>

## Struct `VASPAccountRecovery`

A resource that holds the
<code>KeyRotationCapability</code>s for several accounts belonging to the
same VASP. A VASP account that delegates its
<code>KeyRotationCapability</code> to
a
<code><a href="#0x0_VASPAccountRecovery">VASPAccountRecovery</a></code> resource retains the ability to rotate its own authentication key,
but also allows the account that stores the
<code><a href="#0x0_VASPAccountRecovery">VASPAccountRecovery</a></code> resource to rotate its
authentication key.
This is useful as an account recovery mechanism: VASP accounts can all delegate their
rotation capabilities to a single
<code><a href="#0x0_VASPAccountRecovery">VASPAccountRecovery</a></code> resource stored under address A.
The authentication key for A can be "buried in the mountain" and dug up only if the need to
recover one of accounts in
<code>rotation_caps</code> arises.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_VASPAccountRecovery">VASPAccountRecovery</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>rotation_caps: vector&lt;<a href="LibraAccount.md#0x0_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_VASPAccountRecovery_publish"></a>

## Function `publish`

Extract the
<code>KeyRotationCapability</code> for
<code>recovery_account</code> and publish it in a
<code><a href="#0x0_VASPAccountRecovery">VASPAccountRecovery</a></code> resource under
<code>recovery_account</code>.
Aborts if
<code>recovery_account</code> has delegated its
<code>KeyRotationCapability</code> or already has a
<code><a href="#0x0_VASPAccountRecovery">VASPAccountRecovery</a></code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPAccountRecovery_publish">publish</a>(recovery_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPAccountRecovery_publish">publish</a>(recovery_account: &signer) {
    // put the rotation capability for the recovery account itself in `rotation_caps`. This
    // <b>ensures</b> two things:
    // (1) It's not possible <b>to</b> get into a "recovery cycle" where A is the recovery account for
    //     B and B is the recovery account for A
    // (2) rotation_caps is always nonempty
    <b>let</b> rotation_cap = <a href="LibraAccount.md#0x0_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(recovery_account);
    move_to(
        recovery_account,
        <a href="#0x0_VASPAccountRecovery">VASPAccountRecovery</a> { rotation_caps: <a href="Vector.md#0x0_Vector_singleton">Vector::singleton</a>(rotation_cap) }
    )
}
</code></pre>



</details>

<a name="0x0_VASPAccountRecovery_rotate"></a>

## Function `rotate`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPAccountRecovery_rotate">rotate</a>(recovery_account: &signer, to_recover: address, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_VASPAccountRecovery_rotate">rotate</a>(recovery_account: &signer, to_recover: address, new_key: vector&lt;u8&gt;)
<b>acquires</b> <a href="#0x0_VASPAccountRecovery">VASPAccountRecovery</a> {
    <b>let</b> caps =
        &borrow_global&lt;<a href="#0x0_VASPAccountRecovery">VASPAccountRecovery</a>&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(recovery_account)).rotation_caps;

    <b>let</b> i = 0;
    <b>let</b> len = <a href="Vector.md#0x0_Vector_length">Vector::length</a>(caps);
    <b>while</b> (i &lt; len) {
        <b>let</b> cap = <a href="Vector.md#0x0_Vector_borrow">Vector::borrow</a>(caps, i);
        <b>if</b> (<a href="LibraAccount.md#0x0_LibraAccount_key_rotation_capability_address">LibraAccount::key_rotation_capability_address</a>(cap) == &to_recover) {
            <a href="LibraAccount.md#0x0_LibraAccount_rotate_authentication_key_with_capability">LibraAccount::rotate_authentication_key_with_capability</a>(cap, new_key);
            <b>return</b>
        }
    };
    // Couldn't find `to_recover` in the account recovery <b>resource</b>; <b>abort</b>
    // TODO: proper error code
    <b>abort</b>(555)
}
</code></pre>



</details>
