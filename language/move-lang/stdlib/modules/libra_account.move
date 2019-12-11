address 0x0:

// The module for the account resource that governs every Libra account
module LibraAccount {
    use 0x0::LibraCoin;
    use 0x0::Hash;
    use 0x0::Event;
    use 0x0::Transaction;
    use 0x0::AddressUtil;

    // Every Libra account has a LibraAccount::T resource
    resource struct T {
        // The current authentication key.
        // This can be different than the key used to create the account
        authentication_key: bytearray,
        // The coins stored in this account
        balance: LibraCoin::T,
        // If true, the authority to rotate the authentication key of this account resides elsewhere
        delegated_key_rotation_capability: bool,
        // If true, the authority to withdraw funds from this account resides elsewhere
        delegated_withdrawal_capability: bool,
        // Event handle for received event
        received_events: Event::Handle<Self::ReceivedPaymentEvent>,
        // Event handle for sent event
        sent_events: Event::Handle<Self::SentPaymentEvent>,
        // The current sequence number.
        // Incremented by one each time a transaction is submitted
        sequence_number: u64,
    }

    // The holder of WithdrawalCapability for account_address can withdraw Libra from
    // account_address/LibraAccount::T/balance.
    // There is at most one WithdrawalCapability in existence for a given address.
    resource struct WithdrawalCapability { account_address: address }

    // The holder of KeyRotationCapability for account_address can rotate the authentication key for
    // account_address (i.e., write to account_address/LibraAccount::T/authentication_key).
    // There is at most one KeyRotationCapability in existence for a given address.
    resource struct KeyRotationCapability { account_address: address }

    // Message for sent events
    struct SentPaymentEvent {
        // The amount of LibraCoin::T sent
        amount: u64,
        // The address that was paid
        payee: address,
    }

    // Message for received events
    struct ReceivedPaymentEvent {
        // The amount of LibraCoin::T received
        amount: u64,
        // The address that sent the coin
        payer: address,
    }

    // Creates a new account at `fresh_address` with the `initial_balance` deducted from the
    // transaction sender's account
    public create_new_account(fresh_address: address, initial_balance: u64) acquires T {
        let account = T {
            authentication_key: AddressUtil::address_to_bytes(fresh_address),
            balance: LibraCoin::zero(),
            delegated_key_rotation_capability: false,
            delegated_withdrawal_capability: false,
            received_events: Event::new_event_handle(),
            sent_events: Event::new_event_handle(),
            sequence_number: 0,
        };
        save_account(fresh_address, account);

        if (initial_balance > 0) pay_from_sender(fresh_address, initial_balance);
    }

    // Save an account to a given address if the address does not have an account resource yet
    native save_account(addr: address, account: Self::T);

    // Deposits the `to_deposit` coin into the `payee`'s account
    public deposit(payee: address, to_deposit: LibraCoin::T) acquires T {
        // Check that the `to_deposit` coin is non-zero
        let amount = LibraCoin::value(&to_deposit);
        Transaction::assert(amount > 0, 7);

        let payer = Transaction::sender();
        Event::emit_event(
            &mut borrow_global_mut<T>(payer).sent_events,
            SentPaymentEvent { amount, payee },
        );

        let payee_account = borrow_global_mut<T>(payee);
        // Deposit the `to_deposit` coin
        LibraCoin::deposit(&mut payee_account.balance, to_deposit);
        // Log a received event
        Event::emit_event(
            &mut payee_account.received_events,
            ReceivedPaymentEvent { amount, payer }
        )
    }

    // mint_to_address can only be called by accounts with MintCapability (see LibraCoin)
    // and those account will be charged for gas. If those account don't have enough gas to pay
    // for the transaction cost they will fail minting.
    // However those account can also mint to themselves so that is a decent workaround
    public mint_to_address(payee: address, amount: u64) acquires T {
        // Create an account if it does not exist
        if (!exists(payee)) create_new_account(payee, 0);

        // Mint and deposit the coin
        deposit(payee, LibraCoin::mint_with_default_capability(amount));
    }

    // Withdraw `amount` LibraCoin::T from the transaction sender's account
    public withdraw_from_sender(amount: u64): LibraCoin::T acquires T {
        let sender_account = borrow_global_mut<T>(Transaction::sender());
        // Abort if sender has delegated the privilege to withdraw from her account elsewhere.
        if (sender_account.delegated_withdrawal_capability) abort 11;

        // The sender has retained her withdrawal privileges
        LibraCoin::withdraw(&mut sender_account.balance, amount)
    }

    // Withdraw `amount` LibraCoin::T from account under cap.account_address
    public withdraw_with_capability(
        cap: &WithdrawalCapability,
        amount: u64,
    ): LibraCoin::T
        acquires T
    {
        LibraCoin::withdraw(&mut borrow_global_mut<T>(cap.account_address).balance, amount)
    }

    // Return a unique capability granting permission to withdraw from the sender's account balance.
    public extract_sender_withdrawal_capability(): WithdrawalCapability acquires T {
        let account_address = Transaction::sender();
        let sender_account = borrow_global_mut<T>(account_address);
        // Abort if wWe already extracted the unique withdrawal capability for this account.
        if (sender_account.delegated_withdrawal_capability) abort 11;

        // ensure uniqueness of the capability
        sender_account.delegated_withdrawal_capability = true;
        WithdrawalCapability { account_address }
    }

    // Return the withdrawal capability to the account it originally came from
    public restore_withdrawal_capability(cap: WithdrawalCapability) acquires T {
        // Destroy the capability
        let WithdrawalCapability { account_address } = cap;
        // Update the flag for `account_address` to indicate that the capability has been restored.
        // The account owner will now be able to call pay_from_sender, withdraw_from_sender, and
        // extract_sender_withdrawal_capability again.
        borrow_global_mut<T>(account_address).delegated_withdrawal_capability = false
    }

    // Withdraw `amount` LibraCoin::T from the transaction sender's account and send the coin
    // to the `payee` address
    // Creates the `payee` account if it does not exist
    public pay_from_sender(payee: address, amount: u64) acquires T {
        if (exists(payee)) deposit(payee, withdraw_from_sender(amount))
        else create_new_account(payee, amount)
    }

    // Rotate the transaction sender's authentication key
    // The new key will be used for signing future transactions
    public rotate_authentication_key(new_authentication_key: bytearray) acquires T {
        let sender_account = borrow_global_mut<T>(Transaction::sender());
        // Abort if the sender has delegated the privilege to rotate her key elsewhere
        if (sender_account.delegated_key_rotation_capability) abort 11;

        // The sender has retained her key rotation privileges--proceed.
        sender_account.authentication_key = new_authentication_key
    }

    // Rotate the authentication key for the account under cap.account_address
    public rotate_authentication_key_with_capability(
        cap: &KeyRotationCapability,
        new_authentication_key: bytearray,
    ) acquires T  {
        borrow_global_mut<T>(cap.account_address).authentication_key = new_authentication_key
    }

    // Return a unique capability granting permission to rotate the sender's authentication key
    public extract_sender_key_rotation_capability(): KeyRotationCapability acquires T {
        let account_address = Transaction::sender();
        let sender_account = borrow_global_mut<T>(account_address);
        // Abort if we already extracted the unique key rotation capability for this account.
        if (sender_account.delegated_key_rotation_capability) abort 11;

        // ensure uniqueness of the capability
        sender_account.delegated_key_rotation_capability = true;
        KeyRotationCapability { account_address }
    }

    // Return the key rotation capability to the account it originally came from
    public restore_key_rotation_capability(cap: Self::KeyRotationCapability) acquires T {
        let KeyRotationCapability { account_address } = cap;
        // Update the flag for `account_address` to indicate that the capability has been restored.
        // The account owner will now be able to call rotate_authentication_key and
        // extract_sender_key_rotation_capability again
        borrow_global_mut<T>(account_address).delegated_key_rotation_capability = false
    }

    // Return the current balance of the LibraCoin::T in LibraAccount::T at `addr`
    public balance(addr: address): u64 acquires T {
        LibraCoin::value(&borrow_global<T>(addr).balance)
    }

    // Return the current sequence number at `addr`
    public sequence_number(addr: address): u64 acquires T {
        borrow_global<T>(addr).sequence_number
    }

    // Return true if the account at `addr` has delegated its key rotation capability
    public delegated_key_rotation_capability(addr: address): bool acquires T {
        borrow_global<T>(addr).delegated_key_rotation_capability
    }

    // Return true if the account at `addr` has delegated its withdrawal capability
    public delegated_withdrawal_capability(addr: address): bool acquires T {
        borrow_global<T>(addr).delegated_withdrawal_capability
    }


    // Return a reference to the address associated with the given withdrawal capability
    public withdrawal_capability_address(cap: &WithdrawalCapability): &address {
        &cap.account_address
    }

    // Return a reference to the address associated with the given key rotation capability
    public key_rotation_capability_address(cap: &KeyRotationCapability): &address {
        &cap.account_address
    }

    // Checks if an account exists at `check_addr`
    public exists(check_addr: address): bool {
        ::exists<T>(check_addr)
    }

    // The prologue is invoked at the beginning of every transaction
    // It verifies:
    // - The account's auth key matches the transaction's public key
    // - That the account has enough balance to pay for all of the gas
    // - That the sequence number matches the transaction's sequence key
    prologue() acquires T {
        let sender = Transaction::sender();

        // FUTURE: Make these error codes sequential
        // Verify that the transaction sender's account exists
        Transaction::assert(exists(sender), 5);

        // Load the transaction sender's account
        let sender_account = borrow_global_mut<T>(sender);

        // Check that the transaction's public key matches the account's current auth key

        Transaction::assert(
            &Hash::sha3_256(Transaction::public_key()) == &sender_account.authentication_key,
            2
        );

        // Check that the account has enough balance for all of the gas
        let max_transaction_fee = Transaction::gas_unit_price() * Transaction::max_gas_units();
        Transaction::assert(LibraCoin::value(&sender_account.balance) >= max_transaction_fee, 6);

        // Check that the transaction sequence number matches the sequence number of the account
        let transaction_sequence_number = Transaction::sequence_number();
        let sequence_number = sender_account.sequence_number;
        Transaction::assert(transaction_sequence_number >= sequence_number, 3);
        Transaction::assert(transaction_sequence_number == sequence_number, 4);
    }

    // The epilogue is invoked at the end of transactions.
    // It collects gas and bumps the sequence number
    epilogue() acquires T {
        // Load the transaction sender's account
        let sender_account = borrow_global_mut<T>(Transaction::sender());

        // Charge for gas
        let gas_price = Transaction::gas_unit_price();
        let starting_gas_units = Transaction::max_gas_units();
        let gas_units_remaining = Transaction::gas_remaining();
        let transaction_fee_amount = gas_price * (starting_gas_units - gas_units_remaining);

        Transaction::assert(LibraCoin::value(&sender_account.balance) >= transaction_fee_amount, 6);
        let transaction_fee = LibraCoin::withdraw(
            &mut sender_account.balance,
            transaction_fee_amount
        );

        // Bump the sequence number
        move sender_account.sequence_number = sender_account.sequence_number + 1;
        // Pay the transaction fee into the transaction fee pot
        LibraCoin::deposit(&mut borrow_global_mut<T>(0xFEE).balance, transaction_fee);
    }

}
