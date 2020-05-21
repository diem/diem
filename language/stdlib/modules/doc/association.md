
<a name="0x0_Association"></a>

# Module `0x0::Association`

### Table of Contents

-  [Struct `Root`](#0x0_Association_Root)
-  [Struct `PrivilegedCapability`](#0x0_Association_PrivilegedCapability)
-  [Struct `T`](#0x0_Association_T)
-  [Function `initialize`](#0x0_Association_initialize)
-  [Function `grant_privilege`](#0x0_Association_grant_privilege)
-  [Function `grant_association_address`](#0x0_Association_grant_association_address)
-  [Function `has_privilege`](#0x0_Association_has_privilege)
-  [Function `remove_privilege`](#0x0_Association_remove_privilege)
-  [Function `assert_sender_is_association`](#0x0_Association_assert_sender_is_association)
-  [Function `assert_sender_is_root`](#0x0_Association_assert_sender_is_root)
-  [Function `addr_is_association`](#0x0_Association_addr_is_association)
-  [Function `root_address`](#0x0_Association_root_address)
-  [Function `assert_addr_is_association`](#0x0_Association_assert_addr_is_association)
-  [Specification](#0x0_Association_Specification)
    -  [Module Specification](#0x0_Association_@Module_Specification)
        -  [Management of Root marker](#0x0_Association_@Management_of_Root_marker)
        -  [Privilege Removal](#0x0_Association_@Privilege_Removal)
        -  [Management of Association Privilege](#0x0_Association_@Management_of_Association_Privilege)
    -  [Function `initialize`](#0x0_Association_Specification_initialize)
    -  [Function `remove_privilege`](#0x0_Association_Specification_remove_privilege)
    -  [Function `assert_sender_is_association`](#0x0_Association_Specification_assert_sender_is_association)
    -  [Function `assert_sender_is_root`](#0x0_Association_Specification_assert_sender_is_root)
    -  [Function `addr_is_association`](#0x0_Association_Specification_addr_is_association)
    -  [Function `assert_addr_is_association`](#0x0_Association_Specification_assert_addr_is_association)

Implements logic for registering addresses as association addresses, and
determining if the sending account is an association account.
Errors:

```
1000 -> INVALID_GENESIS_ADDR
1001 -> INSUFFICIENT_PRIVILEGES
1002 -> NOT_AN_ASSOCIATION_ACCOUNT
1003 -> ACCOUNT_DOES_NOT_HAVE_PRIVILEGE
1004 -> ACCOUNT_DOES_NOT_HAVE_PRIVILEGE_RESOURCE
```


<a name="0x0_Association_Root"></a>

## Struct `Root`

The root account privilege. This is created at genesis and has
special privileges (e.g. removing an account as an association
account). It cannot be removed.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Association_Root">Root</a>
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

<a name="0x0_Association_PrivilegedCapability"></a>

## Struct `PrivilegedCapability`

There are certain association capabilities that are more
privileged than other association operations. This resource with the
type representing that privilege is published under the privileged
account.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;
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

<a name="0x0_Association_T"></a>

## Struct `T`

A type tag to mark that this account is an association account.
It cannot be used for more specific/privileged operations.
> DD: The presence of an instance of T at and address
> means that the address is an association address. I suggest giving "T"
> a more meaningful name (e.g., AssociationMember? AssociationPrivileges?)


<pre><code><b>struct</b> <a href="#0x0_Association_T">T</a>
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

<a name="0x0_Association_initialize"></a>

## Function `initialize`

Initialization is called in genesis. It publishes the root resource
under the root_address() address, marks it as a normal
association account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_initialize">initialize</a>(association: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_initialize">initialize</a>(association: &signer) {
    Transaction::assert(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(association) == <a href="#0x0_Association_root_address">root_address</a>(), 1000);
    move_to(association, <a href="#0x0_Association_Root">Root</a>{ });
    move_to(association, <a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;<a href="#0x0_Association_T">T</a>&gt;{  });
}
</code></pre>



</details>

<a name="0x0_Association_grant_privilege"></a>

## Function `grant_privilege`

Certify the privileged capability published under
<code>association</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_grant_privilege">grant_privilege</a>&lt;Privilege&gt;(association: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_grant_privilege">grant_privilege</a>&lt;Privilege&gt;(association: &signer) {
    <a href="#0x0_Association_assert_sender_is_root">assert_sender_is_root</a>();
    move_to(association, <a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;{ });
}
</code></pre>



</details>

<a name="0x0_Association_grant_association_address"></a>

## Function `grant_association_address`

Grant the association privilege to
<code>association</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_grant_association_address">grant_association_address</a>(association: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_grant_association_address">grant_association_address</a>(association: &signer) {
    <a href="#0x0_Association_grant_privilege">grant_privilege</a>&lt;<a href="#0x0_Association_T">T</a>&gt;(association)
}
</code></pre>



</details>

<a name="0x0_Association_has_privilege"></a>

## Function `has_privilege`

Return whether the
<code>addr</code> has the specified
<code>Privilege</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_has_privilege">has_privilege</a>&lt;Privilege&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_has_privilege">has_privilege</a>&lt;Privilege&gt;(addr: address): bool {
    // TODO: figure out what <b>to</b> do with this
    //<a href="#0x0_Association_addr_is_association">addr_is_association</a>(addr) &&
    exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_Association_remove_privilege"></a>

## Function `remove_privilege`

Remove the
<code>Privilege</code> from the address at
<code>addr</code>. The sender must
be the root association account


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_remove_privilege">remove_privilege</a>&lt;Privilege&gt;(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_remove_privilege">remove_privilege</a>&lt;Privilege&gt;(addr: address)
<b>acquires</b> <a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a> {
    <a href="#0x0_Association_assert_sender_is_root">assert_sender_is_root</a>();
    Transaction::assert(exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;&gt;(addr), 1004);
    <a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;{ } = move_from&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;&gt;(addr);
}
</code></pre>



</details>

<a name="0x0_Association_assert_sender_is_association"></a>

## Function `assert_sender_is_association`

Assert that the sender is an association account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_sender_is_association">assert_sender_is_association</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_sender_is_association">assert_sender_is_association</a>() {
    <a href="#0x0_Association_assert_addr_is_association">assert_addr_is_association</a>(Transaction::sender())
}
</code></pre>



</details>

<a name="0x0_Association_assert_sender_is_root"></a>

## Function `assert_sender_is_root`

Assert that the sender is the root association account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_sender_is_root">assert_sender_is_root</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_sender_is_root">assert_sender_is_root</a>() {
    Transaction::assert(exists&lt;<a href="#0x0_Association_Root">Root</a>&gt;(Transaction::sender()), 1001);
}
</code></pre>



</details>

<a name="0x0_Association_addr_is_association"></a>

## Function `addr_is_association`

Return whether the account at
<code>addr</code> is an association account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_addr_is_association">addr_is_association</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_addr_is_association">addr_is_association</a>(addr: address): bool {
    exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;<a href="#0x0_Association_T">T</a>&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_Association_root_address"></a>

## Function `root_address`

The address at which the root account will be published.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_root_address">root_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_root_address">root_address</a>(): address {
    0xA550C18
}
</code></pre>



</details>

<a name="0x0_Association_assert_addr_is_association"></a>

## Function `assert_addr_is_association`

Assert that
<code>addr</code> is an association account.


<pre><code><b>fun</b> <a href="#0x0_Association_assert_addr_is_association">assert_addr_is_association</a>(addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Association_assert_addr_is_association">assert_addr_is_association</a>(addr: address) {
    Transaction::assert(<a href="#0x0_Association_addr_is_association">addr_is_association</a>(addr), 1002);
}
</code></pre>



</details>

<a name="0x0_Association_Specification"></a>

## Specification


<a name="0x0_Association_@Module_Specification"></a>

### Module Specification

> This is preliminary.  This is one of the first real module libraries with global
> specifications, and everything is evolving.

Verify all functions in this module, including private ones.


<pre><code>pragma verify = <b>true</b>;
</code></pre>


Returns the association root address.
<code><a href="#0x0_Association_spec_root_address">Self::spec_root_address</a></code> needs to be
consistent with the Move function
<code><a href="#0x0_Association_root_address">Self::root_address</a></code>.


<a name="0x0_Association_spec_root_address"></a>


<pre><code><b>define</b> <a href="#0x0_Association_spec_root_address">spec_root_address</a>(): address { 0xA550C18 }
</code></pre>


Helper which mirrors Move
<code><a href="#0x0_Association_addr_is_association">Self::addr_is_association</a></code>.


<a name="0x0_Association_spec_addr_is_association"></a>


<pre><code><b>define</b> <a href="#0x0_Association_spec_addr_is_association">spec_addr_is_association</a>(addr: address): bool {
    exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;<a href="#0x0_Association_T">T</a>&gt;&gt;(addr)
}
</code></pre>


> TODO: With grant/remove etc., separately specify that only root may grant or remove privileges

<a name="0x0_Association_@Management_of_Root_marker"></a>

#### Management of Root marker

The root_address is marked by a
<code><a href="#0x0_Association_Root">Root</a></code> object that is stored at that place only.

Defines an abbreviation for an invariant, so that it can be repeated
in a schema and as a post-condition to
<code>initialize</code>. This postulates that
only the root address has a Root resource.


<a name="0x0_Association_only_root_addr_has_root_privilege"></a>


<pre><code><b>define</b> <a href="#0x0_Association_only_root_addr_has_root_privilege">only_root_addr_has_root_privilege</a>(): bool {
    all(domain&lt;address&gt;(), |addr| (exists&lt;<a href="#0x0_Association_Root">Root</a>&gt;(addr)) ==&gt; addr == <a href="#0x0_Association_spec_root_address">spec_root_address</a>())
}
</code></pre>




<a name="0x0_Association_OnlyRootAddressHasRootPrivilege"></a>


<pre><code><b>schema</b> <a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a> {
    <b>invariant</b> <a href="#0x0_Association_only_root_addr_has_root_privilege">only_root_addr_has_root_privilege</a>();
}
</code></pre>



Apply
<code><a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a></code> to all functions except
<code><a href="#0x0_Association_initialize">Self::initialize</a></code> and functions that
<code>initialize</code> calls
before the invariant is established.
> Note: All the called functions *obviously* cannot affect the invariant.
> TODO: Try to find a better approach to this that does not require excepting functions.
> Note: this needs to be applied to *<Privilege>, otherwise it gets a false error on
> the assert_addr_is_root in grant_privilege<Privilege>


<pre><code><b>apply</b> <a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a> <b>to</b> *&lt;Privilege&gt;, *
    <b>except</b> initialize, root_address, has_privilege, addr_is_association,
    assert_addr_is_association, assert_sender_is_association;
</code></pre>




<a name="0x0_Association_@Privilege_Removal"></a>

#### Privilege Removal

Only
<code><a href="#0x0_Association_remove_privilege">Self::remove_privilege</a></code> can remove privileges


<a name="0x0_Association_OnlyRemoveCanRemovePrivileges"></a>


<pre><code><b>schema</b> <a href="#0x0_Association_OnlyRemoveCanRemovePrivileges">OnlyRemoveCanRemovePrivileges</a>&lt;Privilege&gt; {
    <b>ensures</b> any(domain&lt;address&gt;(), |addr1|
        <b>old</b>(exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;&gt;(addr1)) && !exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;&gt;(addr1)
            ==&gt; sender() == <a href="#0x0_Association_spec_root_address">spec_root_address</a>()
    );
}
</code></pre>



> *Feature request*: We need to be able to apply this to functions that don't have
> a type parameter (or a type parameter for something different)? They could violate
> the property by removing a specific privilege. We need the effect of universal
> quantification over all possible instantiations of Privilege.


<pre><code><b>apply</b> <a href="#0x0_Association_OnlyRemoveCanRemovePrivileges">OnlyRemoveCanRemovePrivileges</a>&lt;Privilege&gt; <b>to</b> *&lt;Privilege&gt;;
</code></pre>




<a name="0x0_Association_@Management_of_Association_Privilege"></a>

#### Management of Association Privilege

Every
<code><a href="#0x0_Association_Root">Root</a></code> address is also an association address.
> Note: There is just one root address, so I think it would have been clearer to write
> "invariant spec_addr_is_association(spec_root_address(sender()))"


<a name="0x0_Association_RootAddressIsAssociationAddress"></a>


<pre><code><b>schema</b> <a href="#0x0_Association_RootAddressIsAssociationAddress">RootAddressIsAssociationAddress</a> {
    <b>invariant</b> all(domain&lt;address&gt;(), |a| exists&lt;<a href="#0x0_Association_Root">Root</a>&gt;(a) ==&gt; <a href="#0x0_Association_spec_addr_is_association">spec_addr_is_association</a>(a));
}
</code></pre>


Except functions called from initialize before invariant is established.
> Note: Why doesn't this include initialize, root_address()?
> The prover reports a violation of this property:
> Root can remove its own association privilege, by calling
> remove_privilege<T>(root_address()).
> I have therefore commented out the "apply"
```
apply RootAddressIsAssociationAddress to *<Privilege>, *
except has_privilege, addr_is_association, assert_addr_is_association, assert_sender_is_association;
```

> TODO: add properties that you can't do things without the right privileges.
> TODO: add termination requirements.


<a name="0x0_Association_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_initialize">initialize</a>(association: &signer)
</code></pre>




<code><a href="#0x0_Association_initialize">Self::initialize</a></code> establishes the invariant, so it's a special case.
Before initialize, no addresses have a
<code><a href="#0x0_Association_Root">Root</a></code> resource.
Afterwards, only
<code><a href="#0x0_Association_Root">Root</a></code> has the root resource.


<pre><code><b>requires</b> all(domain&lt;address&gt;(), |addr| !exists&lt;<a href="#0x0_Association_Root">Root</a>&gt;(addr));
<b>ensures</b> <a href="#0x0_Association_only_root_addr_has_root_privilege">only_root_addr_has_root_privilege</a>();
</code></pre>



<a name="0x0_Association_Specification_remove_privilege"></a>

### Function `remove_privilege`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_remove_privilege">remove_privilege</a>&lt;Privilege&gt;(addr: address)
</code></pre>




<pre><code><b>ensures</b> sender() == <a href="#0x0_Association_spec_root_address">spec_root_address</a>();
</code></pre>



<a name="0x0_Association_Specification_assert_sender_is_association"></a>

### Function `assert_sender_is_association`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_sender_is_association">assert_sender_is_association</a>()
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x0_Association_spec_addr_is_association">spec_addr_is_association</a>(sender());
<b>ensures</b> <a href="#0x0_Association_spec_addr_is_association">spec_addr_is_association</a>(sender());
</code></pre>



<a name="0x0_Association_Specification_assert_sender_is_root"></a>

### Function `assert_sender_is_root`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_sender_is_root">assert_sender_is_root</a>()
</code></pre>


This post-condition to
<code><a href="#0x0_Association_assert_sender_is_root">Self::assert_sender_is_root</a></code> is a sanity check that
the
<code><a href="#0x0_Association_Root">Root</a></code> invariant really works. It needs the invariant
<code><a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a></code>, because
<code>assert_sender_is_root</code> does not
directly check that the
<code>sender == <a href="#0x0_Association_root_address">root_address</a>()</code>. Instead, it aborts if
sender has no root privilege, and only the root_address has
<code><a href="#0x0_Association_Root">Root</a></code>.
> TODO: There is a style question about whether this should just check for presence of
a Root privilege. I guess it's moot so long as
<code><a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a></code> holds.


<pre><code><b>ensures</b> sender() == <a href="#0x0_Association_spec_root_address">spec_root_address</a>();
</code></pre>



<a name="0x0_Association_Specification_addr_is_association"></a>

### Function `addr_is_association`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_addr_is_association">addr_is_association</a>(addr: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="#0x0_Association_spec_addr_is_association">spec_addr_is_association</a>(addr);
</code></pre>



<a name="0x0_Association_Specification_assert_addr_is_association"></a>

### Function `assert_addr_is_association`


<pre><code><b>fun</b> <a href="#0x0_Association_assert_addr_is_association">assert_addr_is_association</a>(addr: address)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x0_Association_spec_addr_is_association">spec_addr_is_association</a>(addr);
<b>ensures</b> <a href="#0x0_Association_spec_addr_is_association">spec_addr_is_association</a>(addr);
</code></pre>
