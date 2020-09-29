
<a name="0x1_RecoveryAddress"></a>

# Module `0x1::RecoveryAddress`



-  [Resource <code><a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code>](#0x1_RecoveryAddress_RecoveryAddress)
-  [Const <code><a href="RecoveryAddress.md#0x1_RecoveryAddress_ENOT_A_VASP">ENOT_A_VASP</a></code>](#0x1_RecoveryAddress_ENOT_A_VASP)
-  [Const <code><a href="RecoveryAddress.md#0x1_RecoveryAddress_EKEY_ROTATION_DEPENDENCY_CYCLE">EKEY_ROTATION_DEPENDENCY_CYCLE</a></code>](#0x1_RecoveryAddress_EKEY_ROTATION_DEPENDENCY_CYCLE)
-  [Const <code><a href="RecoveryAddress.md#0x1_RecoveryAddress_ECANNOT_ROTATE_KEY">ECANNOT_ROTATE_KEY</a></code>](#0x1_RecoveryAddress_ECANNOT_ROTATE_KEY)
-  [Const <code><a href="RecoveryAddress.md#0x1_RecoveryAddress_EINVALID_KEY_ROTATION_DELEGATION">EINVALID_KEY_ROTATION_DELEGATION</a></code>](#0x1_RecoveryAddress_EINVALID_KEY_ROTATION_DELEGATION)
-  [Const <code><a href="RecoveryAddress.md#0x1_RecoveryAddress_EACCOUNT_NOT_RECOVERABLE">EACCOUNT_NOT_RECOVERABLE</a></code>](#0x1_RecoveryAddress_EACCOUNT_NOT_RECOVERABLE)
-  [Const <code><a href="RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">ERECOVERY_ADDRESS</a></code>](#0x1_RecoveryAddress_ERECOVERY_ADDRESS)
-  [Function <code>publish</code>](#0x1_RecoveryAddress_publish)
-  [Function <code>rotate_authentication_key</code>](#0x1_RecoveryAddress_rotate_authentication_key)
-  [Function <code>add_rotation_capability</code>](#0x1_RecoveryAddress_add_rotation_capability)
-  [Module Specification](#@Module_Specification_0)
    -  [Module specifications](#@Module_specifications_1)
        -  [RecoveryAddress has its own KeyRotationCapability](#@RecoveryAddress_has_its_own_KeyRotationCapability_2)
        -  [RecoveryAddress resource stays](#@RecoveryAddress_resource_stays_3)
        -  [RecoveryAddress remains same](#@RecoveryAddress_remains_same_4)
        -  [Only VASPs can be RecoveryAddress](#@Only_VASPs_can_be_RecoveryAddress_5)


<a name="0x1_RecoveryAddress_RecoveryAddress"></a>

## Resource `RecoveryAddress`

A resource that holds the <code>KeyRotationCapability</code>s for several accounts belonging to the
same VASP. A VASP account that delegates its <code>KeyRotationCapability</code> to
but also allows the account that stores the <code><a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource to rotate its
authentication key.
This is useful as an account recovery mechanism: VASP accounts can all delegate their
rotation capabilities to a single <code><a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource stored under address A.
The authentication key for A can be "buried in the mountain" and dug up only if the need to
recover one of accounts in <code>rotation_caps</code> arises.


<pre><code><b>resource</b> <b>struct</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>
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

<a name="0x1_RecoveryAddress_ENOT_A_VASP"></a>

## Const `ENOT_A_VASP`

Only VASPs can create a recovery address


<pre><code><b>const</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_ENOT_A_VASP">ENOT_A_VASP</a>: u64 = 0;
</code></pre>



<a name="0x1_RecoveryAddress_EKEY_ROTATION_DEPENDENCY_CYCLE"></a>

## Const `EKEY_ROTATION_DEPENDENCY_CYCLE`

A cycle would have been created would be created


<pre><code><b>const</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_EKEY_ROTATION_DEPENDENCY_CYCLE">EKEY_ROTATION_DEPENDENCY_CYCLE</a>: u64 = 1;
</code></pre>



<a name="0x1_RecoveryAddress_ECANNOT_ROTATE_KEY"></a>

## Const `ECANNOT_ROTATE_KEY`

The signer doesn't have the appropriate privileges to rotate the account's key


<pre><code><b>const</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_ECANNOT_ROTATE_KEY">ECANNOT_ROTATE_KEY</a>: u64 = 2;
</code></pre>



<a name="0x1_RecoveryAddress_EINVALID_KEY_ROTATION_DELEGATION"></a>

## Const `EINVALID_KEY_ROTATION_DELEGATION`

Only accounts belonging to the same VASP can delegate their key rotation capability


<pre><code><b>const</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_EINVALID_KEY_ROTATION_DELEGATION">EINVALID_KEY_ROTATION_DELEGATION</a>: u64 = 3;
</code></pre>



<a name="0x1_RecoveryAddress_EACCOUNT_NOT_RECOVERABLE"></a>

## Const `EACCOUNT_NOT_RECOVERABLE`

The account address couldn't be found in the account recovery resource


<pre><code><b>const</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_EACCOUNT_NOT_RECOVERABLE">EACCOUNT_NOT_RECOVERABLE</a>: u64 = 4;
</code></pre>



<a name="0x1_RecoveryAddress_ERECOVERY_ADDRESS"></a>

## Const `ERECOVERY_ADDRESS`

A <code><a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource was in an unexpected state


<pre><code><b>const</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">ERECOVERY_ADDRESS</a>: u64 = 5;
</code></pre>



<a name="0x1_RecoveryAddress_publish"></a>

## Function `publish`

Extract the <code>KeyRotationCapability</code> for <code>recovery_account</code> and publish it in a
<code><a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource under  <code>recovery_account</code>.
Aborts if <code>recovery_account</code> has delegated its <code>KeyRotationCapability</code>, already has a
<code><a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource, or is not a VASP.


<pre><code><b>public</b> <b>fun</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_publish">publish</a>(recovery_account: &signer, rotation_cap: <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_publish">publish</a>(recovery_account: &signer, rotation_cap: KeyRotationCapability) {
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(recovery_account);
    // Only VASPs can create a recovery address
    <b>assert</b>(<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(addr), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_ENOT_A_VASP">ENOT_A_VASP</a>));
    // put the rotation capability for the recovery account itself in `rotation_caps`. This
    // <b>ensures</b> two things:
    // (1) It's not possible <b>to</b> get into a "recovery cycle" <b>where</b> A is the recovery account for
    //     B and B is the recovery account for A
    // (2) rotation_caps is always nonempty
    <b>assert</b>(
        *<a href="LibraAccount.md#0x1_LibraAccount_key_rotation_capability_address">LibraAccount::key_rotation_capability_address</a>(&rotation_cap) == addr,
         <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_EKEY_ROTATION_DEPENDENCY_CYCLE">EKEY_ROTATION_DEPENDENCY_CYCLE</a>)
    );
    <b>assert</b>(!<b>exists</b>&lt;<a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(addr), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">ERECOVERY_ADDRESS</a>));
    move_to(
        recovery_account,
        <a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a> { rotation_caps: <a href="Vector.md#0x1_Vector_singleton">Vector::singleton</a>(rotation_cap) }
    )
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_PublishAbortsIf">PublishAbortsIf</a>;
<b>include</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_PublishEnsures">PublishEnsures</a>;
</code></pre>




<a name="0x1_RecoveryAddress_PublishAbortsIf"></a>


<pre><code><b>schema</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_PublishAbortsIf">PublishAbortsIf</a> {
    recovery_account: signer;
    rotation_cap: KeyRotationCapability;
    <a name="0x1_RecoveryAddress_addr$6"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(recovery_account);
    <b>aborts_if</b> !<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(addr) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>aborts_if</b> <a href="LibraAccount.md#0x1_LibraAccount_key_rotation_capability_address">LibraAccount::key_rotation_capability_address</a>(rotation_cap) != addr
        <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>




<a name="0x1_RecoveryAddress_PublishEnsures"></a>


<pre><code><b>schema</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_PublishEnsures">PublishEnsures</a> {
    recovery_account: signer;
    rotation_cap: KeyRotationCapability;
    <a name="0x1_RecoveryAddress_addr$7"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(recovery_account);
    <b>ensures</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(addr);
    <b>ensures</b> len(<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(addr)) == 1;
    <b>ensures</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(addr)[0] == rotation_cap;
}
</code></pre>



</details>

<a name="0x1_RecoveryAddress_rotate_authentication_key"></a>

## Function `rotate_authentication_key`

Rotate the authentication key of <code>to_recover</code> to <code>new_key</code>. Can be invoked by either
<code>recovery_address</code> or <code>to_recover</code>.
Aborts if <code>recovery_address</code> does not have the <code>KeyRotationCapability</code> for <code>to_recover</code>.


<pre><code><b>public</b> <b>fun</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_rotate_authentication_key">rotate_authentication_key</a>(account: &signer, recovery_address: address, to_recover: address, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_rotate_authentication_key">rotate_authentication_key</a>(
    account: &signer,
    recovery_address: address,
    to_recover: address,
    new_key: vector&lt;u8&gt;
) <b>acquires</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a> {
    // Check that `recovery_address` has a `<a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>` <b>resource</b>
    <b>assert</b>(<b>exists</b>&lt;<a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(recovery_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">ERECOVERY_ADDRESS</a>));
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(
        // The original owner of a key rotation capability can rotate its own key
        sender == to_recover ||
        // The owner of the `<a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>` <b>resource</b> can rotate any key
        sender == recovery_address,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_ECANNOT_ROTATE_KEY">ECANNOT_ROTATE_KEY</a>)
    );

    <b>let</b> caps = &borrow_global&lt;<a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(recovery_address).rotation_caps;
    <b>let</b> i = 0;
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(caps);
    <b>while</b> ({
        <b>spec</b> {
            <b>assert</b> i &lt;= len;
            <b>assert</b> <b>forall</b> j in 0..i: caps[j].account_address != to_recover;
        };
        (i &lt; len)
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
        <b>assert</b> i == len;
        <b>assert</b> <b>forall</b> j in 0..len: caps[j].account_address != to_recover;
    };
    // Couldn't find `to_recover` in the account recovery <b>resource</b>; <b>abort</b>
    <b>abort</b> <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_EACCOUNT_NOT_RECOVERABLE">EACCOUNT_NOT_RECOVERABLE</a>)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_RotateAuthenticationKeyAbortsIf">RotateAuthenticationKeyAbortsIf</a>;
<b>include</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_RotateAuthenticationKeyEnsures">RotateAuthenticationKeyEnsures</a>;
</code></pre>




<a name="0x1_RecoveryAddress_RotateAuthenticationKeyAbortsIf"></a>


<pre><code><b>schema</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_RotateAuthenticationKeyAbortsIf">RotateAuthenticationKeyAbortsIf</a> {
    account: signer;
    recovery_address: address;
    to_recover: address;
    new_key: vector&lt;u8&gt;;
    <b>aborts_if</b> !<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(recovery_address) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>&gt;(to_recover) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
    <b>aborts_if</b> len(new_key) != 32;
    <b>aborts_if</b> !<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_holds_key_rotation_cap_for">spec_holds_key_rotation_cap_for</a>(recovery_address, to_recover);
    <b>aborts_if</b> !(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) == recovery_address
                || <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) == to_recover);
}
</code></pre>




<a name="0x1_RecoveryAddress_RotateAuthenticationKeyEnsures"></a>


<pre><code><b>schema</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_RotateAuthenticationKeyEnsures">RotateAuthenticationKeyEnsures</a> {
    to_recover: address;
    new_key: vector&lt;u8&gt;;
    <b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>&gt;(to_recover).authentication_key == new_key;
}
</code></pre>



</details>

<a name="0x1_RecoveryAddress_add_rotation_capability"></a>

## Function `add_rotation_capability`

Add <code>to_recover</code> to the <code><a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a></code> resource under <code>recovery_address</code>.
Aborts if <code>to_recover.address</code> and <code>recovery_address belong <b>to</b> different VASPs, or <b>if</b>
</code>recovery_address<code> does not have a </code>RecoveryAddress` resource.


<pre><code><b>public</b> <b>fun</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_add_rotation_capability">add_rotation_capability</a>(to_recover: <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>, recovery_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_add_rotation_capability">add_rotation_capability</a>(to_recover: KeyRotationCapability, recovery_address: address)
<b>acquires</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a> {
    // Check that `recovery_address` has a `<a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>` <b>resource</b>
    <b>assert</b>(<b>exists</b>&lt;<a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(recovery_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_ERECOVERY_ADDRESS">ERECOVERY_ADDRESS</a>));
    // Only accept the rotation capability <b>if</b> both accounts belong <b>to</b> the same <a href="VASP.md#0x1_VASP">VASP</a>
    <b>let</b> to_recover_address = *<a href="LibraAccount.md#0x1_LibraAccount_key_rotation_capability_address">LibraAccount::key_rotation_capability_address</a>(&to_recover);
    <b>assert</b>(
        <a href="VASP.md#0x1_VASP_parent_address">VASP::parent_address</a>(recovery_address) == <a href="VASP.md#0x1_VASP_parent_address">VASP::parent_address</a>(to_recover_address),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_EINVALID_KEY_ROTATION_DELEGATION">EINVALID_KEY_ROTATION_DELEGATION</a>)
    );

    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(
        &<b>mut</b> borrow_global_mut&lt;<a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(recovery_address).rotation_caps,
        to_recover
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_AddRotationCapabilityAbortsIf">AddRotationCapabilityAbortsIf</a>;
<b>include</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_AddRotationCapabilityEnsures">AddRotationCapabilityEnsures</a>;
</code></pre>




<a name="0x1_RecoveryAddress_AddRotationCapabilityAbortsIf"></a>


<pre><code><b>schema</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_AddRotationCapabilityAbortsIf">AddRotationCapabilityAbortsIf</a> {
    to_recover: KeyRotationCapability;
    recovery_address: address;
    <b>aborts_if</b> !<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(recovery_address) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
    <a name="0x1_RecoveryAddress_to_recover_address$8"></a>
    <b>let</b> to_recover_address = <a href="LibraAccount.md#0x1_LibraAccount_key_rotation_capability_address">LibraAccount::key_rotation_capability_address</a>(to_recover);
    <b>aborts_if</b> !<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(recovery_address) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> !<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(to_recover_address) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(recovery_address) != <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(to_recover_address)
        <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>




<a name="0x1_RecoveryAddress_AddRotationCapabilityEnsures"></a>


<pre><code><b>schema</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_AddRotationCapabilityEnsures">AddRotationCapabilityEnsures</a> {
    to_recover: KeyRotationCapability;
    recovery_address: address;
    <b>ensures</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)[
        len(<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)) - 1] == to_recover;
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



<a name="@Module_specifications_1"></a>

### Module specifications


Returns true if <code>addr</code> is a recovery address.


<a name="0x1_RecoveryAddress_spec_is_recovery_address"></a>


<pre><code><b>define</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(addr: address): bool
{
    <b>exists</b>&lt;<a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(addr)
}
</code></pre>


Returns all the <code>KeyRotationCapability</code>s held at <code>recovery_address</code>.


<a name="0x1_RecoveryAddress_spec_get_rotation_caps"></a>


<pre><code><b>define</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address: address):
    vector&lt;<a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>&gt;
{
    <b>global</b>&lt;<a href="RecoveryAddress.md#0x1_RecoveryAddress">RecoveryAddress</a>&gt;(recovery_address).rotation_caps
}
</code></pre>


Returns true if <code>recovery_address</code> holds the
<code>KeyRotationCapability</code> for <code>addr</code>.


<a name="0x1_RecoveryAddress_spec_holds_key_rotation_cap_for"></a>


<pre><code><b>define</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_holds_key_rotation_cap_for">spec_holds_key_rotation_cap_for</a>(
    recovery_address: address,
    addr: address): bool
{
    // BUG: the commented out version will <b>break</b> the postconditions.
    // <b>exists</b> i in 0..len(<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)):
    //     <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)[i].account_address == addr
    <b>exists</b> i: u64
        <b>where</b> 0 &lt;= i && i &lt; len(<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)):
            <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(recovery_address)[i].account_address == addr
}
</code></pre>



<a name="@RecoveryAddress_has_its_own_KeyRotationCapability_2"></a>

#### RecoveryAddress has its own KeyRotationCapability



<pre><code><b>invariant</b> [<b>global</b>, isolated]
    <b>forall</b> addr: address <b>where</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(addr):
        len(<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(addr)) &gt; 0 &&
        <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_get_rotation_caps">spec_get_rotation_caps</a>(addr)[0].account_address == addr;
</code></pre>



<a name="@RecoveryAddress_resource_stays_3"></a>

#### RecoveryAddress resource stays



<pre><code><b>invariant</b> <b>update</b> [<b>global</b>]
   <b>forall</b> addr: address:
       <b>old</b>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(addr)) ==&gt; <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(addr);
</code></pre>



<a name="@RecoveryAddress_remains_same_4"></a>

#### RecoveryAddress remains same



<pre><code><b>invariant</b> <b>update</b> [<b>global</b>]
    <b>forall</b> recovery_addr: address, to_recovery_addr: address
    <b>where</b> <b>old</b>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(recovery_addr)):
        <b>old</b>(<a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_holds_key_rotation_cap_for">spec_holds_key_rotation_cap_for</a>(recovery_addr, to_recovery_addr))
        ==&gt; <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_holds_key_rotation_cap_for">spec_holds_key_rotation_cap_for</a>(recovery_addr, to_recovery_addr);
</code></pre>



<a name="@Only_VASPs_can_be_RecoveryAddress_5"></a>

#### Only VASPs can be RecoveryAddress



<pre><code><b>invariant</b> [<b>global</b>, isolated]
    <b>forall</b> recovery_addr: address <b>where</b> <a href="RecoveryAddress.md#0x1_RecoveryAddress_spec_is_recovery_address">spec_is_recovery_address</a>(recovery_addr):
        <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(recovery_addr);
</code></pre>
