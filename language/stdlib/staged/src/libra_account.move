address 0x0:

// The module for the account resource that governs every Libra account
module LibraAccount {
    use 0x0::Hash;
    use 0x0::LBR;
    use 0x0::LCS;
    use 0x0::Libra;
    use 0x0::LibraTransactionTimeout;
    use 0x0::Transaction;
    use 0x0::Vector;

    // Every Libra account has a LibraAccount::T resource
    resource struct T {
        // The current authentication key.
        // This can be different than the key used to create the account
        authentication_key: vector<u8>,
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

    // A resource that holds the coins stored in this account
    resource struct Balance<Token> {
        coin: Libra::T<Token>,
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
        // The amount of Libra::T<Token> sent
        amount: u64,
        // The address that was paid
        payee: address,
        // Metadata associated with the payment
        metadata: vector<u8>,
    }

    // Message for received events
    struct ReceivedPaymentEvent {
        // The amount of Libra::T<Token> received
        amount: u64,
        // The address that sent the coin
        payer: address,
        // Metadata associated with the payment
        metadata: vector<u8>,
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
        guid: vector<u8>,
    }

    // Deposits the `to_deposit` coin into the `payee`'s account balance
    public fun deposit<Token>(payee: address, to_deposit: Libra::T<Token>) acquires T, Balance {
        // Since we don't have vector<u8> literals in the source language at
        // the moment.
        deposit_with_metadata(payee, to_deposit, x"")
    }

    // Deposits the `to_deposit` coin into the sender's account balance
    public fun deposit_to_sender<Token>(to_deposit: Libra::T<Token>) acquires T, Balance {
        deposit(Transaction::sender(), to_deposit)
    }

    // Deposits the `to_deposit` coin into the `payee`'s account balance with the attached `metadata`
    public fun deposit_with_metadata<Token>(
        payee: address,
        to_deposit: Libra::T<Token>,
        metadata: vector<u8>
    ) acquires T, Balance {
        deposit_with_sender_and_metadata(
            payee,
            Transaction::sender(),
            to_deposit,
            metadata
        );
    }

    // Deposits the `to_deposit` coin into the `payee`'s account balance with the attached `metadata` and
    // sender address
    fun deposit_with_sender_and_metadata<Token>(
        payee: address,
        sender: address,
        to_deposit: Libra::T<Token>,
        metadata: vector<u8>
    ) acquires T, Balance {
        // Check that the `to_deposit` coin is non-zero
        let deposit_value = Libra::value(&to_deposit);
        Transaction::assert(deposit_value > 0, 7);

        // Load the sender's account
        let sender_account_ref = borrow_global_mut<T>(sender);
        // Log a sent event
        emit_event<SentPaymentEvent>(
            &mut sender_account_ref.sent_events,
            SentPaymentEvent {
                amount: deposit_value,
                payee: payee,
                metadata: *&metadata
            },
        );

        // Load the payee's account
        let payee_account_ref = borrow_global_mut<T>(payee);
        let payee_balance = borrow_global_mut<Balance<Token>>(payee);
        // Deposit the `to_deposit` coin
        Libra::deposit(&mut payee_balance.coin, to_deposit);
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

    // mint_to_address can only be called by accounts with MintCapability (see Libra)
    // and those accounts will be charged for gas. If those accounts don't have enough gas to pay
    // for the transaction cost they will fail minting.
    // However those account can also mint to themselves so that is a decent workaround
    public fun mint_to_address(
        payee: address,
        auth_key_prefix: vector<u8>,
        amount: u64
    ) acquires T, Balance {
        // Create an account if it does not exist
        if (!exists(payee)) {
            create_account(payee, auth_key_prefix);
        };

        // Mint and deposit the coin
        deposit(payee, Libra::mint<LBR::T>(amount));
    }

    // Cancel the oldest burn request from `preburn_address` and return the funds.
    // Fails if the sender does not have a published MintCapability.
    public fun cancel_burn<Token>(
        preburn_address: address,
    ) acquires T, Balance {
        let to_return = Libra::cancel_burn<Token>(preburn_address);
        deposit(preburn_address, to_return)
    }

    // Helper to withdraw `amount` from the given account balance and return the withdrawn Libra::T<Token>
    fun withdraw_from_balance<Token>(balance: &mut Balance<Token>, amount: u64): Libra::T<Token> {
        Libra::withdraw(&mut balance.coin, amount)
    }

    // Withdraw `amount` Libra::T<Token> from the transaction sender's account balance
    public fun withdraw_from_sender<Token>(amount: u64): Libra::T<Token> acquires T, Balance {
        let sender_account = borrow_global_mut<T>(Transaction::sender());
        let sender_balance = borrow_global_mut<Balance<Token>>(Transaction::sender());
        // The sender has delegated the privilege to withdraw from her account elsewhere--abort.
        Transaction::assert(!sender_account.delegated_withdrawal_capability, 11);
        // The sender has retained her withdrawal privileges--proceed.
        withdraw_from_balance<Token>(sender_balance, amount)
    }

    // Withdraw `amount` Libra::T<Token> from the account under cap.account_address
    public fun withdraw_with_capability<Token>(
        cap: &WithdrawalCapability, amount: u64
    ): Libra::T<Token> acquires Balance {
        let balance = borrow_global_mut<Balance<Token>>(cap.account_address);
        withdraw_from_balance<Token>(balance , amount)
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

    // Withdraws `amount` Libra::T<Token> using the passed in WithdrawalCapability, and deposits it
    // into the `payee`'s account balance. Creates the `payee` account if it doesn't exist.
    public fun pay_from_capability<Token>(
        payee: address,
        auth_key_prefix: vector<u8>,
        cap: &WithdrawalCapability,
        amount: u64,
        metadata: vector<u8>
    ) acquires T, Balance {
        if (!exists(payee)) {
            create_account(payee, auth_key_prefix);
        };
        deposit_with_sender_and_metadata<Token>(
            payee,
            *&cap.account_address,
            withdraw_with_capability(cap, amount),
            metadata,
        );
    }

    // Withdraw `amount` Libra::T<Token> from the transaction sender's
    // account balance and send the coin to the `payee` address with the
    // attached `metadata` Creates the `payee` account if it does not exist
    public fun pay_from_sender_with_metadata<Token>(
        payee: address,
        auth_key_prefix: vector<u8>,
        amount: u64,
        metadata: vector<u8>
    ) acquires T, Balance {
        if (!exists(payee)) {
            create_account(payee, auth_key_prefix);
        };
        deposit_with_metadata<Token>(
            payee,
            withdraw_from_sender(amount),
            metadata
        );
    }

    // Withdraw `amount` Libra::T<Token> from the transaction sender's
    // account balance  and send the coin to the `payee` address
    // Creates the `payee` account if it does not exist
    public fun pay_from_sender<Token>(
        payee: address,
        auth_key_prefix: vector<u8>,
        amount: u64
    ) acquires T, Balance {
        pay_from_sender_with_metadata<Token>(payee, auth_key_prefix, amount, x"");
    }

    fun rotate_authentication_key_for_account(account: &mut T, new_authentication_key: vector<u8>) {
      // Don't allow rotating to clearly invalid key
      Transaction::assert(Vector::length(&new_authentication_key) == 32, 12);
      account.authentication_key = new_authentication_key;
    }

    // Rotate the transaction sender's authentication key
    // The new key will be used for signing future transactions
    public fun rotate_authentication_key(new_authentication_key: vector<u8>) acquires T {
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
        new_authentication_key: vector<u8>,
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

    // Creates a new account at `fresh_address` with an initial balance of zero and authentication
    // key `auth_key_prefix` | `fresh_address`
    // Creating an account at address 0x0 will cause runtime failure as it is a
    // reserved address for the MoveVM.
    public fun create_account(fresh_address: address, auth_key_prefix: vector<u8>) {
        let generator = EventHandleGenerator {counter: 0};
        let authentication_key = auth_key_prefix;
        Vector::append(&mut authentication_key, LCS::to_bytes(&fresh_address));
        Transaction::assert(Vector::length(&authentication_key) == 32, 12);

        save_account(
            Balance{
                coin: Libra::zero<LBR::T>()
            },
            T {
                authentication_key,
                delegated_key_rotation_capability: false,
                delegated_withdrawal_capability: false,
                received_events: new_event_handle_impl<ReceivedPaymentEvent>(&mut generator, fresh_address),
                sent_events: new_event_handle_impl<SentPaymentEvent>(&mut generator, fresh_address),
                sequence_number: 0,
                event_generator: generator,
            },
            fresh_address,
        );
    }

    // Creates a new account at `fresh_address` with the `initial_balance` deducted from the
    // transaction sender's account
    public fun create_new_account<Token>(
        fresh_address: address,
        auth_key_prefix: vector<u8>,
        initial_balance: u64
    ) acquires T, Balance {
        create_account(fresh_address, auth_key_prefix);
        if (initial_balance > 0) {
            deposit_with_metadata(
                fresh_address,
                withdraw_from_sender<Token>(initial_balance),
                Vector::empty(),
            );
        }
    }

    // Save an account to a given address if the address does not have account resources yet
    native fun save_account<Token>(
        balance: Balance<Token>,
        account: Self::T,
        addr: address,
    );

    // Helper to return the u64 value of the `balance` for `account`
    fun balance_for<Token>(balance: &Balance<Token>): u64 {
        Libra::value<Token>(&balance.coin)
    }

    // Return the current balance of the account at `addr`.
    public fun balance<Token>(addr: address): u64 acquires Balance {
        balance_for(borrow_global<Balance<Token>>(addr))
    }

    // Helper to return the sequence number field for given `account`
    fun sequence_number_for_account(account: &T): u64 {
        account.sequence_number
    }

    // Return the current sequence number at `addr`
    public fun sequence_number(addr: address): u64 acquires T {
        sequence_number_for_account(borrow_global<T>(addr))
    }

    // Return the authentication key for this account
    public fun authentication_key(addr: address): vector<u8> acquires T {
        *&borrow_global<T>(addr).authentication_key
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
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
    ) acquires T, Balance {
        let transaction_sender = Transaction::sender();

        // FUTURE: Make these error codes sequential
        // Verify that the transaction sender's account exists
        Transaction::assert(exists(transaction_sender), 5);

        // Load the transaction sender's account
        let sender_account = borrow_global_mut<T>(transaction_sender);

        // Check that the hash of the transaction's public key matches the account's auth key
        Transaction::assert(
            Hash::sha3_256(txn_public_key) == *&sender_account.authentication_key,
            2
        );

        // Check that the account has enough balance for all of the gas
        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        let balance_amount = balance<LBR::T>(transaction_sender);
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
    ) acquires T, Balance {
        // Load the transaction sender's account and balance resources
        let sender_account = borrow_global_mut<T>(Transaction::sender());
        let sender_balance = borrow_global_mut<Balance<LBR::T>>(Transaction::sender());

        // Charge for gas
        let transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);
        Transaction::assert(
            balance_for(sender_balance) >= transaction_fee_amount,
            6
        );
        let transaction_fee = withdraw_from_balance(
                sender_balance,
                transaction_fee_amount
            );

        // Bump the sequence number
        sender_account.sequence_number = txn_sequence_number + 1;
        // Pay the transaction fee into the transaction fee balance
        let transaction_fee_balance = borrow_global_mut<Balance<LBR::T>>(0xFEE);
        Libra::deposit(&mut transaction_fee_balance.coin, transaction_fee);
    }

    /// Events
    //
    // Derive a fresh unique id by using sender's EventHandleGenerator. The generated vector<u8> is indeed unique because it
    // was derived from the hash(sender's EventHandleGenerator || sender_address). This module guarantees that the
    // EventHandleGenerator is only going to be monotonically increased and there's no way to revert it or destroy it. Thus
    // such counter is going to give distinct value for each of the new event stream under each sender. And since we
    // hash it with the sender's address, the result is guaranteed to be globally unique.
    fun fresh_guid(counter: &mut EventHandleGenerator, sender: address): vector<u8> {
        let sender_bytes = LCS::to_bytes(&sender);
        let count_bytes = LCS::to_bytes(&counter.counter);
        counter.counter = counter.counter + 1;

        // EventHandleGenerator goes first just in case we want to extend address in the future.
        Vector::append(&mut count_bytes, sender_bytes);

        count_bytes
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

    // Emit an event with payload `msg` by using handle's key and counter. Will change the payload from vector<u8> to a
    // generic type parameter once we have generics.
    public fun emit_event<T: copyable>(handle_ref: &mut EventHandle<T>, msg: T) {
        let guid = *&handle_ref.guid;

        write_to_event_store<T>(guid, handle_ref.counter, msg);
        handle_ref.counter = handle_ref.counter + 1;
    }

    // Native procedure that writes to the actual event stream in Event store
    // This will replace the "native" portion of EmitEvent bytecode
    native fun write_to_event_store<T: copyable>(guid: vector<u8>, count: u64, msg: T);

    // Destroy a unique handle.
    public fun destroy_handle<T: copyable>(handle: EventHandle<T>) {
        EventHandle<T> { counter: _, guid: _ } = handle;
    }
}
