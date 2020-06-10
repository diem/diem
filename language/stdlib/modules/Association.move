address 0x1 {

/// Implements logic for registering addresses as association addresses, and
/// determining if the sending account is an association account.

/// Errors:
///
/// ```
/// 1000 -> INVALID_GENESIS_ADDR
/// 1001 -> INSUFFICIENT_PRIVILEGES
/// 1002 -> NOT_AN_ASSOCIATION_ACCOUNT
/// 1003 -> ACCOUNT_DOES_NOT_HAVE_PRIVILEGE
/// 1004 -> ACCOUNT_DOES_NOT_HAVE_PRIVILEGE_RESOURCE
/// 1005 -> CANT_REMOVE_ROOT_PRIVILEGE
/// ```

module Association {
    use 0x1::CoreAddresses;
    use 0x1::Signer;

    /// The root account privilege. This is created at genesis and has
    /// special privileges (e.g. removing an account as an association
    /// account). It cannot be removed.
    resource struct Root { }

    /// There are certain association capabilities that are more
    /// privileged than other association operations. This resource with the
    /// type representing that privilege is published under the privileged
    /// account.
    resource struct PrivilegedCapability<Privilege> { }

    /// A type tag to mark that this account is an association account.
    /// It cannot be used for more specific/privileged operations.

    /// The presence of an instance of Association::Association at and address
    /// means that the address is an association address.
    struct Association { }

    /// The presence of an instance of an `Association::PublishModule`
    /// privilege at an address means that that address can publish code to
    /// the chain.
    struct PublishModule { }

    /// Initialization is called in genesis. It publishes the `Root` resource under `association`
    /// and marks it as an Association account by publishing a `PrivilegedCapability<Association>` resource.
    /// Aborts if the address of `association` is not `root_address`
    public fun initialize(association: &signer) {
        assert(Signer::address_of(association) == root_address(), 1000);
        move_to(association, PrivilegedCapability<Association>{ });
        move_to(association, Root{ });
    }

    /// Certify the privileged capability published under `association`.
    public fun grant_privilege<Privilege>(association: &signer, recipient: &signer) {
        assert_is_root(association);
        move_to(recipient, PrivilegedCapability<Privilege>{ });
    }

    /// Grant the association privilege to `association`
    public fun grant_association_address(association: &signer, recipient: &signer) {
        grant_privilege<Association>(association, recipient)
    }

    /// Return whether the `addr` has the specified `Privilege`.
    public fun has_privilege<Privilege>(addr: address): bool {
        // TODO: make genesis work with this check enabled
        //addr_is_association(addr) &&
        exists<PrivilegedCapability<Privilege>>(addr)
    }

    /// Remove the `Privilege` from the address at `addr`. The `sender` must be the root association
    /// account.
    /// Aborts if `addr` is the address of the root account
    public fun remove_privilege<Privilege>(association: &signer, addr: address)
    acquires PrivilegedCapability {
        assert_is_root(association);
        // root should not be able to remove its own privileges
        assert(Signer::address_of(association) != addr, 1005);
        assert(exists<PrivilegedCapability<Privilege>>(addr), 1004);
        PrivilegedCapability<Privilege>{ } = move_from<PrivilegedCapability<Privilege>>(addr);
    }

    /// Assert that the sender is an association account.
    public fun assert_is_association(account: &signer) {
        assert_addr_is_association(Signer::address_of(account))
    }

    /// Assert that the sender is the root association account.
    public fun assert_is_root(account: &signer) {
        assert(exists<Root>(Signer::address_of(account)), 1001);
    }

    /// Return whether the account at `addr` is an association account.
    public fun addr_is_association(addr: address): bool {
        exists<PrivilegedCapability<Association>>(addr)
    }

    public fun assert_account_is_blessed(sender_account: &signer) {
        // Verify that the sender is treasury compliant account
        assert(Signer::address_of(sender_account) == treasury_compliance_account(), 0)
    }

    public fun treasury_compliance_account(): address {
        CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()
    }

    /// The address at which the root account will be published.
    public fun root_address(): address {
        CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    }

    /// Assert that `addr` is an association account.
    fun assert_addr_is_association(addr: address) {
        assert(addr_is_association(addr), 1002);
    }

    // **************** SPECIFICATIONS ****************

    /// # Module Specification

    /// **Caveat:** These specifications are preliminary.
    /// This is one of the first real module libraries with global
    /// specifications, and everything is evolving.

    spec module {
        /// Verify all functions in this module, including private ones.
        pragma verify = true;

        /// The following helper functions do the same things as Move functions.
        /// I have prefaced them with "spec_" to emphasize that they're different,
        /// which might matter if the Move functions are updated.  I'm not sure
        /// this is a good convention.  The long-term solution would be to allow
        /// the use of side-effect-free Move functions in specifications, directly.

        /// Returns true iff `initialize` has been run.
        define spec_is_initialized(): bool { exists<Root>(spec_root_address()) }

        /// Returns the association root address.  `Self::spec_root_address` needs to be
        /// consistent with the Move function `Self::root_address`.
        define spec_root_address(): address { 0xA550C18 }

        /// Helper which mirrors Move `Self::addr_is_association`.
        define spec_addr_is_association(addr: address): bool {
            exists<PrivilegedCapability<Association>>(addr)
        }
     }

    /// ## Management of Root marker

    /// The root_address is marked by a `Root` object that is stored only at the root address.
    spec module {
        /// Defines an abbreviation for an invariant, so that it can be repeated
        /// in a schema and as a post-condition to `initialize`.
        ///
        /// **Informally:** Only the root address has a Root resource.
        define only_root_addr_has_root_privilege(): bool {
            all(domain<address>(), |addr| exists<Root>(addr) ==> addr == spec_root_address())
        }
    }
    spec schema OnlyRootAddressHasRootPrivilege {
        /// This is the base case of the induction, before initialization.
        ///
        /// **Informally:** No address has Root{} stored at it before initialization.
        ///
        /// **BUG:** If you change `addr1` to `addr` below, we get a 'more than one declaration
        /// of variable error from Boogie, which should not happen with a lambda variable.
        /// I have not been able to figure out what is going on. Is it a Boogie problem?
        invariant module !spec_is_initialized() ==> all(domain<address>(), |addr1| !exists<Root>(addr1));
        /// Induction hypothesis for invariant, after initialization
        invariant module spec_is_initialized() ==> only_root_addr_has_root_privilege();
    }
    spec module {
        /// Apply `OnlyRootAddressHasRootPrivilege` to all functions.
        apply OnlyRootAddressHasRootPrivilege to *<Privilege>, *;
    }

    spec schema InitializationPersists {
        /// For every `is_initialized` predicate, we want to make sure that the
        /// predicate never becomes false again.  Since `is_initialized` simply
        /// checks for the presence of a Root {} value at the root address,
        /// the same property also checks that "root stays root", which is of
        /// interest for the correct functioning of the system.
        ///
        /// *Informally:* Once initialize is run, the module continues to be
        /// initialized, forever.
        ensures old(spec_is_initialized()) ==> spec_is_initialized();
    }
    spec module {
        apply InitializationPersists to *<Privilege>, *;
    }


    /// This post-condition to `Self::assert_is_root` is a sanity check that
    /// the `Root` invariant really works. It needs the invariant
    /// `OnlyRootAddressHasRootPrivilege`, because `assert_is_root` does not
    /// directly check that the `Signer::address_of(account) == root_address()`. Instead, it aborts
    /// if `account` does not have root privilege, and only the root_address has `Root`.
    ///
    /// > TODO: There is a style question about whether this should just check for presence of
    /// a Root privilege. I guess it's moot so long as `OnlyRootAddressHasRootPrivilege` holds.
    spec fun assert_is_root {
        ensures Signer::get_address(account) == spec_root_address();
    }

    // Switch documentation context back to module level.
    spec module {}

    /// ## Privilege granting
    /// > TODO: We want to say: "There is some way to have an association address that is
    /// > not root." It's ok to provide a specific way to do it.

    // Switch documentation context back to module level.
    spec module {}

    /// Taken together, the following properties show that the only way to
    /// remove PrivilegedCapability is to have the root_address invoke remove_privilege.

    /// ## Privilege Removal

    /// Only `Self::remove_privilege` can remove privileges.
    ///
    /// **Informally:** if addr1 had a PrivilegedCapability (of any type),
    /// it continues to have it.
    spec schema OnlyRemoveCanRemovePrivileges {
         ensures all(domain<type>(),
                     |ty| all(domain<address>(),
                              |addr1| old(exists<PrivilegedCapability<ty>>(addr1))
                                         ==> exists<PrivilegedCapability<ty>>(addr1)));
    }
    spec module {
        /// Show that every function except remove_privilege preserves privileges
        apply OnlyRemoveCanRemovePrivileges to *<Privilege>, * except remove_privilege;
    }
    spec fun remove_privilege {
        // Only root can call remove_privilege without aborting.
        ensures Signer::get_address(association) == spec_root_address();
    }

    // Switch documentation context back to module level.
    spec module {}

    /// ## Management of Association Privilege

    /// Every `Root` address is also an association address.
    /// The prover found a violation because root could remove association privilege from
    /// itself.  I added assertion 1005 above to prevent that, and this verifies.
    ///
    /// > **Note:** There is just one root address, so I think it would have been clearer to write
    /// "invariant spec_addr_is_association(spec_root_address(sender()))"
    spec schema RootAddressIsAssociationAddress {
        invariant module
            all(domain<address>(),
                |addr1| exists<Root>(addr1) ==> spec_addr_is_association(addr1));
    }

    /// > **Note:** Why doesn't this include initialize, root_address()?
    /// The prover reports a violation of this property:
    /// Root can remove its own association privilege, by calling
    /// remove_privilege<Association>(root_address()).
    spec module {
        apply RootAddressIsAssociationAddress to *<Privilege>, *;
    }

    spec fun addr_is_association {
        aborts_if false;
        ensures result == spec_addr_is_association(addr);
    }

    spec fun assert_addr_is_association {
        aborts_if !spec_addr_is_association(addr);
        ensures spec_addr_is_association(addr);
    }

    spec fun assert_is_association {
        aborts_if !spec_addr_is_association(Signer::get_address(account));
        ensures spec_addr_is_association(Signer::get_address(account));
    }


    /// > TODO: add properties that you can't do things without the right privileges.
    ///
    /// > TODO: add termination requirements.
    spec module {}

}
}
