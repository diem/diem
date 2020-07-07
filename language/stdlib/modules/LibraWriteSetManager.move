address 0x1 {

module LibraWriteSetManager {
    use 0x1::CoreAddresses;
    use 0x1::LibraAccount;
    use 0x1::Event;
    use 0x1::Hash;
    use 0x1::Signer;
    use 0x1::LibraConfig;
    use 0x1::LibraTimestamp;

    resource struct LibraWriteSetManager {
        upgrade_events: Event::EventHandle<Self::UpgradeEvent>,
    }

    struct UpgradeEvent {
        writeset_payload: vector<u8>,
    }

    const ENOT_GENESIS: u64 = 0;
    const EINVALID_SINGLETON_ADDRESS: u64 = 1;
    const EINVALID_WRITESET_SENDER: u64 = 33;
    const EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY: u64 = 1;
    const EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD: u64 = 2;
    const EWS_PROLOGUE_SEQUENCE_NUMBER_TOO_NEW: u64 = 11;

    public fun initialize(account: &signer) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        // Operational constraint
        assert(Signer::address_of(account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);

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
        assert(sender == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_WRITESET_SENDER);

        let lr_auth_key = LibraAccount::authentication_key(sender);
        let sequence_number = LibraAccount::sequence_number(sender);

        assert(writeset_sequence_number >= sequence_number, EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD);

        assert(writeset_sequence_number == sequence_number, EWS_PROLOGUE_SEQUENCE_NUMBER_TOO_NEW);
        assert(
            Hash::sha3_256(writeset_public_key) == lr_auth_key,
            EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY
        );
    }

    fun epilogue(lr_account: &signer, writeset_payload: vector<u8>) acquires LibraWriteSetManager {
        let t_ref = borrow_global_mut<LibraWriteSetManager>(CoreAddresses::LIBRA_ROOT_ADDRESS());

        Event::emit_event<Self::UpgradeEvent>(
            &mut t_ref.upgrade_events,
            UpgradeEvent { writeset_payload },
        );
        LibraConfig::reconfigure(lr_account)
    }
}

}
