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

    /// Message sent when there are updates to the `NetworkIdentity`.
    struct NetworkIdentityChangeNotification has drop, store {
        /// The new identities
        identities: vector<vector<u8>>,
        /// The time at which the `identities` was rotated
        time_rotated_seconds: u64,
    }

    const MAX_ADDR_IDENTITIES: u64 = 100;

    // Error Codes
    /// Network ID doesn't exist when trying to get it
    const ENETWORK_ID_DOESNT_EXIST: u64 = 0;
    /// Limit exceeded on number of identities for an address
    const ENETWORK_ID_LIMIT_EXCEEDED: u64 = 1;
    /// No identities provided for changes
    const ENETWORK_ID_NO_INPUT: u64 = 2;

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// Initialize `NetworkIdentity` with an empty list
    fun initialize_network_identity(account: &signer) {
        let identities = Vector::empty<vector<u8>>();
        let identity_change_events = Event::new_event_handle<NetworkIdentityChangeNotification>(account);
        move_to(account, NetworkIdentity { identities, identity_change_events });
    }
    spec initialize_network_identity {
        let account_addr = Signer::spec_address_of(account);
        ensures exists<NetworkIdentity>(account_addr);
        modifies global<NetworkIdentity>(account_addr);
    }

    /// Return the underlying `NetworkIdentity` bytes
    public fun get(account_addr: address): vector<vector<u8>> acquires NetworkIdentity {
        assert(exists<NetworkIdentity>(account_addr), ENETWORK_ID_DOESNT_EXIST);
        *&borrow_global<NetworkIdentity>(account_addr).identities
    }

    spec get {
        aborts_if !exists<NetworkIdentity>(account_addr);
    }

    /// Update and create if not exist `NetworkIdentity`
    public fun add_identities(account: &signer, to_add: vector<vector<u8>>) acquires NetworkIdentity {
        let num_to_add = Vector::length(&to_add);
        assert(num_to_add > 0, ENETWORK_ID_NO_INPUT);

        if (!exists<NetworkIdentity>(Signer::address_of(account))) {
            initialize_network_identity(account);
        };
        let identity = borrow_global_mut<NetworkIdentity>(Signer::address_of(account));
        let identities = &mut identity.identities;

        assert(Vector::length(identities) + num_to_add <= MAX_ADDR_IDENTITIES, Errors::limit_exceeded(ENETWORK_ID_LIMIT_EXCEEDED));

        let i = 0;
        let has_change = false;
        while (i < num_to_add) {
           has_change = has_change || add_identity(identities, *Vector::borrow(&to_add, i));
           i = i + 1;
        };


        if (has_change) {
            Event::emit_event(&mut identity.identity_change_events, NetworkIdentityChangeNotification {
                identities: *&identity.identities,
                time_rotated_seconds: DiemTimestamp::now_seconds(),
            });
        }
    }

    spec add_identities {
        let account_addr = Signer::spec_address_of(account);
        let num_identities = len(global<NetworkIdentity>(account_addr).identities);
        // aborts_if len(to_add) == 0;
        // aborts_if num_identities + len(to_add) > MAX_ADDR_IDENTITIES;
        // aborts_if !exists<NetworkIdentity>(account_addr);
        modifies global<NetworkIdentity>(account_addr);
        invariant exists<NetworkIdentity>(account_addr);
        invariant num_identities <= MAX_ADDR_IDENTITIES;
    }

    /// Adds an identity and returns true if a change was made
    fun add_identity(identities: &mut vector<vector<u8>>, to_add: vector<u8>): bool {
        if (!Vector::contains(identities, &to_add)) {
            Vector::push_back(identities, to_add);
            true
        } else {
            false
        }
    }

    spec add_identity {
        ensures contains<vector<u8>>(identities, to_add);
    }

    /// Remove `NetworkIdentity`, skipping if it doesn't exist
    public fun remove_identities(account: &signer, to_remove: vector<vector<u8>>) acquires NetworkIdentity {
        let num_to_remove = Vector::length(&to_remove);
        assert(num_to_remove > 0, ENETWORK_ID_NO_INPUT);
        assert(num_to_remove <= MAX_ADDR_IDENTITIES, ENETWORK_ID_LIMIT_EXCEEDED);

        let account_addr = Signer::address_of(account);
        assert(exists<NetworkIdentity>(account_addr), ENETWORK_ID_DOESNT_EXIST);

        let identity = borrow_global_mut<NetworkIdentity>(account_addr);
        let identities = &mut identity.identities;

        let i = 0;
        let has_change = false;
        while (i < num_to_remove) {
           has_change = has_change || remove_identity(identities, *Vector::borrow(&to_remove, i));
           i = i + 1;
        };

        if (has_change) {
            Event::emit_event(&mut identity.identity_change_events, NetworkIdentityChangeNotification {
                identities: *&identity.identities,
                time_rotated_seconds: DiemTimestamp::now_seconds(),
            });
        };
    }
    spec remove_identities {
        let account_addr = Signer::spec_address_of(account);
        let num_identities = len(global<NetworkIdentity>(account_addr).identities);
        // aborts_if len(to_remove) == 0;
        // aborts_if len(to_remove) > MAX_ADDR_IDENTITIES;
        modifies global<NetworkIdentity>(account_addr);
        invariant exists<NetworkIdentity>(account_addr);
        invariant num_identities <= MAX_ADDR_IDENTITIES;
    }

    /// Removes an identity and returns true if a change was made
    fun remove_identity(identities: &mut vector<vector<u8>>, to_remove: vector<u8>): bool {
        let (exist, i) = Vector::index_of(identities, &to_remove);

        if (exist) {
            Vector::swap_remove(identities, i);
        };

        exist
    }

    spec remove_identity {
    }
}
