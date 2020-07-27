address 0x1 {

module LibraWriteSetManager {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::LibraAccount;
    use 0x1::Event;
    use 0x1::Hash;
    use 0x1::Signer;
    use 0x1::LibraConfig;
    use 0x1::LibraTimestamp;

    resource struct LibraWriteSetManager {
        upgrade_events: Event::EventHandle<Self::UpgradeEvent>,
    }

    spec module {
        invariant [global]
            LibraTimestamp::is_operating() ==> exists<LibraWriteSetManager>(CoreAddresses::LIBRA_ROOT_ADDRESS());
    }

    struct UpgradeEvent {
        writeset_payload: vector<u8>,
    }

    const ELIBRA_WRITE_SET_MANAGER: u64 = 0;

    // The following codes need to be directly used in aborts as the VM expects them.
    const EPROLOGUE_INVALID_WRITESET_SENDER: u64 = 33;
    const EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY: u64 = 1;
    const EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD: u64 = 2;
    const EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW: u64 = 11;

    public fun initialize(account: &signer) {
        LibraTimestamp::assert_genesis();
        // Operational constraint
        CoreAddresses::assert_libra_root(account);

        assert(
            !exists<LibraWriteSetManager>(CoreAddresses::LIBRA_ROOT_ADDRESS()),
            Errors::already_published(ELIBRA_WRITE_SET_MANAGER)
        );
        move_to(
            account,
            LibraWriteSetManager {
                upgrade_events: Event::new_event_handle<Self::UpgradeEvent>(account),
            }
        );
    }
    spec fun initialize {
        include LibraTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot;

        aborts_if exists<LibraWriteSetManager>(CoreAddresses::LIBRA_ROOT_ADDRESS()) with Errors::ALREADY_PUBLISHED;
    }

    fun prologue(
        account: &signer,
        writeset_sequence_number: u64,
        writeset_public_key: vector<u8>,
    ) {
        // The below code uses direct abort codes as per contract with VM.
        let sender = Signer::address_of(account);
        assert(sender == CoreAddresses::LIBRA_ROOT_ADDRESS(), EPROLOGUE_INVALID_WRITESET_SENDER);

        let lr_auth_key = LibraAccount::authentication_key(sender);
        let sequence_number = LibraAccount::sequence_number(sender);

        assert(writeset_sequence_number >= sequence_number, EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD);

        assert(writeset_sequence_number == sequence_number, EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW);
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
