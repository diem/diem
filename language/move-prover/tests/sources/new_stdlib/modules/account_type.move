// Key concept:
// Roots of authority are owned by the GrantingCapability
// These are the root of any child-account tree. The child accounts specify
// their root by setting the `root_address` field in their
// to-be-certified AccountType that they publish.
// `TransitionCapability`s hold the `root_address` field--this is the root
// authority they are acting for--they can then create child accounts of
// the given account type.
address 0x0:

module AccountType {
    use 0x0::Association;
    use 0x0::Transaction;
    use 0x0::Empty;

    // A resource to represent the account type. Multiple of these (of
    // different `AccountType`'s) can be published under the same account.
    // But, at most one of them can have the `is_certified` flag set to
    // true.  Further, an account must always have at least one of these
    // certified. An account is not considered of `AccountType` unless this
    // flag is set.  Cf. `is_a` in this module.
    resource struct T<AccountType: copyable> {
        is_certified: bool,
        // Holds metadata pertinent to the `AccountType`.
        account_metadata: AccountType,
        // If this is a child account, the root/parent address.
        root_address: address,
    }

    // An account with this resource and the `is_certified` field set has
    // the ability to transition account A from an `Empty` account type to
    // the `To` account type. However, it can only do so as long as the
    // `To` account (yet uncertified) account type that is already
    // published under A has a `root_address` field that matches with the
    // `root_address` field stored in this capability. Cf. `transition` in
    // this module. Identity transitions are disallowed since multiple
    // resources of the same type are not allowed to be published under the
    // same address.
    resource struct TransitionCapability<To: copyable> {
        is_certified: bool,
        // This points to the address under which the GrantingCapability
        // is stored that granted this TransitionCapability
        root_address: address,
    }

    // An account with this resource published under it (and certified) can certify
    // `TransitionCapability<To>`'s for other accounts as long as
    // their `root_address` points at the address that this resource is
    // stored under. This resource represents a "root of authority" on-chain.
    // Only the association can certify these capabilities (cf.
    // `certify_granting_capability` in this module). In the VASP
    // world the root VASP account would hold this capability, the
    // parent accounts for the VASP would hold TransitionCapability's, and
    // the child accounts would be of `ChildVASP` account type.
    resource struct GrantingCapability<To: copyable> {
        is_certified: bool,
    }

    // Accounts that hold an `UpdateCapability` resource may
    // update the internal `account_metadata` in the AccountType::T resource.
    resource struct UpdateCapability{}

    // Account types need to be registered before that account type can be
    // created.
    resource struct Registered<Type: copyable> { }

    ///////////////////////////////////////////////////////////////////////////
    // Initialization functions
    ///////////////////////////////////////////////////////////////////////////

    // Account limits accounting information is held within the
    // `account_metadata` field. Until this function is called account-limit
    // tracking cannot work.
    public fun grant_account_tracking(): UpdateCapability {
        // This needs to match the singleton_addr in AccountTrack
        Transaction::assert(Transaction::sender() == 0xA550C18, 2006);
        UpdateCapability{}
    }

    ///////////////////////////////////////////////////////////////////////////
    // Normal usage functions
    ///////////////////////////////////////////////////////////////////////////

    // Return a pre-certified AccountType. This is OK to return since there
    // is no way for this resource to be saved on-chain at the top-level
    // with the exception of `save_account` in the account module.
    public fun create<AccountType: copyable>(
        fresh_address: address,
        account_metadata: AccountType
    ): T<AccountType> {
        assert_is_registered<AccountType>();
        T {
            account_metadata,
            is_certified: true,
            root_address: fresh_address,
        }
    }

    // Publishes a non-certified T<Type> resource under the
    // sending account address.
    public fun apply_for<Type: copyable>(account_metadata: Type, root_address: address) {
        assert_is_registered<Type>();
        move_to_sender(T<Type> {
            is_certified: false,
            account_metadata,
            root_address,
        });
    }

    // Transitions an account from a `T<Empty>` to a `T<To>`. The account
    // at `addr` must already have a non-certified `T<To>` resource
    // published, and a certified `T<Empty>` resource published. This
    // certifies the `T<To>` resource, and unpublishes the `T<Empty>`.
    // The sending account must have a certified `TransitionCapability<To>`
    // resource, and its `root_address` must match the `root_address` in
    // the published `T<To>` resource under `addr`.
    // Once the account is transitioned to a non-Empty account type, that
    // account type cannot be transitioned any further.
    public fun transition<To: copyable>(addr: address)
    acquires TransitionCapability, T {
        // Make sure the account is an empty account
        assert_is_a<Empty::T>(addr);
        // Get the transition capability held under the sender
        let transition_cap = borrow_global<TransitionCapability<To>>(Transaction::sender());
        // Make sure it's certified
        Transaction::assert(transition_cap.is_certified, 2000);
        let T{ is_certified: _, account_metadata: _, root_address: _ } = move_from<T<Empty::T>>(addr);
        let to = borrow_global_mut<T<To>>(addr);
        // Make sure that the root address transition capability matches
        // the root address for the published `To` account type.
        Transaction::assert(to.root_address == transition_cap.root_address, 2001);
        to.is_certified = true;
    }

    // Determine if the account at `addr` is an account of type `AccountType`
    public fun is_a<AccountType: copyable>(addr: address): bool
    acquires T {
        exists<T<AccountType>>(addr) && borrow_global<T<AccountType>>(addr).is_certified
    }

    // Return the root address of the account type.
    public fun root_address<AccountType: copyable>(addr: address): address
    acquires T {
        assert_is_a<AccountType>(addr);
        borrow_global<T<AccountType>>(addr).root_address
    }

    // Return if the account at `addr` has a certified
    // `TransitionCapability<To>` resource published under it.
    public fun has_transition_cap<To: copyable>(addr: address): bool
    acquires TransitionCapability {
        exists<TransitionCapability<To>>(addr) &&
            borrow_global<TransitionCapability<To>>(addr).is_certified
    }

    // Return the root address for the `TransitionCapability<To>`
    // stored under `addr`.
    public fun transition_cap_root_addr<To: copyable>(addr: address): address
    acquires TransitionCapability {
        assert_has_transition_cap<To>(addr);
        borrow_global<TransitionCapability<To>>(addr).root_address
    }

    // Return whether the account at `addr` has a certified `GrantingCapability<To>`
    // published under it.
    public fun has_granting_cap<To: copyable>(addr: address): bool
    acquires GrantingCapability {
        exists<GrantingCapability<To>>(addr) &&
            borrow_global<GrantingCapability<To>>(addr).is_certified
    }

    // Return a copy of the account metadata for the account at `addr`.
    public fun account_metadata<AccountType: copyable>(addr: address): AccountType
    acquires T {
        assert_is_a<AccountType>(addr);
        *&borrow_global<T<AccountType>>(addr).account_metadata
    }

    // Does a value-level swap of the underlying `account_metadata` data. This
    // can allow interior mutability over the contained AccountType
    // metadata for callers with the appropriate TransitionCapability.
    // Only TransitionCapability's with the same `root_address` as
    // specified in the account types `root_address` can update
    // this field.
    public fun update<Type: copyable>(addr: address, new_account_metadata: Type)
    acquires T, TransitionCapability {
        let sender = Transaction::sender();
        assert_is_a<Type>(addr);
        // Make sure that the sender has a transition capability.
        assert_has_transition_cap<Type>(sender);
        let account_type = borrow_global_mut<T<Type>>(addr);
        let transition_cap = borrow_global<TransitionCapability<Type>>(sender);
        // Make sure that the transition capability and the account at
        // `addr` have the same root of authority.
        Transaction::assert(account_type.root_address == transition_cap.root_address, 2001);
        account_type.account_metadata = new_account_metadata;
    }

    // Certain modules are allowed interior mutability without having a
    // TransitionCapability. This allows the same mutability as `update`
    // above, but requires an `UpdateCapability` in order to update
    // account metadata.
    public fun update_with_capability<AccountType: copyable>(
        addr: address,
        new_account_metadata: AccountType,
        _update_cap: &UpdateCapability
    ) acquires T {
        assert_is_a<AccountType>(addr);
        let account_type = borrow_global_mut<T<AccountType>>(addr);
        account_type.account_metadata = new_account_metadata;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Granting and transition-power-related functions
    ///////////////////////////////////////////////////////////////////////////

    // Publish an uncertified `TransitionCapability<To>` under the
    // sending account with the given `root_address`.
    public fun apply_for_transition_capability<To: copyable>(root_address: address) {
        assert_is_registered<To>();
        move_to_sender(TransitionCapability<To> {
            is_certified: false,
            root_address
        });
    }

    // Publish an uncertified `GrantingCapability<To>` under the
    // sending account.
    public fun apply_for_granting_capability<To: copyable>() {
        assert_is_registered<To>();
        move_to_sender(GrantingCapability<To> { is_certified: false });
    }

    // Any account with a `GrantingCapability<To>` can certify
    // `TransitionCapability<To>`s as long as the `root_address` in
    // the TransitionCapability matches the address where the
    // `GrantingCapability` is stored.
    public fun grant_transition_capability<To: copyable>(addr: address)
    acquires GrantingCapability, TransitionCapability {
        let can_grant = borrow_global<GrantingCapability<To>>(Transaction::sender()).is_certified;
        Transaction::assert(can_grant, 2001);
        borrow_global_mut<TransitionCapability<To>>(addr).is_certified = true;
    }

    // Only an association account can certify a GrantingCapability (of any
    // type).
    public fun certify_granting_capability<To: copyable>(addr: address)
    acquires GrantingCapability {
        Association::assert_sender_is_association();
        borrow_global_mut<GrantingCapability<To>>(addr).is_certified = true;
    }

    // Remove the transition capability from the account at `addr`. The
    // sender must have the correct GrantingCapability published under it.
    public fun remove_transition_capability<To: copyable>(addr: address)
    acquires TransitionCapability, GrantingCapability {
        let sender = Transaction::sender();
        let granting_cap = borrow_global<GrantingCapability<To>>(sender);
        TransitionCapability { is_certified: _, root_address: _ } = move_from<TransitionCapability<To>>(addr);
        Transaction::assert(granting_cap.is_certified, 2000);
    }

    // The sending account no longer wants to hold its granting capability.
    // This unpublishes it. It is important to note that this _does not_
    // invalidate any child accounts with a `root_address` pointing at
    // this account.
    public fun remove_granting_capability_from_sender<To: copyable>()
    acquires GrantingCapability {
        GrantingCapability { is_certified: _ } = move_from<GrantingCapability<To>>(Transaction::sender());
    }

    // Remove the `GrantingCapability` from the account at `addr`. Must be
    // called by an association account.
    // It is important to note that this _does not_ invalidate any child
    // accounts with a `root_address` pointing at this account.
    public fun remove_granting_capability<To: copyable>(addr: address)
    acquires GrantingCapability {
        Association::assert_sender_is_association();
        GrantingCapability { is_certified: _ } = move_from<GrantingCapability<To>>(addr);
    }

    // Register `Type` as a valid account type.
    public fun register<Type: copyable>() {
        Transaction::assert(Transaction::sender() == singleton_addr(), 2001);
        move_to_sender(Registered<Type>{});
    }

    // Assert that the account at `addr` is of type `Type`.
    public fun assert_is_a<Type: copyable>(addr: address)
    acquires T {
        Transaction::assert(is_a<Type>(addr), 2004);
    }

    // Assert that the account at `addr` has a certified
    // `TransitionCapability<To>` published under it.
    public fun assert_has_transition_cap<To: copyable>(addr: address)
    acquires TransitionCapability {
        Transaction::assert(has_transition_cap<To>(addr), 2005);
    }

    // The singleton address where we publish the registration of account
    // types.
    public fun singleton_addr(): address {
        0xA550C18
    }

    // Assert that `Type` is a registered account type.
    fun assert_is_registered<Type: copyable>() {
        Transaction::assert(exists<Registered<Type>>(singleton_addr()), 2003);
    }
}
