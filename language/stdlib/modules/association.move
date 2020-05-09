address 0x0 {

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
/// ```

module Association {
    use 0x0::Signer;
    use 0x0::Transaction;

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

    /// > DD: The presence of an instance of T at and address
    /// > means that the address is an association address. I suggest giving "T"
    /// > a more meaningful name (e.g., AssociationMember? AssociationPrivileges?)
    struct T { }

    /// Initialization is called in genesis. It publishes the root resource
    /// under the root_address() address, marks it as a normal
    /// association account.
    public fun initialize(association: &signer) {
        Transaction::assert(Signer::address_of(association) == root_address(), 1000);
        move_to(association, Root{ });
        move_to(association, PrivilegedCapability<T>{  });
    }

    /// Certify the privileged capability published under `association`.
    public fun grant_privilege<Privilege>(association: &signer) {
        assert_sender_is_root();
        move_to(association, PrivilegedCapability<Privilege>{ });
    }

    /// Grant the association privilege to `association`
    public fun grant_association_address(association: &signer) {
        grant_privilege<T>(association)
    }

    /// Return whether the `addr` has the specified `Privilege`.
    public fun has_privilege<Privilege>(addr: address): bool {
        // TODO: figure out what to do with this
        //addr_is_association(addr) &&
        exists<PrivilegedCapability<Privilege>>(addr)
    }

    /// Remove the `Privilege` from the address at `addr`. The sender must
    /// be the root association account
    // DD: Perhaps should not allow root to remove itself from Association?
    public fun remove_privilege<Privilege>(addr: address)
    acquires PrivilegedCapability {
        assert_sender_is_root();
        Transaction::assert(exists<PrivilegedCapability<Privilege>>(addr), 1004);
        PrivilegedCapability<Privilege>{ } = move_from<PrivilegedCapability<Privilege>>(addr);
    }

    /// Assert that the sender is an association account.
    public fun assert_sender_is_association() {
        assert_addr_is_association(Transaction::sender())
    }

    /// Assert that the sender is the root association account.
    public fun assert_sender_is_root() {
        Transaction::assert(exists<Root>(Transaction::sender()), 1001);
    }

    /// Return whether the account at `addr` is an association account.
    public fun addr_is_association(addr: address): bool {
        exists<PrivilegedCapability<T>>(addr)
    }

    /// The address at which the root account will be published.
    public fun root_address(): address {
        0xA550C18
    }

    /// Assert that `addr` is an association account.
    fun assert_addr_is_association(addr: address) {
        Transaction::assert(addr_is_association(addr), 1002);
    }

    // **************** SPECIFICATIONS ****************
    // *Note:* I need to work on the doc comments more.

    /// # Module Specification

    /// > This is preliminary.  This is one of the first real module libraries with global
    /// > specifications, and everything is evolving.

    spec module {
        /// Verify all functions in this module, including private ones.
        pragma verify = true;

        /// Returns the association root address.  `Self::spec_root_address` needs to be
        /// consistent with the Move function `Self::root_address`.
        define spec_root_address(): address { 0xA550C18 }

        /// Helper which mirrors Move `Self::addr_is_association`.
        define spec_addr_is_association(addr: address): bool {
            exists<PrivilegedCapability<T>>(addr)
        }
     }

    /// > TODO: With grant/remove etc., separately specify that only root may grant or remove privileges

    /// ## Management of Root marker

    /// The root_address is marked by a `Root` object that is stored at that place only.
    spec module {
        /// Defines an abbreviation for an invariant, so that it can be repeated
        /// in a schema and as a post-condition to `initialize`. This postulates that
        /// only the root address has a Root resource.
        define only_root_addr_has_root_privilege(): bool {
            all(domain<address>(), |addr| exists<Root>(addr) ==> (addr == spec_root_address()))
        }
    }
    spec schema OnlyRootAddressHasRootPrivilege {
        invariant only_root_addr_has_root_privilege();
    }
    spec module {
        /// Apply `OnlyRootAddressHasRootPrivilege` to all functions except
        /// `Self::initialize` and functions that `initialize` calls
        /// before the invariant is established.
        /// > Note: All the called functions *obviously* cannot affect the invariant.
        /// > TODO: Try to find a better approach to this that does not require excepting functions.
        /// > Note: this needs to be applied to *<Privilege>, otherwise it gets a false error on
        /// > the assert_addr_is_root in grant_privilege<Privilege>
        apply OnlyRootAddressHasRootPrivilege to *<Privilege>, *
            except initialize, root_address, has_privilege, addr_is_association,
            assert_addr_is_association, assert_sender_is_association;
    }
    spec fun initialize {
        /// `Self::initialize` establishes the invariant, so it's a special case.
        /// Before initialize, no addresses have a `Root` resource.
        /// Afterwards, only `Root` has the root resource.
        requires all(domain<address>(), |addr| !exists<Root>(addr));
        ensures only_root_addr_has_root_privilege();
    }

    /// This post-condition to `Self::assert_sender_is_root` is a sanity check that
    /// the `Root` invariant really works. It needs the invariant
    /// `OnlyRootAddressHasRootPrivilege`, because `assert_sender_is_root` does not
    /// directly check that the `sender == root_address()`. Instead, it aborts if
    /// sender has root privilege, and only the root_address has `Root`.
    /// > TODO: There is a style question about whether this should just check for presence of
    /// a Root privilege. I guess it's moot so long as `OnlyRootAddressHasRootPrivilege` holds.
    spec fun assert_sender_is_root {
        ensures sender() == spec_root_address();
    }

    // Switch documentation context back to module level.
    spec module {}

    /// ## Privilege Removal

    /// Only `Self::remove_privilege` can remove privileges
    spec schema OnlyRemoveCanRemovePrivileges<Privilege> {
         ensures any(domain<address>(), |addr1|
             old(exists<PrivilegedCapability<Privilege>>(addr1)) && !exists<PrivilegedCapability<Privilege>>(addr1)
                 ==> sender() == spec_root_address()
         );
    }
    spec module {
        /// > *Feature request*: We need to be able to apply this to functions that don't have
        /// > a type parameter (or a type parameter for something different)? They could violate
        /// > the property by removing a specific privilege. We need the effect of universal
        /// > quantification over all possible instantiations of Privilege.
        apply OnlyRemoveCanRemovePrivileges<Privilege> to *<Privilege>;
    }
    spec fun remove_privilege {
        // Only root can call remove_privilege without aborting.
        ensures sender() == spec_root_address();
    }

    // Switch documentation context back to module level.
    spec module {}

    /// ## Management of Association Privilege

    /// Every `Root` address is also an association address.
    /// > Note: There is just one root address, so I think it would have been clearer to write
    /// > "invariant spec_addr_is_association(spec_root_address(sender()))"
    spec schema RootAddressIsAssociationAddress {
        invariant all(domain<address>(), |a| exists<Root>(a) ==> spec_addr_is_association(a));
    }

    /// Except functions called from initialize before invariant is established.
    /// > Note: Why doesn't this include initialize, root_address()?
    /// > The prover reports a violation of this property:
    /// > Root can remove its own association privilege, by calling
    /// > remove_privilege<T>(root_address()).
    /// > I have therefore commented out the "apply"
    /// ```
    ///    apply RootAddressIsAssociationAddress to *<Privilege>, *
    ///         except has_privilege, addr_is_association, assert_addr_is_association, assert_sender_is_association;
    /// ```
    spec module {
    }

    spec fun addr_is_association {
        aborts_if false;
        ensures result == spec_addr_is_association(addr);
    }

    spec fun assert_addr_is_association {
        aborts_if !spec_addr_is_association(addr);
        ensures spec_addr_is_association(addr);
    }

    spec fun assert_sender_is_association {
        aborts_if !spec_addr_is_association(sender());
        ensures spec_addr_is_association(sender());
    }


    /// > TODO: add properties that you can't do things without the right privileges.
    /// > TODO: add termination requirements.
    spec module {}

}
}
