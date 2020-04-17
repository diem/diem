// An account A that wishes to use K-of-N multisig for authentication can publish a
// `MultiEd25519Manager::T` resource under A to allow:
// (1) Any of the N participants to rotate their individual public keys
// (2) Any of the N particpants to update the multisig policy at A, but only to pull in the result
//     of previous individual key rotations (i.e., cannot add/remove participants from the policy or
//     change the threshold K). In addition, these updates are rate-limited; they can only happen
//     after `minimum_update_window` microseconds have elapsed since the last update
// (3) Multi-signed transactions sent from A to arbitrarily update the participants, threshold, and
//     minimum update window.
address 0x0:
module MultiEd25519Manager {
    use 0x0::Authenticator;
    use 0x0::LibraAccount;
    use 0x0::LibraTimestamp;
    use 0x0::SharedEd25519PublicKey;
    use 0x0::Transaction;
    use 0x0::Vector;

    resource struct T {
        participants: vector<address>,
        public_key: Authenticator::MultiEd25519PublicKey,
        rotation_cap: LibraAccount::KeyRotationCapability,
        last_update_time: u64,
        minimum_update_window: u64,
    }

    // Create a a multisig policy from a vector of addresses that each hold `SharedEd25519PublicKey`
    // resources and a `threshold` and rotate the sender's authentication key to the key derived
    // from `participants` and `threshold`.
    // Note: this does *not* check uniqueness of addresses. Repeating addresses are convenient to
    // encode weighted multisig policies. For example Alice AND 1 of Bob or Carol is
    // addresses: {alice_addr, alice_addr, bob_addr, carol_addr}, threshold:
    // Aborts if the sender is in `participants`
    // Aborts if any element of `participants` does not have a `SharedEd25519PublicKey` resource.
    // Aborts if threshold is zero or bigger than the length of `participabts`.
    public fun publish_for_sender(
        participants: vector<address>,
        threshold: u8,
        minimum_update_window: u64
    ) {
        let public_key = new_multisig_public_key(copy participants, threshold);
        let t = T {
            participants,
            public_key: copy public_key,
            rotation_cap: LibraAccount::extract_sender_key_rotation_capability(),
            last_update_time: LibraTimestamp::now_microseconds(),
            minimum_update_window,
        };
        update_keys_(&mut t, public_key);
        move_to_sender(t)
    }

    // derive a multi-ed public key from public keys of `participants` and `threshold`
    fun new_multisig_public_key(
        participants: vector<address>,
        threshold: u8
    ): Authenticator::MultiEd25519PublicKey {
        let i = 0;
        let len = Vector::length(&participants);
        let public_keys = Vector::empty<vector<u8>>();
        // grab a public key from each address's `SharedEd25519PublicKey` resource
        while (i < len) {
            Vector::push_back(
                &mut public_keys,
                SharedEd25519PublicKey::key(*Vector::borrow(&participants, i))
            );
            i = i + 1;
        };
        Authenticator::create_multi_ed25519(public_keys, threshold)
    }

    // Update both the `t`'s public key and the multisig account's authentication key
    fun update_keys_(t: &mut T, new_public_key: Authenticator::MultiEd25519PublicKey) {
        let new_auth_key = Authenticator::multi_ed25519_authentication_key(&new_public_key);
        t.public_key = new_public_key;

        // rotate the authentication key for the multisig accounts
        LibraAccount::rotate_authentication_key_with_capability(
            &t.rotation_cap,
            new_auth_key
        );
    }

    // Update both `t`'s public key and the multisig account's authentication keys
    fun update_keys(t: &mut T, participants: vector<address>, threshold: u8) {
        update_keys_(t, new_multisig_public_key(participants, threshold))
    }

    // Update both `t`'s public key and the multisig account's authentication keys
    // Aborts if fewer than `t.minimum_update_window` seconds have elapsed since the last update
    // Aborts if the transaction sender is not in `t.participants`.
    public fun update_keys_from_participant(t: &mut T) {
        // ensure that it has been at least `t.minimum_update_window` before the last update
        let now = LibraTimestamp::now_microseconds();
        Transaction::assert(now - t.last_update_time >= t.minimum_update_window, 8002);
        t.last_update_time = now;

        // ensure that the sender is one of participants
        let participants = &t.participants;
        Transaction::assert(
            Vector::contains(
                participants,
                &Transaction::sender(),
            ),
            8003
        );
        // compute a new multisig public key that incorporates the shared key rotations
        let threshold = Authenticator::threshold(&t.public_key);
        update_keys(t, *participants, threshold)
    }

    // Rotate the sender's `SharedEd25519PublicKey`, then update the multied25519 authentication for
    // `t` to reflect the rotation of the sender's key
    public fun rotate_sender_shared_key_(t: &mut T, new_public_key: vector<u8>) {
        SharedEd25519PublicKey::rotate_sender_key(new_public_key);
        update_keys_from_participant(t)
    }

    // Rotate the sender's `SharedEd25519PublicKey`, then update the multied25519 authentication for
    // `multisig_addr` to reflect the rotation of the sender's key
    public fun rotate_sender_shared_key(
        multisig_addr: address,
        new_public_key: vector<u8>,
    ) acquires T {
        rotate_sender_shared_key_(borrow_global_mut<T>(multisig_addr), new_public_key);
    }

    // Set the authentication key for the sender to a new authentication key derived
    // from `participants` and `threshold`.
    // Unlike `rotate_sender_shared_key`, this can only be called by a transaction signed by the
    // multsig account (not a participant).
    public fun set_sender_keys(
        participants: vector<address>,
        threshold: u8
    ) acquires T {
        update_keys(borrow_global_mut<T>(Transaction::sender()), participants, threshold)
    }

    // Set the minimum update window for `t` to `new_update_window`
    public fun set_minimum_update_window(t: &mut T, new_update_window: u64) {
        t.minimum_update_window = new_update_window;
    }

    // Set the minimum update window for the sender's `T` resource to `new_update_window`.
    // Unlike `rotate_sender_shared_key`, this can only be called by a transaction signed by the
    // multsig account (not a participant).
    public fun set_minimum_update_window_for_sender(new_update_window: u64) acquires T {
        set_minimum_update_window(borrow_global_mut<T>(Transaction::sender()), new_update_window)
    }

}
