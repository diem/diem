address 0x1 {
/// Module managing Diemnet NetworkIdentity
module NetworkIdentity {
    use 0x1::DiemTimestamp;
    use 0x1::Event::{Self, EventHandle};
    use 0x1::Signer;
    use 0x1::Vector;

    /// An updatable `address` list with update notifications
    struct AccountList has key {
        accounts: vector<address>,
        account_change_events: EventHandle<AccountListChangeNotification>
    }

    /// Message sent when there are updates to the `AccountList`.
    struct AccountListChangeNotification has drop, store {
        /// The new accounts
        new_accounts: vector<address>,
        /// The time at which the `accounts` was rotated
        time_rotated_seconds: u64,
    }

    /// Holder for all `NetworkIdentity` in an account
    struct NetworkIdentity has key {
        identities: vector<u8>,
        /// Event handle for `identities` rotation events
        identity_change_events: EventHandle<NetworkIdentityChangeNotification>
    }

    /// Message sent when there are updates to the `NetworkIdentity`.
    struct NetworkIdentityChangeNotification has drop, store {
        /// The new identities
        new_identities: vector<u8>,
        /// The time at which the `identities` was rotated
        time_rotated_seconds: u64,
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// Initialize `NetworkIdentity` with an empty list
    fun initialize(account: &signer) {
        let identities = Vector::empty<u8>();
        let identity_change_events = Event::new_event_handle<NetworkIdentityChangeNotification>(account);
        move_to(account, NetworkIdentity { identities, identity_change_events });
    }
    spec initialize {
        pragma opaque;
        let addr = Signer::address_of(account);
        modifies global<NetworkIdentity>(addr);
        ensures exists<NetworkIdentity>(addr);
    }

    /// Return the underlying `NetworkIdentity` bytes
    public fun get(account: address): vector<u8> acquires NetworkIdentity {
        *&borrow_global<NetworkIdentity>(account).identities
    }

    /// Update and create if not exist `NetworkIdentity`
    public fun update_identities(account: &signer, identities: vector<u8>) acquires NetworkIdentity {
        if (!exists<NetworkIdentity>(Signer::address_of(account))) {
            initialize(account);
        };
        let holder = borrow_global_mut<NetworkIdentity>(Signer::address_of(account));
        holder.identities = copy identities;

        Event::emit_event(&mut holder.identity_change_events, NetworkIdentityChangeNotification {
            new_identities: identities,
            time_rotated_seconds: DiemTimestamp::now_seconds(),
        });
    }

    /// Update and create if not exist `AccountList`
    public fun update_accounts(account: &signer, accounts: vector<address>) acquires AccountList {
        if (!exists<NetworkIdentity>(Signer::address_of(account))) {
			let accounts = Vector::empty<address>();
			let account_change_events = Event::new_event_handle<AccountListChangeNotification>(account);
			move_to(account, AccountList { accounts, account_change_events });
        };
        let holder = borrow_global_mut<AccountList>(Signer::address_of(account));
        holder.accounts = copy accounts;

        Event::emit_event(&mut holder.account_change_events, AccountListChangeNotification {
            new_accounts: accounts,
            time_rotated_seconds: DiemTimestamp::now_seconds(),
        });
    }
}
}
