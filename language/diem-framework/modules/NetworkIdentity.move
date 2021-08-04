/// Module managing Diemnet NetworkIdentity
module DiemFramework::NetworkIdentity {
    use DiemFramework::DiemTimestamp;
    use Std::Errors;
    use Std::Event::{Self, EventHandle};
    use Std::Signer;
    use Std::Vector;

    /// Holder for all `NetworkIdentity` in an account
    struct NetworkIdentity has key {
        identities: vector<vector<u8>>,
        /// Event handle for `identities` rotation events
        identity_change_events: EventHandle<NetworkIdentityChangeNotification>
    }
    spec NetworkIdentity {
        include UniqueMembers<vector<u8>> {members: identities};
    }

    /// Message sent when there are updates to the `NetworkIdentity`.
    struct NetworkIdentityChangeNotification has drop, store {
        /// The new identities
        identities: vector<vector<u8>>,
        /// The time at which the `identities` was rotated
        time_rotated_seconds: u64,
    }
    spec NetworkIdentityChangeNotification {
        include UniqueMembers<vector<u8>> {members: identities};
    }

    const MAX_ADDR_IDENTITIES: u64 = 100;

    // Error Codes

    /// Network ID doesn't exist when trying to get it
    const ENETWORK_ID_DOESNT_EXIST: u64 = 0;
    /// Limit exceeded on number of identities for an address
    const ENETWORK_ID_LIMIT_EXCEEDED: u64 = 1;
    /// No identities provided for changes
    const ENETWORK_ID_NO_INPUT: u64 = 2;

    /// Initialize `NetworkIdentity` with an empty list
    fun initialize_network_identity(account: &signer) {
        let identities = Vector::empty<vector<u8>>();
        let identity_change_events = Event::new_event_handle<NetworkIdentityChangeNotification>(account);
        move_to(account, NetworkIdentity { identities, identity_change_events });
    }
    spec initialize_network_identity {
        let account_addr = Signer::spec_address_of(account);
        modifies global<NetworkIdentity>(account_addr);
    }

    /// Return the underlying `NetworkIdentity` bytes
    public fun get(account_addr: address): vector<vector<u8>> acquires NetworkIdentity {
        assert(
            exists<NetworkIdentity>(account_addr),
            Errors::invalid_state(ENETWORK_ID_DOESNT_EXIST)
        );
        *&borrow_global<NetworkIdentity>(account_addr).identities
    }
    spec get {
        aborts_if !exists<NetworkIdentity>(account_addr);
        ensures result == global<NetworkIdentity>(account_addr).identities;
    }

    /// Update and create if not exist `NetworkIdentity`
    public fun add_identities(account: &signer, to_add: vector<vector<u8>>) acquires NetworkIdentity {
        let num_to_add = Vector::length(&to_add);
        assert(num_to_add > 0, Errors::invalid_argument(ENETWORK_ID_NO_INPUT));

        if (!exists<NetworkIdentity>(Signer::address_of(account))) {
            initialize_network_identity(account);
        };
        let identity = borrow_global_mut<NetworkIdentity>(Signer::address_of(account));
        let identities = &mut identity.identities;

        assert(
            Vector::length(identities) + num_to_add <= MAX_ADDR_IDENTITIES,
            Errors::limit_exceeded(ENETWORK_ID_LIMIT_EXCEEDED)
        );

        let has_change = add_members_internal(identities, &to_add);
        if (has_change) {
            Event::emit_event(&mut identity.identity_change_events, NetworkIdentityChangeNotification {
                identities: *&identity.identities,
                time_rotated_seconds: DiemTimestamp::now_seconds(),
            });
        }
    }
    spec add_identities {
        let account_addr = Signer::spec_address_of(account);
        let prior_identities = if (exists<NetworkIdentity>(account_addr)) {
            global<NetworkIdentity>(account_addr).identities
        } else {
            vec()
        };
        let has_change = (exists i in 0..len(to_add): !contains(prior_identities, to_add[i]));

        let post handle = global<NetworkIdentity>(account_addr).identity_change_events;
        let post msg = NetworkIdentityChangeNotification {
            identities: global<NetworkIdentity>(account_addr).identities,
            time_rotated_seconds: DiemTimestamp::spec_now_seconds(),
        };

        aborts_if len(to_add) == 0;
        aborts_if len(prior_identities) + len(to_add) > MAX_ADDR_IDENTITIES;
        include has_change ==> DiemTimestamp::AbortsIfNotOperating;
        include AddMembersInternalEnsures<vector<u8>> {
            old_members: prior_identities,
            new_members: global<NetworkIdentity>(account_addr).identities,
        };
        modifies global<NetworkIdentity>(account_addr);
        emits msg to handle if has_change;
    }

    /// Remove `NetworkIdentity`, skipping if it doesn't exist
    public fun remove_identities(account: &signer, to_remove: vector<vector<u8>>) acquires NetworkIdentity {
        let num_to_remove = Vector::length(&to_remove);
        assert(num_to_remove > 0, Errors::invalid_argument(ENETWORK_ID_NO_INPUT));
        assert(
            num_to_remove <= MAX_ADDR_IDENTITIES,
            Errors::limit_exceeded(ENETWORK_ID_LIMIT_EXCEEDED)
        );

        let account_addr = Signer::address_of(account);
        assert(
            exists<NetworkIdentity>(account_addr),
            Errors::invalid_state(ENETWORK_ID_DOESNT_EXIST)
        );

        let identity = borrow_global_mut<NetworkIdentity>(account_addr);
        let identities = &mut identity.identities;

        let has_change = remove_members_internal(identities, &to_remove);
        if (has_change) {
            Event::emit_event(&mut identity.identity_change_events, NetworkIdentityChangeNotification {
                identities: *&identity.identities,
                time_rotated_seconds: DiemTimestamp::now_seconds(),
            });
        };
    }
    spec remove_identities {
        let account_addr = Signer::spec_address_of(account);
        let prior_identities = global<NetworkIdentity>(account_addr).identities;
        let has_change = (exists i in 0..len(to_remove): contains(prior_identities, to_remove[i]));

        let post handle = global<NetworkIdentity>(account_addr).identity_change_events;
        let post msg = NetworkIdentityChangeNotification {
            identities: global<NetworkIdentity>(account_addr).identities,
            time_rotated_seconds: DiemTimestamp::spec_now_seconds(),
        };

        aborts_if len(to_remove) == 0;
        aborts_if len(to_remove) > MAX_ADDR_IDENTITIES;
        aborts_if !exists<NetworkIdentity>(account_addr);
        include has_change ==> DiemTimestamp::AbortsIfNotOperating;
        include RemoveMembersInternalEnsures<vector<u8>> {
            old_members: prior_identities,
            new_members: global<NetworkIdentity>(account_addr).identities,
        };
        modifies global<NetworkIdentity>(account_addr);
        emits msg to handle if has_change;
    }

    // =================================================================
    // Set operation simulation

    /// Add all elements that appear in `to_add` into `members`.
    ///
    /// The `members` argument is essentially a set simulated by a vector, hence
    /// the uniqueness of its elements are guaranteed, before and after the bulk
    /// insertion. The `to_add` argument, on the other hand, does not guarantee
    /// to be a set and hence can have duplicated elements.
    fun add_members_internal<T: copy>(
        members: &mut vector<T>,
        to_add: &vector<T>,
    ): bool {
        let num_to_add = Vector::length(to_add);
        let num_existing = Vector::length(members);

        let i = 0;
        while ({
            spec {
                invariant i <= num_to_add;
                // the set can never reduce in size
                invariant len(members) >= len(old(members));
                // the current set maintains the uniqueness of the elements
                invariant forall j in 0..len(members), k in 0..len(members): members[j] == members[k] ==> j == k;
                // the left-split of the current set is exactly the same as the original set
                invariant forall j in 0..len(old(members)): members[j] == old(members)[j];
                // all elements in the the right-split of the current set is from the `to_add` vector
                invariant forall j in len(old(members))..len(members): contains(to_add[0..i], members[j]);
                // the current set includes everything in `to_add` we have seen so far
                invariant forall j in 0..i: contains(members, to_add[j]);
                // having no new members means that all elements in the `to_add` vector we have seen so far are already
                // in the existing set, and vice versa.
                invariant len(members) == len(old(members)) <==> (forall j in 0..i: contains(old(members), to_add[j]));
            };
            (i < num_to_add)
        }) {
            let entry = Vector::borrow(to_add, i);
            if (!Vector::contains(members, entry)) {
                Vector::push_back(members, *entry);
            };
            i = i + 1;
        };

        Vector::length(members) > num_existing
    }
    spec add_members_internal {
        pragma opaque;
        // TODO(mengxu): this is to force the prover to honor the "opaque" pragma in the ignore opaque setting
        ensures [concrete] true;

        aborts_if false;
        include AddMembersInternalEnsures<T> {
            old_members: old(members),
            new_members: members,
        };
        // ensures that the `members` argument is and remains a set
        include UniqueMembers<T>;
        // returns whether a new element is added to the set
        ensures result == (exists i in 0..len(to_add): !contains(old(members), to_add[i]));
    }
    spec schema AddMembersInternalEnsures<T: copy> {
        old_members: vector<T>;
        new_members: vector<T>;
        to_add: vector<T>;
        // everything in the `to_add` vector must be in the updated set
        ensures forall i in 0..len(to_add): contains(new_members, to_add[i]);
        // everything in the old set must remain in the updated set
        ensures forall i in 0..len(old_members): contains(new_members, old_members[i]);
        // everything in the updated set must come from either the old set or the `to_add` vector
        ensures forall i in 0..len(new_members): (contains(old_members, new_members[i]) || contains(to_add, new_members[i]));
    }

    /// Remove all elements that appear in `to_remove` from `members`.
    ///
    /// The `members` argument is essentially a set simulated by a vector, hence
    /// the uniqueness of its elements are guaranteed, before and after the bulk
    /// removal. The `to_remove` argument, on the other hand, does not guarantee
    /// to be a set and hence can have duplicated elements.
    fun remove_members_internal<T: drop>(
        members: &mut vector<T>,
        to_remove: &vector<T>,
    ): bool {
        let num_existing = Vector::length(members);
        let num_to_remove = Vector::length(to_remove);

        let i = 0;
        while ({
            spec {
                invariant i <= num_to_remove;
                // the set can never grow in size
                invariant len(members) <= len(old(members));
                // the current set maintains the uniqueness of the elements
                invariant forall j in 0..len(members), k in 0..len(members):
                    members[j] == members[k] ==> j == k;
                // all elements in the the current set come from the original set
                invariant forall j in 0..len(members): contains(old(members), members[j]);
                // the current set never contains anything from the `to_remove` vector
                invariant forall j in 0..i: !contains(members, to_remove[j]);
                // the current set should never remove an element from the original set which is not in `to_remove`
                invariant forall j in 0..len(old(members)): (contains(to_remove[0..i], old(members)[j]) || contains(members, old(members)[j]));
                // having the same member means that all elements in the `to_remove` vector we have seen so far are not
                // in the existing set, and vice versa.
                invariant len(members) == len(old(members)) <==> (forall j in 0..i: !contains(old(members), to_remove[j]));
            };
            (i < num_to_remove)
        }) {
            let entry = Vector::borrow(to_remove, i);
            let (exist, index) = Vector::index_of(members, entry);
            if (exist) {
                Vector::swap_remove(members, index);
            };
            i = i + 1;
        };

        Vector::length(members) < num_existing
    }
    spec remove_members_internal {
        // TODO: due to the complexity of the loop invariants and the extensive use of quantifiers in the spec, this
        // function needs a bit more time to verify. We may need to investigate ways to reduce the verification time
        // for this function in the future.
        pragma timeout = 120;

        pragma opaque;
        // TODO(mengxu): this is to force the prover to honor the "opaque" pragma in the ignore opaque setting
        ensures [concrete] true;

        aborts_if false;
        include RemoveMembersInternalEnsures<T> {
            old_members: old(members),
            new_members: members,
        };
        // ensures that the `members` argument is and remains a set
        include UniqueMembers<T>;
        // returns whether an element is removed from the set
        ensures result == (exists i in 0..len(to_remove): contains(old(members), to_remove[i]));
    }
    spec schema RemoveMembersInternalEnsures<T: drop> {
        old_members: vector<T>;
        new_members: vector<T>;
        to_remove: vector<T>;
        // everything in the `to_remove` vector must not be in the updated set
        ensures forall i in 0..len(to_remove): !contains(new_members, to_remove[i]);
        // all members in the updated set must be in the original set
        ensures forall i in 0..len(new_members): contains(old_members, new_members[i]);
        // an element from the original set that is not in the `to_remove` must be in the updated set
        ensures forall i in 0..len(old_members): (contains(to_remove, old_members[i]) || contains(new_members, old_members[i]));
    }

    spec schema UniqueMembers<T> {
        members: vector<T>;
        invariant forall i in 0..len(members), j in 0..len(members): members[i] == members[j] ==> i == j;
    }
}
