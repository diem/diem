address 0x0 {

// The module for the account resource that governs every Libra account
module LibraAccount {
    use 0x0::AccountLimits;
    use 0x0::Association;
    use 0x0::Coin1;
    use 0x0::Coin2;
    use 0x0::Empty;
    use 0x0::Event;
    use 0x0::Hash;
    use 0x0::LBR;
    use 0x0::LCS;
    use 0x0::Libra;
    use 0x0::LibraTimestamp;
    use 0x0::LibraTransactionTimeout;
    use 0x0::Signature;
    use 0x0::Signer;
    use 0x0::Testnet;
    use 0x0::Transaction;
    use 0x0::Unhosted;
    use 0x0::VASP;
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
        received_events: Event::EventHandle<ReceivedPaymentEvent>,
        // Event handle for sent event
        sent_events: Event::EventHandle<SentPaymentEvent>,
        // The current sequence number.
        // Incremented by one each time a transaction is submitted
        sequence_number: u64,
        is_frozen: bool,
    }

    // TODO: could also do T<AccountType> field inside LibraAccount::T (but makes revocation hard)
    // TODO: can revoking a role just swap it for Empty?
    // TODO: do we need separate scheme for revoking a role, or is freezing sufficient?
    resource struct Role<RoleData: copyable> { role_data: RoleData }

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
        // The code symbol for the currency that was sent
        currency_code: vector<u8>,
        // The address that was paid
        payee: address,
        // Metadata associated with the payment
        metadata: vector<u8>,
    }

    // Message for received events
    struct ReceivedPaymentEvent {
        // The amount of Libra::T<Token> received
        amount: u64,
        // The code symbol for the currency that was received
        currency_code: vector<u8>,
        // The address that sent the coin
        payer: address,
        // Metadata associated with the payment
        metadata: vector<u8>,
    }

    // A privilege to allow the freezing of accounts.
    struct FreezingPrivilege { }

    // Message for freeze account events
    struct FreezeAccountEvent {
        // The address that initiated freeze txn
        initiator_address: address,
        // The address that was frozen
        frozen_address: address,
    }


    // Message for freeze account events
    struct UnfreezeAccountEvent {
        // The address that initiated unfreeze txn
        initiator_address: address,
        // The address that was unfrozen
        unfrozen_address: address,
    }

    resource struct AccountOperationsCapability {
        limits_cap: AccountLimits::CallingCapability,
        freeze_event_handle: Event::EventHandle<FreezeAccountEvent>,
        unfreeze_event_handle: Event::EventHandle<UnfreezeAccountEvent>,
    }

    // Return true if `addr` is an account of type `AT`
    public fun is<RoleData: copyable>(addr: address): bool {
        ::exists<Role<RoleData>>(addr)
    }

    public fun is_unhosted(addr: address): bool {
        is<Unhosted::T>(addr)
    }

    public fun is_parent_vasp(addr: address): bool {
        is<VASP::ParentVASP>(addr)
    }

    public fun is_child_vasp(addr: address): bool {
        is<VASP::ChildVASP>(addr)
    }

    public fun is_vasp(addr: address): bool {
        is_parent_vasp(addr) || is_child_vasp(addr)
    }

    public fun parent_vasp_address(addr: address): address acquires Role {
        if (is_parent_vasp(addr)) {
            addr
        } else if (is_child_vasp(addr)) {
            let child_vasp = &borrow_global<Role<VASP::ChildVASP>>(addr).role_data;
            VASP::child_parent_address(child_vasp)
        } else { // wrong account type, abort
            abort(88)
        }
    }

    public fun compliance_public_key(addr: address): vector<u8> acquires Role {
        let parent_vasp = &borrow_global<Role<VASP::ParentVASP>>(addr).role_data;
        VASP::compliance_public_key(parent_vasp)
    }

    public fun expiration_date(addr: address): u64 acquires Role {
        let parent_vasp = &borrow_global<Role<VASP::ParentVASP>>(addr).role_data;
        VASP::expiration_date(parent_vasp)
    }

    public fun base_url(addr: address): vector<u8> acquires Role {
        let parent_vasp = &borrow_global<Role<VASP::ParentVASP>>(addr).role_data;
        VASP::base_url(parent_vasp)
    }

    public fun human_name(addr: address): vector<u8> acquires Role {
        let parent_vasp = &borrow_global<Role<VASP::ParentVASP>>(addr).role_data;
        VASP::human_name(parent_vasp)
    }

    public fun rotate_compliance_public_key(vasp: &signer, new_key: vector<u8>) acquires Role {
        let parent_vasp =
            &mut borrow_global_mut<Role<VASP::ParentVASP>>(Signer::address_of(vasp)).role_data;
        VASP::rotate_compliance_public_key(parent_vasp, new_key)
    }

    // TODO: temporary, remove when VASP account feature in E2E tests works
    public fun add_parent_vasp_role_from_association(
        association: &signer,
        addr: address,
        human_name: vector<u8>,
        base_url: vector<u8>,
        compliance_public_key: vector<u8>,
    ) {
        Transaction::assert(exists(addr), 0);
        Transaction::assert(Signer::address_of(association) == 0xA550C18, 0);
        let role_data =
            VASP::create_parent_vasp_credential(human_name, base_url, compliance_public_key);
        let account = create_signer(addr);
        move_to(&account, Role<VASP::ParentVASP> { role_data });
        VASP::register_vasp(addr);
        destroy_signer(account);
    }

    public fun initialize(association: &signer) {
        Transaction::assert(Signer::address_of(association) == 0xA550C18, 0);
        move_to(
            association,
            AccountOperationsCapability {
                limits_cap: AccountLimits::grant_calling_capability(),
                freeze_event_handle: Event::new_event_handle(association),
                unfreeze_event_handle: Event::new_event_handle(association),
            }
        );
    }

    // Deposits the `to_deposit` coin into the `payee`'s account balance
    public fun deposit<Token>(payee: address, to_deposit: Libra::T<Token>)
    acquires T, Balance, AccountOperationsCapability, Role {
        // Since we don't have vector<u8> literals in the source language at
        // the moment.
        deposit_with_metadata(payee, to_deposit, x"", x"")
    }

    // Deposits the `to_deposit` coin into the sender's account balance
    public fun deposit_to_sender<Token>(to_deposit: Libra::T<Token>)
    acquires T, Balance, AccountOperationsCapability, Role {
        deposit(Transaction::sender(), to_deposit)
    }

    // Deposits the `to_deposit` coin into the `payee`'s account balance with the attached `metadata`
    public fun deposit_with_metadata<Token>(
        payee: address,
        to_deposit: Libra::T<Token>,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires T, Balance, AccountOperationsCapability, Role {
        deposit_with_sender_and_metadata(
            payee,
            Transaction::sender(),
            to_deposit,
            metadata,
            metadata_signature
        );
    }

    // Deposits the `to_deposit` coin into the `payee`'s account balance with the attached `metadata` and
    // sender address
    fun deposit_with_sender_and_metadata<Token>(
        payee: address,
        sender: address,
        to_deposit: Libra::T<Token>,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires T, Balance, AccountOperationsCapability, Role {
        // Check that the `to_deposit` coin is non-zero
        let deposit_value = Libra::value(&to_deposit);
        Transaction::assert(deposit_value > 0, 7);

        // TODO: on-chain config for travel rule limit instead of hardcoded value
        // TODO: nail down details of limit (specified in LBR? is 1 LBR a milliLibra or microLibra?)
        let travel_rule_limit = 1000;
        // travel rule only applies for payments over a threshold
        let above_threshold =
            Libra::approx_lbr_for_value<Token>(deposit_value) >= travel_rule_limit;
        // travel rule only applies if the sender and recipient are both VASPs
        let both_vasps = is_vasp(sender) && is_vasp(payee);
        // Don't check the travel rule if we're on testnet and sender
        // doesn't specify a metadata signature
        let is_testnet_transfer = Testnet::is_testnet() && Vector::is_empty(&metadata_signature);
        if (!is_testnet_transfer &&
            above_threshold &&
            both_vasps &&
            // travel rule does not apply for intra-VASP transactions
            parent_vasp_address(sender) != parent_vasp_address(payee)
        ) {
            // sanity check of signature validity
            Transaction::assert(Vector::length(&metadata_signature) == 64, 9001);
            // message should be metadata | sender_address | amount | domain_separator
            // separator is the UTF8-encoded string @@$$LIBRA_ATTEST$$@@
            let domain_separator = x"404024244C494252415F41545445535424244040";
            let message = copy metadata;
            Vector::append(&mut message, LCS::to_bytes(&sender));
            Vector::append(&mut message, LCS::to_bytes(&deposit_value));
            Vector::append(&mut message, domain_separator);
            // cryptographic check of signature validity
            Transaction::assert(
                Signature::ed25519_verify(
                    metadata_signature,
                    compliance_public_key(payee),
                    message
                ),
                9002, // TODO: proper error code
            );
        };

        // Ensure that this deposit is compliant with the account limits on
        // this account.
        let _ = borrow_global<AccountOperationsCapability>(0xA550C18);
        /*Transaction::assert(
            AccountLimits::update_deposit_limits<Token>(
                deposit_value,
                payee,
                &borrow_global<AccountOperationsCapability>(0xA550C18).limits_cap
            ),
            9
        );*/

        // Get the code symbol for this currency
        let currency_code = Libra::currency_code<Token>();

        // Load the sender's account
        let sender_account_ref = borrow_global_mut<T>(sender);
        // Log a sent event
        Event::emit_event<SentPaymentEvent>(
            &mut sender_account_ref.sent_events,
            SentPaymentEvent {
                amount: deposit_value,
                currency_code: copy currency_code,
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
        Event::emit_event<ReceivedPaymentEvent>(
            &mut payee_account_ref.received_events,
            ReceivedPaymentEvent {
                amount: deposit_value,
                currency_code,
                payer: sender,
                metadata: metadata
            }
        );
    }

    // Create `amount` coins of type `Token` and send them to `payee`.
    // `mint_to_address` can only be called by accounts with Libra::MintCapability<Token> and with
    // Token=Coin1 or Token=Coin2. `mint_lbr_to_address` should be used for minting LBR
    public fun mint_to_address<Token>(
        payee: address,
        amount: u64
    ) acquires T, Balance, AccountOperationsCapability, Role {
        // Mint and deposit the coin
        deposit(payee, Libra::mint<Token>(amount));
    }

    // Create `amount` LBR and send them to `payee`.
    // `mint_lbr_to_address` can only be called by accounts with Libra::MintCapability<Coin1> and
    // Libra::MintCapability<Coin2>
    public fun mint_lbr_to_address(
        payee: address,
        amount: u64
    ) acquires T, Balance, AccountOperationsCapability, Role {
        // Mint and deposit the coin
        deposit(payee, LBR::mint(amount));
    }

    // Cancel the oldest burn request from `preburn_address` and return the funds.
    // Fails if the sender does not have a published MintCapability.
    public fun cancel_burn<Token>(
        preburn_address: address,
    ) acquires T, Balance, AccountOperationsCapability, Role {
        let to_return = Libra::cancel_burn<Token>(preburn_address);
        deposit(preburn_address, to_return)
    }

    // Helper to withdraw `amount` from the given account balance and return the withdrawn Libra::T<Token>
    fun withdraw_from_balance<Token>(
        _addr: address,
        balance: &mut Balance<Token>,
        amount: u64
    ): Libra::T<Token> acquires AccountOperationsCapability {
        // Make sure that this withdrawal is compliant with the limits on
        // the account.
        let _  = borrow_global<AccountOperationsCapability>(0xA550C18);
        /*let can_withdraw = AccountLimits::update_withdrawal_limits<Token>(
            amount,
            addr,
            &borrow_global<AccountOperationsCapability>(0xA550C18).limits_cap
        );
        Transaction::assert(can_withdraw, 11);*/
        Libra::withdraw(&mut balance.coin, amount)
    }

    // Withdraw `amount` Libra::T<Token> from the transaction sender's account balance
    public fun withdraw_from_sender<Token>(amount: u64): Libra::T<Token>
    acquires T, Balance, AccountOperationsCapability {
        let sender = Transaction::sender();
        let sender_account = borrow_global_mut<T>(sender);
        let sender_balance = borrow_global_mut<Balance<Token>>(sender);
        // The sender has delegated the privilege to withdraw from her account elsewhere--abort.
        Transaction::assert(!sender_account.delegated_withdrawal_capability, 11);
        // The sender has retained her withdrawal privileges--proceed.
        withdraw_from_balance<Token>(sender, sender_balance, amount)
    }

    // Withdraw `amount` Libra::T<Token> from the account under cap.account_address
    public fun withdraw_with_capability<Token>(
        cap: &WithdrawalCapability, amount: u64
    ): Libra::T<Token> acquires Balance, AccountOperationsCapability {
        let balance = borrow_global_mut<Balance<Token>>(cap.account_address);
        withdraw_from_balance<Token>(cap.account_address, balance , amount)
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
        cap: &WithdrawalCapability,
        amount: u64,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires T, Balance, AccountOperationsCapability, Role {
        deposit_with_sender_and_metadata<Token>(
            payee,
            *&cap.account_address,
            withdraw_with_capability(cap, amount),
            metadata,
            metadata_signature
        );
    }

    // Withdraw `amount` Libra::T<Token> from the transaction sender's
    // account balance and send the coin to the `payee` address with the
    // attached `metadata` Creates the `payee` account if it does not exist
    public fun pay_from_sender_with_metadata<Token>(
        payee: address,
        amount: u64,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires T, Balance, AccountOperationsCapability, Role {
        deposit_with_metadata<Token>(
            payee,
            withdraw_from_sender(amount),
            metadata,
            metadata_signature
        );
    }

    // Withdraw `amount` Libra::T<Token> from the transaction sender's
    // account balance  and send the coin to the `payee` address
    // Creates the `payee` account if it does not exist
    public fun pay_from_sender<Token>(
        payee: address,
        amount: u64
    ) acquires T, Balance, AccountOperationsCapability, Role {
        pay_from_sender_with_metadata<Token>(payee, amount, x"", x"");
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

    // Creates a new testnet account at `fresh_address` with a balance of
    // zero `Token` type coins, and authentication key `auth_key_prefix` | `fresh_address`.
    // Trying to create an account at address 0x0 will cause runtime failure as it is a
    // reserved address for the MoveVM.
    public fun create_testnet_account<Token>(
        new_account_address: address,
        auth_key_prefix: vector<u8>
    ) {
        Transaction::assert(Testnet::is_testnet(), 10042);
        let vasp_parent =
            VASP::create_parent_vasp_credential(
                // "testnet"
                x"746573746E6574",
                // "https://libra.org"
                x"68747470733A2F2F6C696272612E6F72672F",
                // An empty compliance key
                x"00000000000000000000000000000000"
            );
        let new_account = create_signer(new_account_address);
        Event::publish_generator(&new_account);
        make_account<Token, VASP::ParentVASP>(new_account, auth_key_prefix, vasp_parent)
    }

    /// Creates a new account with account type `Role` at `new_account_address` with a balance of
    /// zero in `Token` and authentication key `auth_key_prefix` | `fresh_address`.
    /// Aborts if there is already an account at `new_account_address`
    /// Creating an account at address 0x0 will abort as it is a reserved address for the MoveVM.
    fun make_account<Token, RoleData: copyable>(
        new_account: signer,
        auth_key_prefix: vector<u8>,
        role_data: RoleData,
    ) {
        // cannot create an account at the reserved address 0x0
        Transaction::assert(Signer::address_of(&new_account) != 0x0, 0);

        // (1) publish Account::T
        let authentication_key = auth_key_prefix;
        Vector::append(
            &mut authentication_key, LCS::to_bytes(Signer::borrow_address(&new_account))
        );
        Transaction::assert(Vector::length(&authentication_key) == 32, 12);
        move_to(
            &new_account,
            T {
                authentication_key,
                delegated_key_rotation_capability: false,
                delegated_withdrawal_capability: false,
                received_events: Event::new_event_handle<ReceivedPaymentEvent>(&new_account),
                sent_events: Event::new_event_handle<SentPaymentEvent>(&new_account),
                sequence_number: 0,
                is_frozen: false,
            }
        );
        // (2) publish Account::Role. it's the caller's job to publish resources with role-specific
        //     configuration such as AccountLimits before calling this
        move_to(&new_account, Role<RoleData> { role_data });
        // (3) publish Balance resource(s)
        move_to(&new_account, Balance<Token>{ coin: Libra::zero<Token>() });
        // (4) TODO: publish account limits?

        destroy_signer(new_account);
    }

    /// Create an account with the Empty role at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`
    // TODO: can we get rid of this? the main thing this does is create an account without an
    // EventGenerator resource (which is just needed to avoid circular dep issues in gensis)
    public fun create_genesis_account<Token>(
        new_account_address: address,
        auth_key_prefix: vector<u8>
    ) {
        Transaction::assert(LibraTimestamp::is_genesis(), 0);
        let new_account = create_signer(new_account_address);
        make_account<Token, Empty::T>(new_account, auth_key_prefix, Empty::create())
    }

    /// Create a treasury/compliance account at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`
    public fun create_treasury_compliance_account<Token>(
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        coin1_mint_cap: Libra::MintCapability<Coin1::T>,
        coin1_burn_cap: Libra::BurnCapability<Coin1::T>,
        coin2_mint_cap: Libra::MintCapability<Coin2::T>,
        coin2_burn_cap: Libra::BurnCapability<Coin2::T>,
    ) {
        Association::assert_sender_is_root();
        let new_account = create_signer(new_account_address);
        Association::grant_association_address(&new_account);
        Association::grant_privilege<FreezingPrivilege>(&new_account);
        Libra::publish_mint_capability<Coin1::T>(&new_account, coin1_mint_cap);
        Libra::publish_burn_capability<Coin1::T>(&new_account, coin1_burn_cap);
        Libra::publish_mint_capability<Coin2::T>(&new_account, coin2_mint_cap);
        Libra::publish_burn_capability<Coin2::T>(&new_account, coin2_burn_cap);

        // TODO: add Association or TreasuryCompliance role instead of using Empty?
        Event::publish_generator(&new_account);
        make_account<Token, Empty::T>(new_account, auth_key_prefix, Empty::create())
    }

    /// Create an account with the ParentVASP role at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`.
    public fun create_parent_vasp_account<Token>(
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
        base_url: vector<u8>,
        compliance_public_key: vector<u8>,
    ) {
        Association::assert_sender_is_association();
        let vasp_parent =
            VASP::create_parent_vasp_credential(human_name, base_url, compliance_public_key);
        let new_account = create_signer(new_account_address);
        Event::publish_generator(&new_account);
        make_account<Token, VASP::ParentVASP>(new_account, auth_key_prefix, vasp_parent);
        VASP::register_vasp(new_account_address)
    }

    /// Create an account with the ChildVASP role at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`. This account will be a child of `creator`, which
    /// must be a ParentVASP.
    public fun create_child_vasp_account<Token>(
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        creator: &signer
    ) {
        let child_vasp = VASP::create_child_vasp(creator);
        let new_account = create_signer(new_account_address);
        Event::publish_generator(&new_account);
        make_account<Token, VASP::ChildVASP>(new_account, auth_key_prefix, child_vasp)
    }

    public fun create_unhosted_account<Token>(
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        let unhosted = Unhosted::create();
        let new_account = create_signer(new_account_address);
        Event::publish_generator(&new_account);
        make_account<Token, Unhosted::T>(new_account, auth_key_prefix, unhosted)
    }

    native fun create_signer(addr: address): signer;
    native fun destroy_signer(sig: signer);

    // Helper to return the u64 value of the `balance` for `account`
    fun balance_for<Token>(balance: &Balance<Token>): u64 {
        Libra::value<Token>(&balance.coin)
    }

    // Return the current balance of the account at `addr`.
    public fun balance<Token>(addr: address): u64 acquires Balance {
        balance_for(borrow_global<Balance<Token>>(addr))
    }

    // Add a balance of `Token` type to the sending account.
    public fun add_currency<Token>() {
        move_to_sender(Balance<Token>{ coin: Libra::zero<Token>() })
    }

    // Return whether the account at `addr` accepts `Token` type coins
    public fun accepts_currency<Token>(addr: address): bool {
        ::exists<Balance<Token>>(addr)
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

    ///////////////////////////////////////////////////////////////////////////
    // Freezing
    ///////////////////////////////////////////////////////////////////////////

    // Freeze the account at `addr`.
    public fun freeze_account(addr: address)
    acquires T, AccountOperationsCapability {
        assert_can_freeze(Transaction::sender());
        // The root association account cannot be frozen
        Transaction::assert(addr != Association::root_address(), 14);
        borrow_global_mut<T>(addr).is_frozen = true;
        Event::emit_event<FreezeAccountEvent>(
            &mut borrow_global_mut<AccountOperationsCapability>(0xA550C18).freeze_event_handle,
            FreezeAccountEvent {
                initiator_address: Transaction::sender(),
                frozen_address: addr
            },
        );
    }

    // Unfreeze the account at `addr`.
    public fun unfreeze_account(addr: address)
    acquires T, AccountOperationsCapability {
        assert_can_freeze(Transaction::sender());
        borrow_global_mut<T>(addr).is_frozen = false;
        Event::emit_event<UnfreezeAccountEvent>(
            &mut borrow_global_mut<AccountOperationsCapability>(0xA550C18).unfreeze_event_handle,
            UnfreezeAccountEvent {
                initiator_address: Transaction::sender(),
                unfrozen_address: addr
            },
        );
    }

    // Returns if the account at `addr` is frozen.
    public fun account_is_frozen(addr: address): bool
    acquires T {
        borrow_global<T>(addr).is_frozen
     }

    fun assert_can_freeze(addr: address) {
        Transaction::assert(Association::has_privilege<FreezingPrivilege>(addr), 13);
    }

    // The prologue is invoked at the beginning of every transaction
    // It verifies:
    // - The account's auth key matches the transaction's public key
    // - That the account has enough balance to pay for all of the gas
    // - That the sequence number matches the transaction's sequence key
    fun prologue<Token>(
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

        Transaction::assert(!account_is_frozen(transaction_sender), 0);

        // Load the transaction sender's account
        let sender_account = borrow_global_mut<T>(transaction_sender);

        // Check that the hash of the transaction's public key matches the account's auth key
        Transaction::assert(
            Hash::sha3_256(txn_public_key) == *&sender_account.authentication_key,
            2
        );

        // Check that the account has enough balance for all of the gas
        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        let balance_amount = balance<Token>(transaction_sender);
        Transaction::assert(balance_amount >= max_transaction_fee, 6);

        // Check that the transaction sequence number matches the sequence number of the account
        Transaction::assert(txn_sequence_number >= sender_account.sequence_number, 3);
        Transaction::assert(txn_sequence_number == sender_account.sequence_number, 4);
        Transaction::assert(LibraTransactionTimeout::is_valid_transaction_timestamp(txn_expiration_time), 7);
    }

    // The epilogue is invoked at the end of transactions.
    // It collects gas and bumps the sequence number
    fun epilogue<Token>(
        txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64
    ) acquires T, Balance, AccountOperationsCapability {
        // Load the transaction sender's account and balance resources
        let sender_account = borrow_global_mut<T>(Transaction::sender());
        let sender_balance = borrow_global_mut<Balance<Token>>(Transaction::sender());

        // Charge for gas
        let transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);
        Transaction::assert(
            balance_for(sender_balance) >= transaction_fee_amount,
            6
        );
        // Bump the sequence number
        sender_account.sequence_number = txn_sequence_number + 1;

        if (transaction_fee_amount > 0) {
            let transaction_fee = withdraw_from_balance(
                    Transaction::sender(),
                    sender_balance,
                    transaction_fee_amount
            );
            // Pay the transaction fee into the transaction fee balance.
            // Don't use the account deposit in order to not emit a
            // sent/received payment event.
            let transaction_fee_balance = borrow_global_mut<Balance<Token>>(0xFEE);
            Libra::deposit(&mut transaction_fee_balance.coin, transaction_fee);
        }
    }
}

}
