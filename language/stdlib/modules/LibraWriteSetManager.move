address 0x1 {

module LibraWriteSetManager {
    use 0x1::CoreAddresses;
    use 0x1::LibraAccount;
    use 0x1::Event;
    use 0x1::Hash;
    use 0x1::Signer;
    use 0x1::LibraConfig;

    resource struct LibraWriteSetManager {
        upgrade_events: Event::EventHandle<Self::UpgradeEvent>,
    }

    struct UpgradeEvent {
        writeset_payload: vector<u8>,
    }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), 1);

        move_to(
            account,
            LibraWriteSetManager {
                upgrade_events: Event::new_event_handle<Self::UpgradeEvent>(account),
            }
        );
    }

    fun prologue(
        account: &signer,
        writeset_sequence_number: u64,
        writeset_public_key: vector<u8>,
    ) {
        let sender = Signer::address_of(account);
        assert(sender == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), 33);

        let association_auth_key = LibraAccount::authentication_key(sender);
        let sequence_number = LibraAccount::sequence_number(sender);

        assert(writeset_sequence_number >= sequence_number, 3);

        assert(writeset_sequence_number == sequence_number, 11);
        assert(
            Hash::sha3_256(writeset_public_key) == association_auth_key,
            2
        );
    }

    fun epilogue(account: &signer, writeset_payload: vector<u8>) acquires LibraWriteSetManager {
        let t_ref = borrow_global_mut<LibraWriteSetManager>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());

        Event::emit_event<Self::UpgradeEvent>(
            &mut t_ref.upgrade_events,
            UpgradeEvent { writeset_payload },
        );
        LibraConfig::reconfigure(account);
    }
}

}
