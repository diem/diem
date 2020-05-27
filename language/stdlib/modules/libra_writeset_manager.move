address 0x0 {

module LibraWriteSetManager {
    use 0x0::LibraAccount;
    use 0x0::Event;
    use 0x0::Hash;
    use 0x0::Signer;
    use 0x0::Transaction;
    use 0x0::LibraConfig;

    resource struct T {
        upgrade_events: Event::EventHandle<Self::UpgradeEvent>,
    }

    struct UpgradeEvent {
        writeset_payload: vector<u8>,
    }

    public fun initialize(sig: &signer) {
        Transaction::assert(Signer::address_of(sig) == 0xA550C18, 1);

        move_to(
            sig,
            T {
                upgrade_events: Event::new_event_handle<Self::UpgradeEvent>(sig),
            }
        );
    }

    fun prologue(
        writeset_sequence_number: u64,
        writeset_public_key: vector<u8>,
    ) {
        let sender = Transaction::sender();
        Transaction::assert(sender == 0xA550C18, 33);

        let association_auth_key = LibraAccount::authentication_key(sender);
        let sequence_number = LibraAccount::sequence_number(sender);

        Transaction::assert(writeset_sequence_number >= sequence_number, 3);

        Transaction::assert(writeset_sequence_number == sequence_number, 11);
        Transaction::assert(
            Hash::sha3_256(writeset_public_key) == association_auth_key,
            2
        );
    }

    fun epilogue(writeset_payload: vector<u8>) acquires T {
        let t_ref = borrow_global_mut<T>(0xA550C18);

        Event::emit_event<Self::UpgradeEvent>(
            &mut t_ref.upgrade_events,
            UpgradeEvent { writeset_payload },
        );
        LibraConfig::reconfigure();
    }
}

}
