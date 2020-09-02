address 0x1 {

module LibraTransaction {
    use 0x1::ChainId;
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::LibraAccount;
    use 0x1::Event;
    use 0x1::Signer;
    use 0x1::LBR;
    use 0x1::LibraConfig;
    use 0x1::LibraTimestamp;
    use 0x1::LibraTransactionPublishingOption;

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

    const MAX_U64: u128 = 18446744073709551615;

    /// The `LibraWriteSetManager` was not in the required state
    const ELIBRA_WRITE_SET_MANAGER: u64 = 0;

    // The following codes need to be directly used in aborts as the VM expects them.
    const PROLOGUE_EINVALID_WRITESET_SENDER: u64 = 33;
    const PROLOGUE_ETRANSACTION_EXPIRED: u64 = 6;
    const PROLOGUE_EBAD_CHAIN_ID: u64 = 7;
    const PROLOGUE_ESCRIPT_NOT_ALLOWED: u64 = 8;
    const PROLOGUE_EMODULE_NOT_ALLOWED: u64 = 9;

    /// An invalid amount of gas units was provided for execution of the transaction
    const EGAS: u64 = 20;

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

    /// The prologue for module transaction
    fun module_prologue<Token>(
        sender: &signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
    ) {
        assert(
            LibraTransactionPublishingOption::is_module_allowed(sender),
            PROLOGUE_EMODULE_NOT_ALLOWED
        );

        prologue_common<Token>(
            sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units,
            txn_expiration_time,
            chain_id,
        )
    }

    /// The prologue for script transaction
    fun script_prologue<Token>(
        sender: &signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
        script_hash: vector<u8>,
    ) {
        assert(
            LibraTransactionPublishingOption::is_script_allowed(sender, &script_hash),
            PROLOGUE_ESCRIPT_NOT_ALLOWED
        );

        prologue_common<Token>(
            sender,
            txn_sequence_number,
            txn_public_key,
            txn_gas_price,
            txn_max_gas_units,
            txn_expiration_time,
            chain_id,
        )
    }

    /// The common prologue is invoked at the beginning of every transaction
    /// It verifies:
    /// - The account's auth key matches the transaction's public key
    /// - That the account has enough balance to pay for all of the gas
    /// - That the sequence number matches the transaction's sequence key
    fun prologue_common<Token>(
        sender: &signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time_seconds: u64,
        chain_id: u8,
    ) {
        // Check that the chain ID stored on-chain matches the chain ID specified by the transaction
        assert(ChainId::get() == chain_id, PROLOGUE_EBAD_CHAIN_ID);

        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        LibraAccount::check_sender<Token>(sender, max_transaction_fee, txn_sequence_number, txn_public_key);

        assert(
            LibraTimestamp::now_seconds() < txn_expiration_time_seconds,
            PROLOGUE_ETRANSACTION_EXPIRED
        );
    }

    fun writeset_prologue(
        account: &signer,
        writeset_sequence_number: u64,
        writeset_public_key: vector<u8>,
    ) {
        assert(sender == CoreAddresses::LIBRA_ROOT_ADDRESS(), PROLOGUE_EINVALID_WRITESET_SENDER);

        // Currency code don't matter here as it won't be charged anyway.
        LibraAccount::check_sender<LBR::LBR>(account, 0, writeset_sequence_number, writeset_public_key);
        // TODO: Check ChainID & Expiration Time?
    }

    /// The success_epilogue is invoked at the end of successfully executed transactions.
    fun success_epilogue<Token>(
        account: &signer,
        txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64
    ) {
        let sender = Signer::address_of(account);

        // Charge for gas
        assert(txn_max_gas_units >= gas_units_remaining, Errors::invalid_argument(EGAS));
        let gas_used = txn_max_gas_units - gas_units_remaining;
        assert((txn_gas_price as u128) * (gas_used as u128) <= MAX_U64, Errors::limit_exceeded(EGAS));
        let transaction_fee_amount = txn_gas_price * gas_used;

        LibraAccount::charge_transaction<Token>(sender, transaction_fee_amount, txn_sequence_number);
    }

    /// The failure_epilogue is invoked at the end of transactions when the transaction is aborted during execution or
    /// during `success_epilogue`.
    fun failure_epilogue<Token>(
        account: &signer,
        txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64
    ) {
        let sender = Signer::address_of(account);
        // Charge for gas
        assert(txn_max_gas_units >= gas_units_remaining, Errors::invalid_argument(EGAS));
        let gas_used = txn_max_gas_units - gas_units_remaining;
        assert((txn_gas_price as u128) * (gas_used as u128) <= MAX_U64, Errors::limit_exceeded(EGAS));
        let transaction_fee_amount = txn_gas_price * gas_used;

        LibraAccount::charge_transaction<Token>(sender, transaction_fee_amount, txn_sequence_number);
    }

    fun writeset_epilogue(
        lr_account: &signer,
        writeset_payload: vector<u8>,
        writeset_sequence_number: u64
    ) acquires LibraWriteSetManager {
        let t_ref = borrow_global_mut<LibraWriteSetManager>(CoreAddresses::LIBRA_ROOT_ADDRESS());

        Event::emit_event<Self::UpgradeEvent>(
            &mut t_ref.upgrade_events,
            UpgradeEvent { writeset_payload },
        );
        // Currency code don't matter here as it won't be charged anyway.
        LibraAccount::charge_transaction<LBR::LBR>(Signer::address_of(lr_account), 0, writeset_sequence_number);
        LibraConfig::reconfigure(lr_account)
    }
}

}
