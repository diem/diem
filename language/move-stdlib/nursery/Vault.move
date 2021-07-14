/// A module which implements secure memory (called a *vault*) of some content which can only be operated
/// on if authorized by a signer. Authorization is managed by
/// [*capabilities*](https://en.wikipedia.org/wiki/Capability-based_security). The vault supports delegation
/// of capabilities to other signers (including revocation) as well as transfer of ownership.
///
/// # Overview
///
/// ## Capabilities
///
/// Capabilities are unforgeable tokens which represent the right to perform a particular
/// operation on the vault. To acquire a capability instance, authentication via a signer is needed.
/// This signer must either be the owner of the vault, or someone the capability has been delegated to.
////
/// Once acquired, a capability can be passed to other functions to perform the operation it enables.
/// Specifically, those called functions do not need to have access to the original signer. This is a key
/// property of capability based security as it prevents granting more rights to code than needed.
///
/// Capability instances are unforgeable because they are localized to transactions. They can only be
/// created by functions of this module, and as they do not have the Move language `store` or `key` abilities,
/// they cannot leak out of a transaction.
///
/// Example:
///
/// ```move
/// struct Content has store { ssn: u64 }
/// ...
/// // Create new vault
/// Vault::new(signer, b"My Vault", Content{ ssn: 525659745 });
/// ...
/// // Obtain a read capability
/// let read_cap = Vault::acquire_read_cap<Content>(signer);
/// process(&read_cap)
/// ...
/// fun process(cap: &Vault::ReadCap<Content>) {
///     let accessor = Vault::read_accessor(cap);
///     let content = Vault::borrow(accessor);
///     << do something with `content: &Content` >>
///     Vault::release_read_accessor(accessor);
/// }
/// ```
///
/// ## Delegation
///
/// Delegation provides the option to delegate the right to acquire a vault capability to other
/// signers than the owner of the vault. Delegates still need to authenticate themselves using their
/// signer, similar as the owner does. All security arguments for owners apply also to delegates.
/// Delegation can be revoked removing previously granted rights from a delegate.
///
/// Delegation can be configured to be transitive by granting the right to acquire a delegation capability
/// to delegates, which can then further delegate access rights.
///
/// By default, when a vault is created, it does not support delegation. The owner of the vault
/// needs to explicitly enable delegation. This allows to create vaults which are not intended for delegation
/// and one does not need to worry about its misuse.
///
/// Example:
///
/// ```move
/// Vault::new(signer, b"My Vault", Content{ ssn: 525659745 });
/// // Enable delegation for this vault. Only the owning signer can do this.
/// Vault::enable_delegation<Content>(signer);
/// ...
/// // Delegate read capability to some other signer.
/// let delegate_cap = Vault::acquire_delegate_cap<Content>(signer);
/// Vault::delegate_read_cap(&delegate_cap, other_signer);
/// ...
/// // Other signer can now acquire read cap
/// let read_cap = Vault::acquire_read_cap<Content>(other_signer);
/// ...
/// // The granted capability can be revoked. There is no need to have the other signer for this.
/// Vault::revoke_read_cap(&delegate_cap, Signer::address_of(other_signer));
/// ```
///
/// ## Abilities
///
/// Currently, we require that the `Content` type of a vault has the `drop` ability in order to instantiate
/// a capability type like `ReadCap<Content>`. Without this, capabilities themselves would need to have an
/// explicit release function, which makes little sense as they are pure values. We expect the Move
/// language to have 'phantom type parameters' or similar features added, which will allows us to have
/// `ReadCap<Content>` droppable and copyable without `Content` needing the same.
module Std::Vault {

    use Std::Errors;
    use Std::Event;
    use Std::Option;
    use Std::Signer;
    use Std::Vector;

    // ================================================================================================================
    // Error reasons

    const EVAULT: u64 = 0;
    const EDELEGATE: u64 = 1;
    const EACCESSOR_IN_USE: u64 = 2;
    const EACCESSOR_INCONSISTENCY: u64 = 3;
    const EDELEGATE_TO_SELF: u64 = 4;
    const EDELEGATION_NOT_ENABLED: u64 = 5;
    const EEVENT: u64 = 6;


    // ================================================================================================================
    // Capabilities

    /// A capability to read the content of the vault. Notice that the capability cannot be
    /// stored but can be freely copied and dropped.
    //
    /// TODO: remove `drop` on `Content` here and elsewhere once we have phantom type parameters.
    struct ReadCap<phantom Content: store + drop> has copy, drop {
        vault_address: address,
        authority: address
    }

    /// A capability to modify the content of the vault.
    struct ModifyCap<phantom Content: store + drop> has copy, drop {
        vault_address: address,
        authority: address
    }

    /// A capability to delegate access to the vault.
    struct DelegateCap<phantom Content: store + drop> has copy, drop {
        vault_address: address,
        authority: address
    }

    /// A capability to transfer ownership of the vault.
    struct TransferCap<phantom Content: store + drop> has copy, drop {
        vault_address: address,
        authority: address
    }

    /// A type describing a capability. This is used for functions like `Self::delegate` where we need to
    /// specify capability types.
    struct CapType has copy, drop, store { code: u8 }

    /// Creates a read capability type.
    public fun read_cap_type(): CapType { CapType{ code : 0 } }

    /// Creates a modify  capability type.
    public fun modify_cap_type(): CapType { CapType{ code : 1 } }

    /// Creates a delegate  capability type.
    public fun delegate_cap_type(): CapType { CapType{ code : 2 } }

    /// Creates a transfer  capability type.
    public fun transfer_cap_type(): CapType { CapType{ code : 3 } }

    // ================================================================================================================
    // Events

    /// An event which we generate on vault access delegation or revocation if event generation is enabled.
    struct VaultDelegateEvent has store, drop {
        metadata: vector<u8>,
        vault_address: address,
        authority: address,
        delegate: address,
        cap: CapType,
        is_revoked: bool,
    }

    /// An event which we generate on vault transfer if event generation is enabled.
    struct VaultTransferEvent has store, drop {
        metadata: vector<u8>,
        vault_address: address,
        authority: address,
        new_vault_address: address,
    }

    // ================================================================================================================
    // Representation

    /// Private. The vault representation.
    struct Vault<Content: store> has key {
        /// The content. If the option is empty, the content is currently moved into an
        /// accessor in order to work with it.
        content: Option::Option<Content>,
    }

    /// Private. If the vault supports delegation, information about the delegates.
    struct VaultDelegates<phantom Content: store> has key {
         /// The currently authorized delegates.
        delegates: vector<address>,
    }

    /// Private. If event generation is enabled, contains the event generators.
    struct VaultEvents<phantom Content: store> has key {
        /// Metadata which identifies this vault. This information is used
        /// in events generated by this module.
        metadata: vector<u8>,
        /// Event handle for vault delegation.
        delegate_events: Event::EventHandle<VaultDelegateEvent>,
        /// Event handle for vault transfer.
        transfer_events: Event::EventHandle<VaultTransferEvent>,
    }

    /// Private. A value stored at a delegates address pointing to the owner of the vault. Also
    /// describes the capabilities granted to this delegate.
    struct VaultDelegate<phantom Content: store> has key {
        vault_address: address,
        granted_caps: vector<CapType>,
    }

    // ================================================================================================================
    // Vault Creation and Removal

    /// Creates new vault for the given signer. The vault is populated with the `initial_content`.
    public fun new<Content: store>(owner: &signer,  initial_content: Content) {
        let addr = Signer::address_of(owner);
        assert(!exists<Vault<Content>>(addr), Errors::already_published(EVAULT));
        move_to<Vault<Content>>(
            owner,
            Vault{
                content: Option::some(initial_content)
            }
        )
    }

    /// Returns `true` if the delegation functionality has been enabled.
    /// Returns `false` otherwise.
    public fun is_delegation_enabled<Content: store>(owner: &signer): bool {
        let addr = Signer::address_of(owner);
        assert(exists<Vault<Content>>(addr), Errors::not_published(EVAULT));
        exists<VaultDelegates<Content>>(addr)
    }

    /// Enables delegation functionality for this vault. By default, vaults to not support delegation.
    public fun enable_delegation<Content: store>(owner: &signer) {
        assert(!is_delegation_enabled<Content>(owner), Errors::already_published(EDELEGATE));
        move_to<VaultDelegates<Content>>(owner, VaultDelegates{delegates: Vector::empty()})
    }

    /// Enables event generation for this vault. This passed metadata is used to identify
    /// the vault in events.
    public fun enable_events<Content: store>(owner: &signer, metadata: vector<u8>) {
        let addr = Signer::address_of(owner);
        assert(exists<Vault<Content>>(addr), Errors::not_published(EVAULT));
        assert(!exists<VaultEvents<Content>>(addr), Errors::already_published(EEVENT));
        move_to<VaultEvents<Content>>(
            owner,
            VaultEvents{
                metadata,
                delegate_events: Event::new_event_handle<VaultDelegateEvent>(owner),
                transfer_events: Event::new_event_handle<VaultTransferEvent>(owner),
            }
        );
    }

    /// Removes a vault and all its associated data, returning the current content. In order for
    /// this to succeed, there must be no active accessor for the vault.
    public fun remove_vault<Content: store + drop>(owner: &signer): Content
    acquires Vault, VaultDelegates, VaultDelegate, VaultEvents {
        let addr = Signer::address_of(owner);
        assert(exists<Vault<Content>>(addr), Errors::not_published(EVAULT));
        let Vault{content} = move_from<Vault<Content>>(addr);
        assert(Option::is_some(&content), Errors::invalid_state(EACCESSOR_IN_USE));

        if (exists<VaultDelegates<Content>>(addr)) {
            let delegate_cap = DelegateCap<Content>{vault_address: addr, authority: addr};
            revoke_all(&delegate_cap);
        };
        if (exists<VaultEvents<Content>>(addr)) {
            let VaultEvents{metadata: _metadata, delegate_events, transfer_events} =
                move_from<VaultEvents<Content>>(addr);
            Event::destroy_handle(delegate_events);
            Event::destroy_handle(transfer_events);
        };

        Option::extract(&mut content)
    }

    // ================================================================================================================
    // Acquiring Capabilities

    /// Acquires the capability to read the vault. The passed signer must either be the owner
    /// of the vault or a delegate with appropriate access.
    public fun acquire_read_cap<Content: store + drop>(requester: &signer): ReadCap<Content>
    acquires VaultDelegate {
        let (vault_address, authority) = validate_cap<Content>(requester, read_cap_type());
        ReadCap{ vault_address, authority }
    }

    /// Acquires the capability to modify the vault. The passed signer must either be the owner
    /// of the vault or a delegate with appropriate access.
    public fun acquire_modify_cap<Content: store + drop>(requester: &signer): ModifyCap<Content>
    acquires VaultDelegate {
        let (vault_address, authority) = validate_cap<Content>(requester, modify_cap_type());
        ModifyCap{ vault_address, authority }
    }

    /// Acquires the capability to delegate access to the vault. The passed signer must either be the owner
    /// of the vault or a delegate with appropriate access.
    public fun acquire_delegate_cap<Content: store + drop>(requester: &signer): DelegateCap<Content>
    acquires VaultDelegate {
        let (vault_address, authority) = validate_cap<Content>(requester, delegate_cap_type());
        DelegateCap{ vault_address, authority }
    }

    /// Acquires the capability to transfer the vault. The passed signer must either be the owner
    /// of the vault or a delegate with appropriate access.
    public fun acquire_transfer_cap<Content: store + drop>(requester: &signer): TransferCap<Content>
    acquires VaultDelegate {
        let (vault_address, authority) = validate_cap<Content>(requester, transfer_cap_type());
        TransferCap{ vault_address, authority }
    }

    /// Private. Validates whether a capability can be acquired by the given signer. Returns the
    /// pair of the vault address and the used authority.
    fun validate_cap<Content: store + drop>(requester: &signer, cap: CapType): (address, address)
    acquires VaultDelegate {
        let addr = Signer::address_of(requester);
        if (exists<VaultDelegate<Content>>(addr)) {
            // The signer is a delegate. Check it's granted capabilities.
            let delegate = borrow_global<VaultDelegate<Content>>(addr);
            assert(Vector::contains(&delegate.granted_caps, &cap), Errors::requires_capability(EDELEGATE));
            (delegate.vault_address, addr)
        } else {
            // If it is not a delegate, it must be the owner to succeed.
            assert(exists<Vault<Content>>(addr), Errors::not_published(EVAULT));
            (addr, addr)
        }
    }


    // ================================================================================================================
    // Read Accessor

    /// A read accessor for the content of the vault.
    struct ReadAccessor<Content: store + drop> {
        content: Content,
        vault_address: address,
    }

    /// Creates a read accessor for the content in the vault based on a read capability.
    ///
    /// Only one accessor (whether read or modify) for the same vault can exist at a time, and this
    /// function will abort if one is in use. An accessor must be explicitly released using
    /// `Self::release_read_accessor`.
    public fun read_accessor<Content: store + drop>(cap: &ReadCap<Content>): ReadAccessor<Content>
    acquires Vault {
        let content = &mut borrow_global_mut<Vault<Content>>(cap.vault_address).content;
        assert(Option::is_some(content), Errors::invalid_state(EACCESSOR_IN_USE));
        ReadAccessor{ vault_address: cap.vault_address, content: Option::extract(content) }
    }

    /// Returns a reference to the content represented by a read accessor.
    public fun borrow<Content: store + drop>(accessor: &ReadAccessor<Content>): &Content {
        &accessor.content
    }

    /// Releases read accessor.
    public fun release_read_accessor<Content: store + drop>(accessor: ReadAccessor<Content>)
    acquires Vault {
        let ReadAccessor{ content: new_content, vault_address } = accessor;
        let content = &mut borrow_global_mut<Vault<Content>>(vault_address).content;
        // We (should be/are) able to prove that the below cannot happen, but we leave the assertion
        // here anyway for double safety.
        assert(Option::is_none(content), Errors::internal(EACCESSOR_INCONSISTENCY));
        Option::fill(content, new_content);
    }

    // ================================================================================================================
    // Modify Accessor

    /// A modify accessor for the content of the vault.
    struct ModifyAccessor<Content: store + drop> {
        content: Content,
        vault_address: address,
    }

    /// Creates a modify accessor for the content in the vault based on a modify capability. This
    /// is similar like `Self::read_accessor` but the returned accessor will allow to mutate
    /// the content.
    public fun modify_accessor<Content: store + drop>(cap: &ModifyCap<Content>): ModifyAccessor<Content>
    acquires Vault {
        let content = &mut borrow_global_mut<Vault<Content>>(cap.vault_address).content;
        assert(Option::is_some(content), Errors::invalid_state(EACCESSOR_IN_USE));
        ModifyAccessor{ vault_address: cap.vault_address, content: Option::extract(content) }
    }

    /// Returns a mutable reference to the content represented by a modify accessor.
    public fun borrow_mut<Content: store + drop>(accessor: &mut ModifyAccessor<Content>): &mut Content {
        &mut accessor.content
    }

    /// Releases a modify accessor. This will ensure that any modifications are written back
    /// to the vault.
    public fun release_modify_accessor<Content: store + drop>(accessor: ModifyAccessor<Content>)
    acquires Vault {
        let ModifyAccessor{ content: new_content, vault_address } = accessor;
        let content = &mut borrow_global_mut<Vault<Content>>(vault_address).content;
        // We (should be/are) able to prove that the below cannot happen, but we leave the assertion
        // here anyway for double safety.
        assert(Option::is_none(content), Errors::internal(EACCESSOR_INCONSISTENCY));
        Option::fill(content, new_content);
    }


    // ================================================================================================================
    // Delegation

    /// Delegates the right to acquire a capability of the given type. Delegation must have been enabled
    /// during vault creation for this to succeed.
    public fun delegate<Content: store + drop>(cap: &DelegateCap<Content>, to_signer: &signer, cap_type: CapType)
    acquires VaultDelegates, VaultDelegate, VaultEvents {
        assert(
            exists<VaultDelegates<Content>>(cap.vault_address),
            Errors::invalid_state(EDELEGATION_NOT_ENABLED)
        );

        let addr = Signer::address_of(to_signer);
        assert(addr != cap.vault_address, Errors::invalid_argument(EDELEGATE_TO_SELF));

        if (!exists<VaultDelegate<Content>>(addr)) {
            // Create VaultDelegate if it is not yet existing.
            move_to<VaultDelegate<Content>>(
                to_signer,
                VaultDelegate{vault_address: cap.vault_address, granted_caps: Vector::empty()}
            );
            // Add the the delegate to VaultDelegates.
            let vault_delegates = borrow_global_mut<VaultDelegates<Content>>(cap.vault_address);
            add_element(&mut vault_delegates.delegates, addr);
        };

        // Grant the capability.
        let delegate = borrow_global_mut<VaultDelegate<Content>>(addr);
        add_element(&mut delegate.granted_caps, *&cap_type);

        // Generate event
        emit_delegate_event(cap, cap_type, addr, false);
    }

    /// Revokes the delegated right to acquire a capability of given type.
    public fun revoke<Content: store + drop>(cap: &DelegateCap<Content>, addr: address, cap_type: CapType)
    acquires VaultDelegates, VaultDelegate, VaultEvents {
        assert(
            exists<VaultDelegates<Content>>(cap.vault_address),
            Errors::invalid_state(EDELEGATION_NOT_ENABLED)
        );
        assert(exists<VaultDelegate<Content>>(addr), Errors::not_published(EDELEGATE));

        let delegate = borrow_global_mut<VaultDelegate<Content>>(addr);
        remove_element(&mut delegate.granted_caps, &cap_type);

        // If the granted caps of this delegate drop to zero, remove it.
        if (Vector::is_empty(&delegate.granted_caps)) {
            let VaultDelegate{ vault_address: _owner, granted_caps: _granted_caps} =
                move_from<VaultDelegate<Content>>(addr);
            let vault_delegates = borrow_global_mut<VaultDelegates<Content>>(cap.vault_address);
            remove_element(&mut vault_delegates.delegates, &addr);
        };

        // Generate event.
        emit_delegate_event(cap, cap_type, addr, true);
    }

    /// Revokes all delegate rights for this vault.
    public fun revoke_all<Content: store + drop>(cap: &DelegateCap<Content>)
    acquires VaultDelegates, VaultDelegate, VaultEvents {
        assert(
            exists<VaultDelegates<Content>>(cap.vault_address),
            Errors::invalid_state(EDELEGATION_NOT_ENABLED)
        );
        let delegates = &mut borrow_global_mut<VaultDelegates<Content>>(cap.vault_address).delegates;
        while (!Vector::is_empty(delegates)) {
            let addr = Vector::pop_back(delegates);
            let VaultDelegate{ vault_address: _vault_address, granted_caps} =
                move_from<VaultDelegate<Content>>(cap.vault_address);
            while (!Vector::is_empty(&granted_caps)) {
                let cap_type = Vector::pop_back(&mut granted_caps);
                emit_delegate_event(cap, cap_type, addr, true);
            }
        }
    }

    /// Helper to remove an element from a vector.
    fun remove_element<E: drop>(v: &mut vector<E>, x: &E) {
        let (found, index) = Vector::index_of(v, x);
        if (found) {
            Vector::remove(v, index);
        }
    }

    /// Helper to add an element to a vector.
    fun add_element<E: drop>(v: &mut vector<E>, x: E) {
        if (!Vector::contains(v, &x)) {
            Vector::push_back(v, x)
        }
    }

    /// Emits a delegation or revocation event if event generation is enabled.
    fun emit_delegate_event<Content: store + drop>(
           cap: &DelegateCap<Content>,
           cap_type: CapType,
           delegate: address,
           is_revoked: bool
    ) acquires VaultEvents {
        if (exists<VaultEvents<Content>>(cap.vault_address)) {
            let event = VaultDelegateEvent{
                metadata: *&borrow_global<VaultEvents<Content>>(cap.vault_address).metadata,
                vault_address: cap.vault_address,
                authority: cap.authority,
                delegate,
                cap: cap_type,
                is_revoked
            };
            Event::emit_event(&mut borrow_global_mut<VaultEvents<Content>>(cap.vault_address).delegate_events, event);
        }
    }

    // ================================================================================================================
    // Transfer

    /// Transfers ownership of the vault to a new signer. All delegations are revoked before transfer,
    /// and the new owner must re-create delegates as needed.
    public fun transfer<Content: store + drop>(cap: &TransferCap<Content>, to_owner: &signer)
    acquires Vault, VaultEvents, VaultDelegate, VaultDelegates {
        let new_addr = Signer::address_of(to_owner);
        assert(!exists<Vault<Content>>(new_addr), Errors::already_published(EVAULT));
        assert(
            Option::is_some(&borrow_global<Vault<Content>>(cap.vault_address).content),
            Errors::invalid_state(EACCESSOR_IN_USE)
        );

        // Revoke all delegates.
        if (exists<VaultDelegates<Content>>(cap.vault_address)) {
            let delegate_cap = DelegateCap<Content>{vault_address: cap.vault_address, authority: cap.authority };
            revoke_all(&delegate_cap);
        };

        // Emit event if event generation is enabled. We emit the event on the old vault not the new one.
        if (exists<VaultEvents<Content>>(cap.vault_address)) {
            let event = VaultTransferEvent {
                metadata: *&borrow_global<VaultEvents<Content>>(cap.vault_address).metadata,
                vault_address: cap.vault_address,
                authority: cap.authority,
                new_vault_address: new_addr
            };
            Event::emit_event(&mut borrow_global_mut<VaultEvents<Content>>(cap.vault_address).transfer_events, event);
        };

        // Move the vault.
        move_to<Vault<Content>>(to_owner, move_from<Vault<Content>>(cap.vault_address));
    }
}
