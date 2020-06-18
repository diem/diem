
<a name="0x1_RecoveryAddress"></a>

# Module `0x1::RecoveryAddress`

### Table of Contents

-  [Struct `RecoveryAddress`](#0x1_RecoveryAddress_RecoveryAddress)
-  [Function `publish`](#0x1_RecoveryAddress_publish)
-  [Function `rotate_authentication_key`](#0x1_RecoveryAddress_rotate_authentication_key)
-  [Function `add_rotation_capability`](#0x1_RecoveryAddress_add_rotation_capability)
-  [Specification](#0x1_RecoveryAddress_Specification)
    -  [Module specifications](#0x1_RecoveryAddress_@Module_specifications)
        -  [RecoveryAddress has its own KeyRotationCapability](#0x1_RecoveryAddress_@RecoveryAddress_has_its_own_KeyRotationCapability)
        -  [RecoveryAddress resource stays](#0x1_RecoveryAddress_@RecoveryAddress_resource_stays)
        -  [RecoveryAddress remains same](#0x1_RecoveryAddress_@RecoveryAddress_remains_same)
        -  [Only VASPs can be RecoveryAddress](#0x1_RecoveryAddress_@Only_VASPs_can_be_RecoveryAddress)
    -  [Function `publish`](#0x1_RecoveryAddress_Specification_publish)
    -  [Specifications for individual functions](#0x1_RecoveryAddress_@Specifications_for_individual_functions)
    -  [Function `rotate_authentication_key`](#0x1_RecoveryAddress_Specification_rotate_authentication_key)
    -  [Function `add_rotation_capability`](#0x1_RecoveryAddress_Specification_add_rotation_capability)



<a name="0x1_RecoveryAddress_RecoveryAddress"></a>

## Struct `RecoveryAddress`

A resource that holds the
<code>KeyRotationCapability</code>s for several accounts belonging to the
same VASP. A VASP account that delegates its
<code>KeyRotationCapability</code> to
a
<code><a href="#0x1_RecoveryAddress">RecoveryAddress</a></code> resource retains the ability to rotate its own authentication key,
but also allows the account that stores the
<code><a href="#0x1_RecoveryAddress">RecoveryAddress</a></code> resource to rotate its
authentication key.
This is useful as an account recovery mechanism: VASP accounts can all delegate their
rotation capabilities to a single
<code><a href="#0x1_RecoveryAddress">RecoveryAddress</a></code> resource stored under address A.
The authentication key for A can be "buried in the mountain" and dug up only if the need to
recover one of accounts in
<code>rotation_caps</code> arises.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_RecoveryAddress">RecoveryAddress</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>rotation_caps: vector&lt;<a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_RecoveryAddress_publish"></a>

## Function `publish`

Extract the
<code>KeyRotationCapability</code> for
<code>recovery_account</code> and publish it in a
<code><a href="#0x1_RecoveryAddress">RecoveryAddress</a></code> resource under
<code>recovery_account</code>.
Aborts if
<code>recovery_account</code> has delegated its
<code>KeyRotationCapability</code>, already has a
<code><a href="#0x1_RecoveryAddress">RecoveryAddress</a></code> resource, or is not a VASP.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RecoveryAddress_publish">publish</a>(recovery_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RecoveryAddress_publish">publish</a>(recovery_account: &signer) {
    // Only VASPs can create a recovery address
    // TODO: proper error code
    <b>assert</b>(<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(recovery_account)), 2222);
    // put the rotation capability for the recovery account itself in `rotation_caps`. This
    // <b>ensures</b> two things:
    // (1) It's not possible <b>to</b> get into a "recovery cycle" where A is the recovery account for
    //     B and B is the recovery account for A
    // (2) rotation_caps is always nonempty
    <b>let</b> rotation_cap = <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(recovery_account);
    // TODO: proper error code
    <b>assert</b>(*<a href="LibraAccount.md#0x1_LibraAccount_key_rotation_capability_address">LibraAccount::key_rotation_capability_address</a>(&rotation_cap)
         == <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(recovery_account), 2222);
    move_to(
        recovery_account,
        <a href="#0x1_RecoveryAddress">RecoveryAddress</a> { rotation_caps: <a href="Vector.md#0x1_Vector_singleton">Vector::singleton</a>(rotation_cap) }
    )
}
</code></pre>



</details>

<a name="0x1_RecoveryAddress_rotate_authentication_key"></a>

## Function `rotate_authentication_key`

Rotate the authentication key of
<code>to_recover</code> to
<code>new_key</code>. Can be invoked by either
<code>recovery_address</code> or
<code>to_recover</code>.
Aborts if
<code>recovery_address</code> does not have the
<code>KeyRotationCapability</code> for
<code>to_recover</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RecoveryAddress_rotate_authentication_key">rotate_authentication_key</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RecoveryAddress_rotate_authentication_key">rotate_authentication_key</a>(
    account: &signer,
    recovery_address: address,
    to_recover: address,
    new_key: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x1_RecoveryAddress">RecoveryAddress</a> {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Both the original owner `to_recover` of the KeyRotationCapability and the
    // `recovery_address` can rotate the authentication key
    // TODO: proper error code
    <b>assert</b>(sender == recovery_address || sender == to_recover, 3333);

    <b>let</b> caps = &borrow_global&lt;<a href="#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(recovery_address).rotation_caps;
    <b>let</b> i = 0;
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(caps);
    <b>while</b> ({
        <b>spec</b> {
            <b>assert</b> i &lt;= len + 1;
            <b>assert</b> forall j in 0..i: caps[j].account_address != to_recover;
        };
        (i &lt;= len)
    })
    {
        <b>let</b> cap = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(caps, i);
        <b>if</b> (<a href="LibraAccount.md#0x1_LibraAccount_key_rotation_capability_address">LibraAccount::key_rotation_capability_address</a>(cap) == &to_recover) {
            <a href="LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">LibraAccount::rotate_authentication_key</a>(cap, new_key);
            <b>return</b>
        };
        i = i + 1
    };
    <b>spec</b> {
        <b>assert</b> i == len + 1;
        <b>assert</b> forall j in 0..len: caps[j].account_address != to_recover;
    };
    // Couldn't find `to_recover` in the account recovery <b>resource</b>; <b>abort</b>
    // TODO: proper error code
    <b>abort</b>(555)
}
</code></pre>



</details>

<a name="0x1_RecoveryAddress_add_rotation_capability"></a>

## Function `add_rotation_capability`

Add the
<code>KeyRotationCapability</code> for
<code>to_recover_account</code> to the
<code><a href="#0x1_RecoveryAddress">RecoveryAddress</a></code>
resource under
<code>recovery_address</code>.
Aborts if
<code>to_recovery_account</code> and
<code>to_recovery_address belong <b>to</b> different VASPs, <b>if</b>
</code>recovery_address
<code> does not have a </code>RecoveryAddress
<code> <b>resource</b>, or <b>if</b>
</code>to_recover_account
<code> has already extracted its </code>KeyRotationCapability`.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RecoveryAddress_add_rotation_capability">add_rotation_capability</a>(to_recover_account: &signer, recovery_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RecoveryAddress_add_rotation_capability">add_rotation_capability</a>(to_recover_account: &signer, recovery_address: address)
<b>acquires</b> <a href="#0x1_RecoveryAddress">RecoveryAddress</a> {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(to_recover_account);
    // Only accept the rotation capability <b>if</b> both accounts belong <b>to</b> the same <a href="VASP.md#0x1_VASP">VASP</a>
    <b>assert</b>(
        <a href="VASP.md#0x1_VASP_parent_address">VASP::parent_address</a>(recovery_address) ==
            <a href="VASP.md#0x1_VASP_parent_address">VASP::parent_address</a>(addr),
        444 // TODO: proper error code
    );

    <b>let</b> caps = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(recovery_address).rotation_caps;
    <b>let</b> rotation_cap = <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">LibraAccount::extract_key_rotation_capability</a>(to_recover_account);
    // TODO: proper error code
    <b>assert</b>(*<a href="LibraAccount.md#0x1_LibraAccount_key_rotation_capability_address">LibraAccount::key_rotation_capability_address</a>(&rotation_cap)
         == <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(to_recover_account), 2222);
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(caps, rotation_cap);
}
</code></pre>



</details>

<a name="0x1_RecoveryAddress_Specification"></a>

## Specification


<a name="0x1_RecoveryAddress_@Module_specifications"></a>

### Module specifications



<pre><code>pragma verify = <b>true</b>;
</code></pre>



Returns true if
<code>addr</code> is a recovery address.


<a name="0x1_RecoveryAddress_spec_is_recovery_address"></a>


<pre><code><b>define</b> <a href="#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(addr: address): bool
{
    exists&lt;<a href="#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(addr)
}
</code></pre>


Returns all the
<code>KeyRotationCapability</code>s held at
<code>recovery_address</code>.


<a name="0x1_RecoveryAddress_spec_get_rotation_caps"></a>


<pre><code><b>define</b> <a href="#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address: address):
    vector&lt;<a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>&gt;
{
    <b>global</b>&lt;<a href="#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(recovery_address).rotation_caps
}
</code></pre>


Returns true if
<code>recovery_address</code> holds the
<code>KeyRotationCapability</code> for
<code>addr</code>.


<a name="0x1_RecoveryAddress_spec_holds_key_rotation_cap_for"></a>


<pre><code><b>define</b> <a href="#0x1_RecoveryAddress_spec_holds_key_rotation_cap_for">spec_holds_key_rotation_cap_for</a>(
    recovery_address: address,
    addr: address): bool
{
    // BUG: the commented out version will <b>break</b> the postconditions.
    // exists i in 0..len(<a href="#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)):
    //     <a href="#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)[i].account_address == addr
    exists i: u64
        where 0 &lt;= i && i &lt; len(<a href="#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)):
            <a href="#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)[i].account_address == addr
}
</code></pre>



<a name="0x1_RecoveryAddress_@RecoveryAddress_has_its_own_KeyRotationCapability"></a>

#### RecoveryAddress has its own KeyRotationCapability



<a name="0x1_RecoveryAddress_RecoveryAddressHasItsOwnKeyRotationCap"></a>


<pre><code><b>schema</b> <a href="#0x1_RecoveryAddress_RecoveryAddressHasItsOwnKeyRotationCap">RecoveryAddressHasItsOwnKeyRotationCap</a> {
    <b>invariant</b> <b>module</b> forall addr1: address
        where <a href="#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(addr1):
            len(<a href="#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(addr1)) &gt; 0
            && <a href="#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(addr1)[0].account_address == addr1;
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_RecoveryAddress_RecoveryAddressHasItsOwnKeyRotationCap">RecoveryAddressHasItsOwnKeyRotationCap</a> <b>to</b> *;
</code></pre>



<a name="0x1_RecoveryAddress_@RecoveryAddress_resource_stays"></a>

#### RecoveryAddress resource stays



<a name="0x1_RecoveryAddress_RecoveryAddressStays"></a>


<pre><code><b>schema</b> <a href="#0x1_RecoveryAddress_RecoveryAddressStays">RecoveryAddressStays</a> {
    <b>ensures</b> forall addr1: address:
        <b>old</b>(<a href="#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(addr1))
        ==&gt; <a href="#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(addr1);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_RecoveryAddress_RecoveryAddressStays">RecoveryAddressStays</a> <b>to</b> *;
</code></pre>



<a name="0x1_RecoveryAddress_@RecoveryAddress_remains_same"></a>

#### RecoveryAddress remains same



<a name="0x1_RecoveryAddress_RecoveryAddressRemainsSame"></a>


<pre><code><b>schema</b> <a href="#0x1_RecoveryAddress_RecoveryAddressRemainsSame">RecoveryAddressRemainsSame</a> {
    <b>ensures</b> forall recovery_addr: address, to_recovery_addr: address
        where <b>old</b>(<a href="#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(recovery_addr)):
            <b>old</b>(<a href="#0x1_RecoveryAddress_spec_holds_key_rotation_cap_for">spec_holds_key_rotation_cap_for</a>(recovery_addr, to_recovery_addr))
            ==&gt; <a href="#0x1_RecoveryAddress_spec_holds_key_rotation_cap_for">spec_holds_key_rotation_cap_for</a>(recovery_addr, to_recovery_addr);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_RecoveryAddress_RecoveryAddressRemainsSame">RecoveryAddressRemainsSame</a> <b>to</b> *;
</code></pre>



<a name="0x1_RecoveryAddress_@Only_VASPs_can_be_RecoveryAddress"></a>

#### Only VASPs can be RecoveryAddress



<a name="0x1_RecoveryAddress_RecoveryAddressIsVASP"></a>


<pre><code><b>schema</b> <a href="#0x1_RecoveryAddress_RecoveryAddressIsVASP">RecoveryAddressIsVASP</a> {
    <b>invariant</b> <b>module</b> forall recovery_addr: address
        where <a href="#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(recovery_addr):
            <a href="VASP.md#0x1_VASP_spec_is_vasp">VASP::spec_is_vasp</a>(recovery_addr);
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_RecoveryAddress_RecoveryAddressIsVASP">RecoveryAddressIsVASP</a> <b>to</b> *;
</code></pre>



<a name="0x1_RecoveryAddress_Specification_publish"></a>

### Function `publish`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RecoveryAddress_publish">publish</a>(recovery_account: &signer)
</code></pre>



<a name="0x1_RecoveryAddress_@Specifications_for_individual_functions"></a>

### Specifications for individual functions



<pre><code><b>aborts_if</b> !exists&lt;<a href="LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>&gt;(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(recovery_account));
<b>aborts_if</b> !<a href="VASP.md#0x1_VASP_spec_is_vasp">VASP::spec_is_vasp</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(recovery_account));
<b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_spec_holds_own_key_rotation_cap">LibraAccount::spec_holds_own_key_rotation_cap</a>(
    <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(recovery_account));
<b>aborts_if</b> <a href="#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(recovery_account));
<b>ensures</b> <a href="#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(recovery_account));
</code></pre>



<a name="0x1_RecoveryAddress_Specification_rotate_authentication_key"></a>

### Function `rotate_authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RecoveryAddress_rotate_authentication_key">rotate_authentication_key</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(recovery_address);
<b>aborts_if</b> !exists&lt;<a href="LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>&gt;(to_recover);
<b>aborts_if</b> len(new_key) != 32;
<b>aborts_if</b> !<a href="#0x1_RecoveryAddress_spec_holds_key_rotation_cap_for">spec_holds_key_rotation_cap_for</a>(recovery_address, to_recover);
<b>aborts_if</b> !(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account) == recovery_address
            || <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account) == to_recover);
<b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>&gt;(to_recover).authentication_key == new_key;
</code></pre>



<a name="0x1_RecoveryAddress_Specification_add_rotation_capability"></a>

### Function `add_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_RecoveryAddress_add_rotation_capability">add_rotation_capability</a>(to_recover_account: &signer, recovery_address: address)
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>&gt;(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(to_recover_account));
<b>aborts_if</b> !<a href="#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(recovery_address);
<b>aborts_if</b> <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(recovery_address)
       != <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(to_recover_account));
<b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_spec_holds_own_key_rotation_cap">LibraAccount::spec_holds_own_key_rotation_cap</a>(
    <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(to_recover_account));
<b>aborts_if</b> !<a href="VASP.md#0x1_VASP_spec_is_vasp">VASP::spec_is_vasp</a>(<a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(to_recover_account));
<b>ensures</b> <a href="#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)[
    len(<a href="#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)) - 1].account_address == <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(to_recover_account);
</code></pre>
