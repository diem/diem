address 0x0:

module LibraWriteSetManager {
    use 0x0::LibraAccount;
    use 0x0::Event;
    use 0x0::Hash;
    use 0x0::Transaction;
    use 0x0::LibraConfig;

    resource struct T {
        sequence_number: u64,
        upgrade_events: Event::EventHandle<Self::UpgradeEvent>,
    }

    struct UpgradeEvent {
        writeset_payload: vector<u8>,
    }

    public fun initialize() {
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);

        move_to_sender<T>(T {
            sequence_number: 0,
            upgrade_events: Event::new_event_handle<Self::UpgradeEvent>(),
        });
    }

    fun prologue(
        writeset_sequence_number: u64,
        writeset_public_key: vector<u8>,
    ) acquires T {
        let sender = Transaction::sender();
        Transaction::assert(sender == 0xA550C18, 33);

        let association_auth_key = LibraAccount::authentication_key(sender);

        let t_ref = borrow_global<T>(0xA550C18);
        Transaction::assert(writeset_sequence_number >= t_ref.sequence_number, 3);

        Transaction::assert(writeset_sequence_number == t_ref.sequence_number, 11);
        Transaction::assert(
            Hash::sha3_256(writeset_public_key) == association_auth_key,
            2
        );
    }

    fun epilogue(writeset_payload: vector<u8>) acquires T {
        let t_ref = borrow_global_mut<T>(0xA550C18);
        t_ref.sequence_number = t_ref.sequence_number + 1;

        Event::emit_event<Self::UpgradeEvent>(
            &mut t_ref.upgrade_events,
            UpgradeEvent { writeset_payload },
        );
        LibraConfig::reconfigure();
    }
}
