address 0x0:

// The module for the account resource that governs every Libra account
module LibraAccount {
    use 0x0::LibraCoin;
    use 0x0::Hash;
    use 0x0::U64Util;
    use 0x0::AddressUtil;
    use 0x0::BytearrayUtil;
    use 0x0::LibraTransactionTimeout;
    use 0x0::Transaction;

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
        received_events: EventHandle<ReceivedPaymentEvent>,
        // Event handle for sent event
        sent_events: EventHandle<SentPaymentEvent>,
        // The current sequence number.
        // Incremented by one each time a transaction is submitted
        sequence_number: u64,
        // Generator for event handles
        event_generator: EventHandleGenerator,
    }

    // The holder of WithdrawalCapability for account_address can withdraw Libra from
    // account_address/LibraAccount::T/balance.
    // There is at most one WithdrawalCapability in existence for a given address.
    resource struct WithdrawalCapability {
        account_address: address,
    }

    // The holder of KeyRotationCapability for account_address can rotate the authentication key for
    // account_address (i.e., write to account_address/LibraAccount::T/authentication_key).
    // There is at most one KeyRotationCapability in existence for a given address.
    resource struct KeyRotationCapability {
        account_address: address,
    }

    // Message for sent events
    struct SentPaymentEvent {
        // The amount of LibraCoin::T sent
        amount: u64,
        // The address that was paid
        payee: address,
        // Metadata associated with the payment
        metadata: bytearray,
    }

    // Message for received events
    struct ReceivedPaymentEvent {
        // The amount of LibraCoin::T received
        amount: u64,
        // The address that sent the coin
        payer: address,
        // Metadata associated with the payment
        metadata: bytearray,
    }

    /// Events
    // A resource representing the counter used to generate uniqueness under each account. There won't be destructor for
    // this resource to guarantee the uniqueness of the generated handle.
    resource struct EventHandleGenerator {
        // A monotonically increasing counter
        counter: u64,
    }

    // A handle for an event such that:
    // 1. Other modules can emit events to this handle.
    // 2. Storage can use this handle to prove the total number of events that happened in the past.
    resource struct EventHandle<T: copyable> {
        // Total number of events emitted to this event stream.
        counter: u64,
        // A globally unique ID for this event stream.
        guid: bytearray,
    }

    // Deposits the `to_deposit` coin into the `payee`'s account
    public fun deposit(payee: address, to_deposit: LibraCoin::T) acquires T {
        // Since we don't have bytearray literals in the source language at
        // the moment.
        // FIXME: Update this once we have bytearray literals
        deposit_with_metadata(payee, to_deposit, U64Util::u64_to_bytes(0));
    }

    // Deposits the `to_deposit` coin into the `payee`'s account with the attached `metadata`
    public fun deposit_with_metadata(
        payee: address,
        to_deposit: LibraCoin::T,
        metadata: bytearray
    ) acquires T {
        deposit_with_sender_and_metadata(
            payee,
            Transaction::sender(),
            to_deposit,
            metadata
        );
    }

    // Deposits the `to_deposit` coin into the `payee`'s account with the attached `metadata` and
    // sender address
    fun deposit_with_sender_and_metadata(
        payee: address,
        sender: address,
        to_deposit: LibraCoin::T,
        metadata: bytearray
    ) acquires T {
        // Check that the `to_deposit` coin is non-zero
        let deposit_value = LibraCoin::value(&to_deposit);
        Transaction::assert(deposit_value > 0, 7);

        // Load the sender's account
        let sender_account_ref = borrow_global_mut<T>(sender);
        // Log a sent event
        emit_event<SentPaymentEvent>(
            &mut sender_account_ref.sent_events,
            SentPaymentEvent {
                amount: deposit_value,
                payee: payee,
                metadata: metadata
            },
        );

        // Load the payee's account
        let payee_account_ref = borrow_global_mut<T>(payee);
        // Deposit the `to_deposit` coin
        LibraCoin::deposit(&mut payee_account_ref.balance, to_deposit);
        // Log a received event
        emit_event<ReceivedPaymentEvent>(
            &mut payee_account_ref.received_events,
            ReceivedPaymentEvent {
                amount: deposit_value,
                payer: sender,
                metadata: metadata
            }
        );
    }

    // mint_to_address can only be called by accounts with MintCapability (see LibraCoin)
    // and those account will be charged for gas. If those account don't have enough gas to pay
    // for the transaction cost they will fail minting.
    // However those account can also mint to themselves so that is a decent workaround
    public fun mint_to_address(payee: address, amount: u64) acquires T {
        // Create an account if it does not exist
        if (!exists(payee)) {
            create_account(payee);
        };

        // Mint and deposit the coin
        deposit(payee, LibraCoin::mint_with_default_capability(amount));
    }

    // Helper to withdraw `amount` from the given `account` and return the resulting LibraCoin::T
    fun withdraw_from_account(account: &mut T, amount: u64): LibraCoin::T {
        LibraCoin::withdraw(&mut account.balance, amount)
    }

    // Withdraw `amount` LibraCoin::T from the transaction sender's account
    public fun withdraw_from_sender(amount: u64): LibraCoin::T acquires T {
        let sender_account = borrow_global_mut<T>(Transaction::sender());
        // The sender has delegated the privilege to withdraw from her account elsewhere--abort.
        Transaction::assert(!sender_account.delegated_withdrawal_capability, 11);
        // The sender has retained her withdrawal privileges--proceed.
        withdraw_from_account(sender_account, amount)
    }

    // Withdraw `amount` LibraCoin::T from account under cap.account_address
    public fun withdraw_with_capability(
        cap: &WithdrawalCapability, amount: u64
    ): LibraCoin::T acquires T  {
        let account = borrow_global_mut<T>(cap.account_address);
        withdraw_from_account(account, amount)
    }

    // Return a unique capability granting permission to withdraw from the sender's account balance.
    public fun extract_sender_withdrawal_capability(): WithdrawalCapability acquires T {
        let sender = Transaction::sender();
        let sender_account = borrow_global_mut<T>(sender);

        // Abort if we already extracted the unique withdrawal capability for this account.
        Transaction::assert(!sender_account.delegated_withdrawal_capability, 11);

        // Ensure the uniqueness of the capability
        sender_account.delegated_withdrawal_capability = true;
        WithdrawalCapability { account_address: sender }
    }

    // Return the withdrawal capability to the account it originally came from
    public fun restore_withdrawal_capability(cap: WithdrawalCapability) acquires T {
        // Destroy the capability
        let WithdrawalCapability { account_address } = cap;
        let account = borrow_global_mut<T>(account_address);
        // Update the flag for `account_address` to indicate that the capability has been restored.
        // The account owner will now be able to call pay_from_sender, withdraw_from_sender, and
        // extract_sender_withdrawal_capability again.
        account.delegated_withdrawal_capability = false;
    }

    // Withdraws `amount` LibraCoin::T using the passed in WithdrawalCapability, and deposits it
    // into the `payee`'s account. Creates the `payee` account if it doesn't exist.
    public fun pay_from_capability(
        payee: address,
        cap: &WithdrawalCapability,
        amount: u64,
        metadata: bytearray
    ) acquires T {
        if (!exists(payee)) {
            create_account(payee);
        };
        deposit_with_sender_and_metadata(
            payee,
            *&cap.account_address,
            withdraw_with_capability(cap, amount),
            metadata,
        );
    }

    // Withdraw `amount` LibraCoin::T from the transaction sender's account and send the coin
    // to the `payee` address with the attached `metadata`
    // Creates the `payee` account if it does not exist
    public fun pay_from_sender_with_metadata(
        payee: address,
        amount: u64,
        metadata: bytearray
    ) acquires T {
        if (!exists(payee)) create_account(payee);
        deposit_with_metadata(
            payee,
            withdraw_from_sender(amount),
            metadata
        );
    }

    // Withdraw `amount` LibraCoin::T from the transaction sender's account and send the coin
    // to the `payee` address
    // Creates the `payee` account if it does not exist
    public fun pay_from_sender(payee: address, amount: u64) acquires T {
        // FIXME: Update this once we have bytearray literals
        pay_from_sender_with_metadata(payee, amount, U64Util::u64_to_bytes(0));
    }

    fun rotate_authentication_key_for_account(account: &mut T, new_authentication_key: bytearray) {
      account.authentication_key = new_authentication_key;
    }

    // Rotate the transaction sender's authentication key
    // The new key will be used for signing future transactions
    public fun rotate_authentication_key(new_authentication_key: bytearray) acquires T {
        let sender_account = borrow_global_mut<T>(Transaction::sender());
        // The sender has delegated the privilege to rotate her key elsewhere--abort
        Transaction::assert(!sender_account.delegated_key_rotation_capability, 11);
        // The sender has retained her key rotation privileges--proceed.
        rotate_authentication_key_for_account(
            sender_account,
            new_authentication_key
        );
    }

    // Rotate the authentication key for the account under cap.account_address
    public fun rotate_authentication_key_with_capability(
        cap: &KeyRotationCapability,
        new_authentication_key: bytearray,
    ) acquires T  {
        rotate_authentication_key_for_account(
            borrow_global_mut<T>(*&cap.account_address),
            new_authentication_key
        );
    }

    // Return a unique capability granting permission to rotate the sender's authentication key
    public fun extract_sender_key_rotation_capability(): KeyRotationCapability acquires T {
        let sender = Transaction::sender();
        let sender_account = borrow_global_mut<T>(sender);
        // Abort if we already extracted the unique key rotation capability for this account.
        Transaction::assert(!sender_account.delegated_key_rotation_capability, 11);
        sender_account.delegated_key_rotation_capability = true; // Ensure uniqueness of the capability
        KeyRotationCapability { account_address: sender }
    }

    // Return the key rotation capability to the account it originally came from
    public fun restore_key_rotation_capability(cap: KeyRotationCapability) acquires T {
        // Destroy the capability
        let KeyRotationCapability { account_address } = cap;
        let account = borrow_global_mut<T>(account_address);
        // Update the flag for `account_address` to indicate that the capability has been restored.
        // The account owner will now be able to call rotate_authentication_key and
        // extract_sender_key_rotation_capability again
        account.delegated_key_rotation_capability = false;
    }

    // Creates a new account at `fresh_address` with an initial balance of zero
    // Creating an account at 0x0 will cause runtime failure as it is a
    // reserved address for the MoveVM.
    public fun create_account(fresh_address: address) {
        let generator = EventHandleGenerator {counter: 0};
        save_account(
            fresh_address,
            T {
                authentication_key: AddressUtil::address_to_bytes(fresh_address),
                balance: LibraCoin::zero(),
                delegated_key_rotation_capability: false,
                delegated_withdrawal_capability: false,
                received_events: new_event_handle_impl<ReceivedPaymentEvent>(&mut generator, fresh_address),
                sent_events: new_event_handle_impl<SentPaymentEvent>(&mut generator, fresh_address),
                sequence_number: 0,
                event_generator: generator,
            }
        );
    }

    // Creates a new account at `fresh_address` with the `initial_balance` deducted from the
    // transaction sender's account
    public fun create_new_account(fresh_address: address, initial_balance: u64) acquires T {
        create_account(fresh_address);
        if (initial_balance > 0) {
            pay_from_sender(fresh_address, initial_balance);
        }
    }

    // Save an account to a given address if the address does not have an account resource yet
    native fun save_account(addr: address, account: Self::T);

    // Helper to return u64 value of the `balance` field for given `account`
    fun balance_for_account(account: &T): u64 {
        LibraCoin::value(&account.balance)
    }

    // Return the current balance of the LibraCoin::T in LibraAccount::T at `addr`
    public fun balance(addr: address): u64 acquires T {
        balance_for_account(borrow_global<T>(addr))
    }

    // Helper to return the sequence number field for given `account`
    fun sequence_number_for_account(account: &T): u64 {
        account.sequence_number
    }

    // Return the current sequence number at `addr`
    public fun sequence_number(addr: address): u64 acquires T {
        sequence_number_for_account(borrow_global<T>(addr))
    }

   // Return true if the account at `addr` has delegated its key rotation capability
    public fun delegated_key_rotation_capability(addr: address): bool acquires T {
        borrow_global<T>(addr).delegated_key_rotation_capability
    }

    // Return true if the account at `addr` has delegated its withdrawal capability
    public fun delegated_withdrawal_capability(addr: address): bool acquires T {
        borrow_global<T>(addr).delegated_withdrawal_capability
    }


    // Return a reference to the address associated with the given withdrawal capability
    public fun withdrawal_capability_address(cap: &WithdrawalCapability): &address {
        &cap.account_address
    }

    // Return a reference to the address associated with the given key rotation capability
    public fun key_rotation_capability_address(cap: &KeyRotationCapability): &address {
        &cap.account_address
    }

    // Checks if an account exists at `check_addr`
    public fun exists(check_addr: address): bool {
        ::exists<T>(check_addr)
    }

    // The prologue is invoked at the beginning of every transaction
    // It verifies:
    // - The account's auth key matches the transaction's public key
    // - That the account has enough balance to pay for all of the gas
    // - That the sequence number matches the transaction's sequence key
    fun prologue(
        txn_sequence_number: u64,
        txn_public_key: bytearray,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
    ) acquires T {
        let transaction_sender = Transaction::sender();

        // FUTURE: Make these error codes sequential
        // Verify that the transaction sender's account exists
        Transaction::assert(exists(transaction_sender), 5);

        // Load the transaction sender's account
        let sender_account = borrow_global_mut<T>(transaction_sender);

        // Check that the hash of the transaction's public key matches the account's auth key
        Transaction::assert(
            Hash::sha3_256(txn_public_key) == sender_account.authentication_key,
            2
        );

        // Check that the account has enough balance for all of the gas
        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        let imm_sender_account = freeze(sender_account);
        let balance_amount = balance_for_account(imm_sender_account);
        Transaction::assert(balance_amount >= max_transaction_fee, 6);

        // Check that the transaction sequence number matches the sequence number of the account
        Transaction::assert(txn_sequence_number >= sender_account.sequence_number, 3);
        Transaction::assert(txn_sequence_number == sender_account.sequence_number, 4);
        Transaction::assert(LibraTransactionTimeout::is_valid_transaction_timestamp(txn_expiration_time), 7);
    }

    // The epilogue is invoked at the end of transactions.
    // It collects gas and bumps the sequence number
    fun epilogue(
        txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64
    ) acquires T {
        // Load the transaction sender's account
        let sender_account = borrow_global_mut<T>(Transaction::sender());

        // Charge for gas
        let transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);
        let imm_sender_account = sender_account;
        Transaction::assert(
            balance_for_account(imm_sender_account) >= transaction_fee_amount,
            6
        );
        let transaction_fee = withdraw_from_account(
                sender_account,
                transaction_fee_amount
            );

        // Bump the sequence number
        sender_account.sequence_number = txn_sequence_number + 1;
        // Pay the transaction fee into the transaction fee pot
        let transaction_fee_account = borrow_global_mut<T>(0xFEE);
        LibraCoin::deposit(&mut transaction_fee_account.balance, transaction_fee);
    }

    /// Events
    //
    // Derive a fresh unique id by using sender's EventHandleGenerator. The generated bytearray is indeed unique because it
    // was derived from the hash(sender's EventHandleGenerator || sender_address). This module guarantees that the
    // EventHandleGenerator is only going to be monotonically increased and there's no way to revert it or destroy it. Thus
    // such counter is going to give distinct value for each of the new event stream under each sender. And since we
    // hash it with the sender's address, the result is guaranteed to be globally unique.
    fun fresh_guid(counter: &mut EventHandleGenerator, sender: address): bytearray {
        let sender_bytes = AddressUtil::address_to_bytes(sender);

        let count_bytes = U64Util::u64_to_bytes(counter.counter);
        counter.counter = counter.counter + 1;

        // EventHandleGenerator goes first just in case we want to extend address in the future.
        BytearrayUtil::bytearray_concat(count_bytes, sender_bytes)
    }

    // Use EventHandleGenerator to generate a unique event handle that one can emit an event to.
    fun new_event_handle_impl<T: copyable>(counter: &mut EventHandleGenerator, sender: address): EventHandle<T> {
        EventHandle<T> {counter: 0, guid: fresh_guid(counter, sender)}
    }

    // Use sender's EventHandleGenerator to generate a unique event handle that one can emit an event to.
    public fun new_event_handle<E: copyable>(): EventHandle<E> acquires T {
        let sender_account_ref = borrow_global_mut<T>(Transaction::sender());
        new_event_handle_impl<E>(&mut sender_account_ref.event_generator, Transaction::sender())
    }

    // Emit an event with payload `msg` by using handle's key and counter. Will change the payload from bytearray to a
    // generic type parameter once we have generics.
    public fun emit_event<T: copyable>(handle_ref: &mut EventHandle<T>, msg: T) {
        let guid = handle_ref.guid;

        write_to_event_store<T>(guid, handle_ref.counter, msg);
        handle_ref.counter = handle_ref.counter + 1;
    }

    // Native procedure that writes to the actual event stream in Event store
    // This will replace the "native" portion of EmitEvent bytecode
    native fun write_to_event_store<T: copyable>(guid: bytearray, count: u64, msg: T);

    // Destroy a unique handle.
    public fun destroy_handle<T: copyable>(handle: EventHandle<T>) {
        EventHandle<T> { counter: _, guid: _ } = handle;
    }
}
