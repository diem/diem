address 0x1 {

module LibraWriteSetManager {
    use 0x1::CoreErrors;
    use 0x1::CoreAddresses;
    use 0x1::LibraAccount;
    use 0x1::Event;
    use 0x1::Hash;
    use 0x1::Signer;
    use 0x1::LibraConfig;
    use 0x1::Roles;

    fun MODULE_ERROR_BASE(): u64 { 20000 }

    resource struct LibraWriteSetManager {
        upgrade_events: Event::EventHandle<Self::UpgradeEvent>,
    }

    struct UpgradeEvent {
        writeset_payload: vector<u8>,
    }

    public fun initialize(account: &signer) {
        // Operational constraint
        assert(
            Signer::address_of(account) == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(),
            MODULE_ERROR_BASE() + CoreErrors::INVALID_SINGLETON_ADDRESS()
        );

        move_to(
            account,
            LibraWriteSetManager {
                upgrade_events: Event::new_event_handle<Self::UpgradeEvent>(account),
            }
        );
    }

    // Called from the VM
    fun prologue(
        account: &signer,
        writeset_sequence_number: u64,
        writeset_public_key: vector<u8>,
    ) {
        let sender = Signer::address_of(account);
        assert(
            sender == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(),
            MODULE_ERROR_BASE() + CoreErrors::INSUFFICIENT_PRIVILEGE()
        );

        let association_auth_key = LibraAccount::authentication_key(sender);
        let sequence_number = LibraAccount::sequence_number(sender);

        assert(writeset_sequence_number >= sequence_number, CoreErrors::SEQ_NUM_TOO_OLD());

        assert(writeset_sequence_number == sequence_number, CoreErrors::SEQ_NUM_TOO_NEW());
        assert(
            Hash::sha3_256(writeset_public_key) == association_auth_key,
            MODULE_ERROR_BASE() + CoreErrors::INVALID_AUTH_KEY()
        );
    }

    // Called from the VM
    fun epilogue(account: &signer, writeset_payload: vector<u8>) acquires LibraWriteSetManager {
        let t_ref = borrow_global_mut<LibraWriteSetManager>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
        let association_root_capability = Roles::extract_privilege_to_capability(account);

        Event::emit_event<Self::UpgradeEvent>(
            &mut t_ref.upgrade_events,
            UpgradeEvent { writeset_payload },
        );
        LibraConfig::reconfigure(&association_root_capability);
        Roles::restore_capability_to_privilege(account, association_root_capability);
    }
}

}
