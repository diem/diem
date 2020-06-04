
<a name="0x0_Association"></a>

# Module `0x0::Association`

### Table of Contents

-  [Struct `Root`](#0x0_Association_Root)
-  [Struct `PrivilegedCapability`](#0x0_Association_PrivilegedCapability)
-  [Struct `Association`](#0x0_Association_Association)
-  [Function `initialize`](#0x0_Association_initialize)
-  [Function `grant_privilege`](#0x0_Association_grant_privilege)
-  [Function `grant_association_address`](#0x0_Association_grant_association_address)
-  [Function `has_privilege`](#0x0_Association_has_privilege)
-  [Function `remove_privilege`](#0x0_Association_remove_privilege)
-  [Function `assert_is_association`](#0x0_Association_assert_is_association)
-  [Function `assert_is_root`](#0x0_Association_assert_is_root)
-  [Function `addr_is_association`](#0x0_Association_addr_is_association)
-  [Function `assert_account_is_blessed`](#0x0_Association_assert_account_is_blessed)
-  [Function `treasury_compliance_account`](#0x0_Association_treasury_compliance_account)
-  [Function `root_address`](#0x0_Association_root_address)
-  [Function `assert_addr_is_association`](#0x0_Association_assert_addr_is_association)
-  [Specification](#0x0_Association_Specification)
    -  [Module Specification](#0x0_Association_@Module_Specification)
        -  [Management of Root marker](#0x0_Association_@Management_of_Root_marker)
        -  [Privilege granting](#0x0_Association_@Privilege_granting)
        -  [Privilege Removal](#0x0_Association_@Privilege_Removal)
        -  [Management of Association Privilege](#0x0_Association_@Management_of_Association_Privilege)
    -  [Function `remove_privilege`](#0x0_Association_Specification_remove_privilege)
    -  [Function `assert_is_association`](#0x0_Association_Specification_assert_is_association)
    -  [Function `assert_is_root`](#0x0_Association_Specification_assert_is_root)
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
1005 -> CANT_REMOVE_ROOT_PRIVILEGE
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

<a name="0x0_Association_Association"></a>

## Struct `Association`

A type tag to mark that this account is an association account.
It cannot be used for more specific/privileged operations.
The presence of an instance of Association::Association at and address
means that the address is an association address.


<pre><code><b>struct</b> <a href="#0x0_Association">Association</a>
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

Initialization is called in genesis. It publishes the
<code><a href="#0x0_Association_Root">Root</a></code> resource under
<code>association</code>
and marks it as an Association account by publishing a
<code><a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;<a href="#0x0_Association">Association</a>&gt;</code> resource.
Aborts if the address of
<code>association</code> is not
<code>root_address</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_initialize">initialize</a>(association: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_initialize">initialize</a>(association: &signer) {
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(association) == <a href="#0x0_Association_root_address">root_address</a>(), 1000);
    move_to(association, <a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;<a href="#0x0_Association">Association</a>&gt;{ });
    move_to(association, <a href="#0x0_Association_Root">Root</a>{ });
}
</code></pre>



</details>

<a name="0x0_Association_grant_privilege"></a>

## Function `grant_privilege`

Certify the privileged capability published under
<code>association</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_grant_privilege">grant_privilege</a>&lt;Privilege&gt;(association: &signer, recipient: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_grant_privilege">grant_privilege</a>&lt;Privilege&gt;(association: &signer, recipient: &signer) {
    <a href="#0x0_Association_assert_is_root">assert_is_root</a>(association);
    move_to(recipient, <a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;{ });
}
</code></pre>



</details>

<a name="0x0_Association_grant_association_address"></a>

## Function `grant_association_address`

Grant the association privilege to
<code>association</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_grant_association_address">grant_association_address</a>(association: &signer, recipient: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_grant_association_address">grant_association_address</a>(association: &signer, recipient: &signer) {
    <a href="#0x0_Association_grant_privilege">grant_privilege</a>&lt;<a href="#0x0_Association">Association</a>&gt;(association, recipient)
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
    // TODO: make genesis work with this check enabled
    //<a href="#0x0_Association_addr_is_association">addr_is_association</a>(addr) &&
    exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_Association_remove_privilege"></a>

## Function `remove_privilege`

Remove the
<code>Privilege</code> from the address at
<code>addr</code>. The
<code>sender</code> must be the root association
account.
Aborts if
<code>addr</code> is the address of the root account


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_remove_privilege">remove_privilege</a>&lt;Privilege&gt;(association: &signer, addr: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_remove_privilege">remove_privilege</a>&lt;Privilege&gt;(association: &signer, addr: address)
<b>acquires</b> <a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a> {
    <a href="#0x0_Association_assert_is_root">assert_is_root</a>(association);
    // root should not be able <b>to</b> remove its own privileges
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(association) != addr, 1005);
    Transaction::assert(exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;&gt;(addr), 1004);
    <a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;{ } = move_from&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;Privilege&gt;&gt;(addr);
}
</code></pre>



</details>

<a name="0x0_Association_assert_is_association"></a>

## Function `assert_is_association`

Assert that the sender is an association account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_is_association">assert_is_association</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_is_association">assert_is_association</a>(account: &signer) {
    <a href="#0x0_Association_assert_addr_is_association">assert_addr_is_association</a>(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account))
}
</code></pre>



</details>

<a name="0x0_Association_assert_is_root"></a>

## Function `assert_is_root`

Assert that the sender is the root association account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_is_root">assert_is_root</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_is_root">assert_is_root</a>(account: &signer) {
    Transaction::assert(exists&lt;<a href="#0x0_Association_Root">Root</a>&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account)), 1001);
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
    exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;<a href="#0x0_Association">Association</a>&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x0_Association_assert_account_is_blessed"></a>

## Function `assert_account_is_blessed`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_account_is_blessed">assert_account_is_blessed</a>(sender_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_account_is_blessed">assert_account_is_blessed</a>(sender_account: &signer) {
    // Verify that the sender is treasury compliant account
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(sender_account) == <a href="#0x0_Association_treasury_compliance_account">treasury_compliance_account</a>(), 0)
}
</code></pre>



</details>

<a name="0x0_Association_treasury_compliance_account"></a>

## Function `treasury_compliance_account`



<pre><code><b>fun</b> <a href="#0x0_Association_treasury_compliance_account">treasury_compliance_account</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_Association_treasury_compliance_account">treasury_compliance_account</a>(): address {
    0xB1E55ED
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

**Caveat:** These specifications are preliminary.
This is one of the first real module libraries with global
specifications, and everything is evolving.

Verify all functions in this module, including private ones.


<pre><code>pragma verify = <b>true</b>;
</code></pre>


The following helper functions do the same things as Move functions.
I have prefaced them with "spec_" to emphasize that they're different,
which might matter if the Move functions are updated.  I'm not sure
this is a good convention.  The long-term solution would be to allow
the use of side-effect-free Move functions in specifications, directly.
Returns true iff
<code>initialize</code> has been run.


<a name="0x0_Association_spec_is_initialized"></a>


<pre><code><b>define</b> <a href="#0x0_Association_spec_is_initialized">spec_is_initialized</a>(): bool { exists&lt;<a href="#0x0_Association_Root">Root</a>&gt;(<a href="#0x0_Association_spec_root_address">spec_root_address</a>()) }
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
    exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;<a href="#0x0_Association">Association</a>&gt;&gt;(addr)
}
</code></pre>



<a name="0x0_Association_@Management_of_Root_marker"></a>

#### Management of Root marker

The root_address is marked by a
<code><a href="#0x0_Association_Root">Root</a></code> object that is stored only at the root address.

Defines an abbreviation for an invariant, so that it can be repeated
in a schema and as a post-condition to
<code>initialize</code>.

**Informally:** Only the root address has a Root resource.


<a name="0x0_Association_only_root_addr_has_root_privilege"></a>


<pre><code><b>define</b> <a href="#0x0_Association_only_root_addr_has_root_privilege">only_root_addr_has_root_privilege</a>(): bool {
    all(domain&lt;address&gt;(), |addr| exists&lt;<a href="#0x0_Association_Root">Root</a>&gt;(addr) ==&gt; addr == <a href="#0x0_Association_spec_root_address">spec_root_address</a>())
}
</code></pre>




<a name="0x0_Association_OnlyRootAddressHasRootPrivilege"></a>

This is the base case of the induction, before initialization.

**Informally:** No address has Root{} stored at it before initialization.

**BUG:** If you change
<code>addr1</code> to
<code>addr</code> below, we get a 'more than one declaration
of variable error from Boogie, which should not happen with a lambda variable.
I have not been able to figure out what is going on. Is it a Boogie problem?


<pre><code><b>schema</b> <a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a> {
    <b>invariant</b> <b>module</b> !<a href="#0x0_Association_spec_is_initialized">spec_is_initialized</a>() ==&gt; all(domain&lt;address&gt;(), |addr1| !exists&lt;<a href="#0x0_Association_Root">Root</a>&gt;(addr1));
}
</code></pre>


Induction hypothesis for invariant, after initialization


<pre><code><b>schema</b> <a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a> {
    <b>invariant</b> <b>module</b> <a href="#0x0_Association_spec_is_initialized">spec_is_initialized</a>() ==&gt; <a href="#0x0_Association_only_root_addr_has_root_privilege">only_root_addr_has_root_privilege</a>();
}
</code></pre>



Apply
<code><a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a></code> to all functions.


<pre><code><b>apply</b> <a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a> <b>to</b> *&lt;Privilege&gt;, *;
</code></pre>




<a name="0x0_Association_@Privilege_granting"></a>

#### Privilege granting

> TODO: We want to say: "There is some way to have an association address that is
> not root." It's ok to provide a specific way to do it.

Taken together, the following properties show that the only way to
remove PrivilegedCapability is to have the root_address invoke remove_privilege.

<a name="0x0_Association_@Privilege_Removal"></a>

#### Privilege Removal

Only
<code><a href="#0x0_Association_remove_privilege">Self::remove_privilege</a></code> can remove privileges.

**Informally:** if addr1 had a PrivilegedCapability (of any type),
it continues to have it.


<a name="0x0_Association_OnlyRemoveCanRemovePrivileges"></a>


<pre><code><b>schema</b> <a href="#0x0_Association_OnlyRemoveCanRemovePrivileges">OnlyRemoveCanRemovePrivileges</a> {
    <b>ensures</b> all(domain&lt;type&gt;(),
                |ty| all(domain&lt;address&gt;(),
                         |addr1| <b>old</b>(exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;ty&gt;&gt;(addr1))
                                    ==&gt; exists&lt;<a href="#0x0_Association_PrivilegedCapability">PrivilegedCapability</a>&lt;ty&gt;&gt;(addr1)));
}
</code></pre>



Show that every function except remove_privilege preserves privileges


<pre><code><b>apply</b> <a href="#0x0_Association_OnlyRemoveCanRemovePrivileges">OnlyRemoveCanRemovePrivileges</a> <b>to</b> *&lt;Privilege&gt;, * <b>except</b> remove_privilege;
</code></pre>




<a name="0x0_Association_@Management_of_Association_Privilege"></a>

#### Management of Association Privilege

Every
<code><a href="#0x0_Association_Root">Root</a></code> address is also an association address.
The prover found a violation because root could remove association privilege from
itself.  I added assertion 1005 above to prevent that, and this verifies.

> **Note:** There is just one root address, so I think it would have been clearer to write
"invariant spec_addr_is_association(spec_root_address(sender()))"


<a name="0x0_Association_RootAddressIsAssociationAddress"></a>


<pre><code><b>schema</b> <a href="#0x0_Association_RootAddressIsAssociationAddress">RootAddressIsAssociationAddress</a> {
    <b>invariant</b> <b>module</b>
        all(domain&lt;address&gt;(),
            |addr1| exists&lt;<a href="#0x0_Association_Root">Root</a>&gt;(addr1) ==&gt; <a href="#0x0_Association_spec_addr_is_association">spec_addr_is_association</a>(addr1));
}
</code></pre>


> **Note:** Why doesn't this include initialize, root_address()?
The prover reports a violation of this property:
Root can remove its own association privilege, by calling
remove_privilege<Association>(root_address()).


<pre><code><b>apply</b> <a href="#0x0_Association_RootAddressIsAssociationAddress">RootAddressIsAssociationAddress</a> <b>to</b> *&lt;Privilege&gt;, *;
</code></pre>


> TODO: add properties that you can't do things without the right privileges.

> TODO: add termination requirements.


<a name="0x0_Association_Specification_remove_privilege"></a>

### Function `remove_privilege`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_remove_privilege">remove_privilege</a>&lt;Privilege&gt;(association: &signer, addr: address)
</code></pre>




<pre><code><b>ensures</b> <a href="Signer.md#0x0_Signer_get_address">Signer::get_address</a>(association) == <a href="#0x0_Association_spec_root_address">spec_root_address</a>();
</code></pre>



<a name="0x0_Association_Specification_assert_is_association"></a>

### Function `assert_is_association`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_is_association">assert_is_association</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x0_Association_spec_addr_is_association">spec_addr_is_association</a>(<a href="Signer.md#0x0_Signer_get_address">Signer::get_address</a>(account));
<b>ensures</b> <a href="#0x0_Association_spec_addr_is_association">spec_addr_is_association</a>(<a href="Signer.md#0x0_Signer_get_address">Signer::get_address</a>(account));
</code></pre>



<a name="0x0_Association_Specification_assert_is_root"></a>

### Function `assert_is_root`


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_Association_assert_is_root">assert_is_root</a>(account: &signer)
</code></pre>


This post-condition to
<code><a href="#0x0_Association_assert_is_root">Self::assert_is_root</a></code> is a sanity check that
the
<code><a href="#0x0_Association_Root">Root</a></code> invariant really works. It needs the invariant
<code><a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a></code>, because
<code>assert_is_root</code> does not
directly check that the
<code><a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(account) == <a href="#0x0_Association_root_address">root_address</a>()</code>. Instead, it aborts
if
<code>account</code> does not have root privilege, and only the root_address has
<code><a href="#0x0_Association_Root">Root</a></code>.

> TODO: There is a style question about whether this should just check for presence of
a Root privilege. I guess it's moot so long as
<code><a href="#0x0_Association_OnlyRootAddressHasRootPrivilege">OnlyRootAddressHasRootPrivilege</a></code> holds.


<pre><code><b>ensures</b> <a href="Signer.md#0x0_Signer_get_address">Signer::get_address</a>(account) == <a href="#0x0_Association_spec_root_address">spec_root_address</a>();
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
