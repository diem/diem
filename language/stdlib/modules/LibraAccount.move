address 0x1 {

// The module for the account resource that governs every Libra account
module LibraAccount {
    use 0x1::CoreAddresses;
    use 0x1::AccountLimits;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::Event::{Self, EventHandle};
    use 0x1::Hash;
    use 0x1::LBR::LBR;
    use 0x1::LCS;
    use 0x1::LibraTimestamp;
    use 0x1::LibraTransactionTimeout;
    use 0x1::Signature;
    use 0x1::Signer;
    use 0x1::SlidingNonce;
    use 0x1::TransactionFee;
    use 0x1::ValidatorConfig;
    use 0x1::VASP;
    use 0x1::Vector;
    use 0x1::DesignatedDealer;
    use 0x1::Libra::{Self, Libra};
    use 0x1::Option::{Self, Option};
    use 0x1::DualAttestationLimit;
    use 0x1::Roles::{Self, Capability, LibraRootRole, ParentVASPRole, TreasuryComplianceRole};
    use 0x1::SlidingNonce::CreateSlidingNonce;

    resource struct AccountFreezing {}
    resource struct AccountUnfreezing {}
    resource struct PublishModule {}

    // Every Libra account has a LibraAccount resource
    resource struct LibraAccount {
        // The current authentication key.
        // This can be different than the key used to create the account
        authentication_key: vector<u8>,
        // A `withdrawal_capability` allows whoever holds this capability
        // to withdraw from the account. At the time of account creation
        // this capability is stored in this option. It can later be
        // "extracted" from this field via `extract_withdraw_capability`,
        // and can also be restored via `restore_withdraw_capability`.
        withdrawal_capability: Option<WithdrawCapability>,
        // A `key_rotation_capability` allows whoever holds this capability
        // the ability to rotate the authentication key for the account. At
        // the time of account creation this capability is stored in this
        // option. It can later be "extracted" from this field via
        // `extract_key_rotation_capability`, and can also be restored via
        // `restore_key_rotation_capability`.
        key_rotation_capability: Option<KeyRotationCapability>,
        // Event handle for received event
        received_events: EventHandle<ReceivedPaymentEvent>,
        // Event handle for sent event
        sent_events: EventHandle<SentPaymentEvent>,
        // The current sequence number.
        // Incremented by one each time a transaction is submitted
        sequence_number: u64,
        // If true, the account cannot be used to send transactions or receiver funds
        is_frozen: bool,
    }

    // A resource that holds the coins stored in this account
    resource struct Balance<Token> {
        coin: Libra<Token>,
    }

    // The holder of WithdrawCapability for account_address can withdraw Libra from
    // account_address/LibraAccount/balance.
    // There is at most one WithdrawCapability in existence for a given address.
    resource struct WithdrawCapability {
        account_address: address,
    }

    // The holder of KeyRotationCapability for account_address can rotate the authentication key for
    // account_address (i.e., write to account_address/LibraAccount/authentication_key).
    // There is at most one KeyRotationCapability in existence for a given address.
    resource struct KeyRotationCapability {
        account_address: address,
    }

    resource struct AccountOperationsCapability {
        limits_cap: AccountLimits::CallingCapability,
        freeze_event_handle: EventHandle<FreezeAccountEvent>,
        unfreeze_event_handle: EventHandle<UnfreezeAccountEvent>,
    }

    // Message for sent events
    struct SentPaymentEvent {
        // The amount of Libra<Token> sent
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
        // The amount of Libra<Token> received
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

    /// Grants `AccountFreezing` and `AccountUnfreezing` privileges to the calling `account`.
    /// Aborts if the `account` does not have the correct role (association root).
    public fun grant_association_privileges(account: &signer) {
        // TODO: Need to also grant this to the core code address account.
        Roles::add_privilege_to_account_association_root_role(account, PublishModule{});
    }

    /// Grants `AccountFreezing` and `AccountUnfreezing` privileges to the calling `account`.
    /// Aborts if the `account` does not have the correct role (treasury compliance).
    public fun grant_treasury_compliance_privileges(account: &signer) {
        Roles::add_privilege_to_account_treasury_compliance_role(account, AccountFreezing{});
        Roles::add_privilege_to_account_treasury_compliance_role(account, AccountUnfreezing{});
    }

    public fun initialize(
        association: &signer,
        assoc_root_capability: &Capability<LibraRootRole>,
    ) {
        // Operational constraint, not a privilege constraint.
        assert(Signer::address_of(association) == CoreAddresses::LIBRA_ROOT_ADDRESS(), 0);
        move_to(
            association,
            AccountOperationsCapability {
                limits_cap: AccountLimits::grant_calling_capability(assoc_root_capability),
                freeze_event_handle: Event::new_event_handle(association),
                unfreeze_event_handle: Event::new_event_handle(association),
            }
        );
    }

    // Deposits the `to_deposit` coin into the `payee`'s account balance
    public fun deposit<Token>(payer: &signer, payee: address, to_deposit: Libra<Token>)
    acquires LibraAccount, Balance, AccountOperationsCapability {
        deposit_with_metadata(payer, payee, to_deposit, x"", x"")
    }

    // Deposits the `to_deposit` coin into `account`
    public fun deposit_to<Token>(account: &signer, to_deposit: Libra<Token>)
    acquires LibraAccount, Balance, AccountOperationsCapability {
        deposit(account, Signer::address_of(account), to_deposit)
    }

    // Deposits the `to_deposit` coin into the `payee`'s account balance with the attached `metadata`
    // We specifically do _not_ use a withdrawal capability here since the
    // funds need not be derived from an account withdrawal (e.g. minting),
    // or from a withdrawal from the same account that is depositing coins
    // (e.g. using a different withdrawal capability).
    public fun deposit_with_metadata<Token>(
        payer: &signer,
        payee: address,
        to_deposit: Libra<Token>,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        deposit_with_sender_and_metadata(
            payee,
            Signer::address_of(payer),
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
        to_deposit: Libra<Token>,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        // Check that the `to_deposit` coin is non-zero
        let deposit_value = Libra::value(&to_deposit);
        assert(deposit_value > 0, 7);
        let travel_rule_limit_microlibra = DualAttestationLimit::get_cur_microlibra_limit();
        // travel rule only applies for payments over a threshold
        let approx_lbr_microlibra_value = Libra::approx_lbr_for_value<Token>(deposit_value);
        let above_threshold = approx_lbr_microlibra_value >= travel_rule_limit_microlibra;
        // travel rule only applies if the sender and recipient are both VASPs
        let both_vasps = VASP::is_vasp(sender) && VASP::is_vasp(payee);
        if (above_threshold &&
            both_vasps &&
            // travel rule does not apply for intra-VASP transactions
            VASP::parent_address(sender) != VASP::parent_address(payee)
        ) {
            // sanity check of signature validity
            assert(Vector::length(&metadata_signature) == 64, 9001);
            // message should be metadata | sender_address | amount | domain_separator
            let domain_separator = b"@@$$LIBRA_ATTEST$$@@";
            let message = copy metadata;
            Vector::append(&mut message, LCS::to_bytes(&sender));
            Vector::append(&mut message, LCS::to_bytes(&deposit_value));
            Vector::append(&mut message, domain_separator);
            // cryptographic check of signature validity
            assert(
                Signature::ed25519_verify(
                    metadata_signature,
                    VASP::compliance_public_key(payee),
                    message
                ),
                9002, // TODO: proper error code
            );
        };

        // Ensure that this deposit is compliant with the account limits on
        // this account.
        let _ = borrow_global<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        /*assert(
            AccountLimits::update_deposit_limits<Token>(
                deposit_value,
                payee,
                &borrow_global<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()).limits_cap
            ),
            9
        );*/

        // Get the code symbol for this currency
        let currency_code = Libra::currency_code<Token>();

        // Load the sender's account
        let sender_account_ref = borrow_global_mut<LibraAccount>(sender);
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
        let payee_account_ref = borrow_global_mut<LibraAccount>(payee);
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
    // Token=Coin1 or Token=Coin2
    public fun mint_to_address<Token>(
        account: &signer,
        payee: address,
        amount: u64
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        // Mint and deposit the coin
        deposit(account, payee, Libra::mint<Token>(account, amount));
    }

    // Cancel the oldest burn request from `preburn_address` and return the funds.
    // Fails if the sender does not have a published MintCapability.
    public fun cancel_burn<Token>(
        account: &signer,
        preburn_address: address,
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        let to_return = Libra::cancel_burn<Token>(account, preburn_address);
        deposit(account, preburn_address, to_return)
    }

    // Helper to withdraw `amount` from the given account balance and return the withdrawn Libra<Token>
    fun withdraw_from_balance<Token>(
        _addr: address,
        balance: &mut Balance<Token>,
        amount: u64
    ): Libra<Token> acquires AccountOperationsCapability {
        // Make sure that this withdrawal is compliant with the limits on
        // the account.
        let _  = borrow_global<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS());
        /*let can_withdraw = AccountLimits::update_withdrawal_limits<Token>(
            amount,
            addr,
            &borrow_global<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()).limits_cap
        );
        assert(can_withdraw, 11);*/
        Libra::withdraw(&mut balance.coin, amount)
    }

    // Withdraw `amount` Libra<Token> from the account balance under
    // `cap.account_address`
    public fun withdraw_from<Token>(cap: &WithdrawCapability, amount: u64): Libra<Token>
    acquires Balance, AccountOperationsCapability {
        let account_balance = borrow_global_mut<Balance<Token>>(cap.account_address);
        // The sender has retained her withdraw privileges--proceed.
        withdraw_from_balance<Token>(cap.account_address, account_balance, amount)
    }

    // Return a unique capability granting permission to withdraw from the sender's account balance.
    public fun extract_withdraw_capability(
        sender: &signer
    ): WithdrawCapability acquires LibraAccount {
        let sender_addr = Signer::address_of(sender);
        // Abort if we already extracted the unique withdraw capability for this account.
        assert(!delegated_withdraw_capability(sender_addr), 11);
        let account = borrow_global_mut<LibraAccount>(sender_addr);
        Option::extract(&mut account.withdrawal_capability)
    }

    // Return the withdraw capability to the account it originally came from
    public fun restore_withdraw_capability(cap: WithdrawCapability)
    acquires LibraAccount {
        let account = borrow_global_mut<LibraAccount>(cap.account_address);
        Option::fill(&mut account.withdrawal_capability, cap)
    }

    // Withdraws `amount` Libra<Token> using the passed in WithdrawCapability, and deposits it
    // into the `payee`'s account balance. Creates the `payee` account if it doesn't exist.
    public fun pay_from_with_metadata<Token>(
        cap: &WithdrawCapability,
        payee: address,
        amount: u64,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        deposit_with_sender_and_metadata<Token>(
            payee,
            *&cap.account_address,
            withdraw_from(cap, amount),
            metadata,
            metadata_signature
        );
    }

    // Withdraw `amount` Libra<Token> from the transaction sender's
    // account balance  and send the coin to the `payee` address
    // Creates the `payee` account if it does not exist
    public fun pay_from<Token>(withdraw_cap: &WithdrawCapability, payee: address, amount: u64)
    acquires LibraAccount, Balance, AccountOperationsCapability {
        pay_from_with_metadata<Token>(withdraw_cap, payee, amount, x"", x"");
    }

    // Rotate the authentication key for the account under cap.account_address
    public fun rotate_authentication_key(
        cap: &KeyRotationCapability,
        new_authentication_key: vector<u8>,
    ) acquires LibraAccount  {
        let sender_account_resource = borrow_global_mut<LibraAccount>(cap.account_address);
        // Don't allow rotating to clearly invalid key
        assert(Vector::length(&new_authentication_key) == 32, 12);
        sender_account_resource.authentication_key = new_authentication_key;
    }

    // Return a unique capability granting permission to rotate the sender's authentication key
    public fun extract_key_rotation_capability(account: &signer): KeyRotationCapability
    acquires LibraAccount {
        let account_address = Signer::address_of(account);
        // Abort if we already extracted the unique key rotation capability for this account.
        assert(!delegated_key_rotation_capability(account_address), 11);
        let account = borrow_global_mut<LibraAccount>(account_address);
        Option::extract(&mut account.key_rotation_capability)
    }

    // Return the key rotation capability to the account it originally came from
    public fun restore_key_rotation_capability(cap: KeyRotationCapability)
    acquires LibraAccount {
        let account = borrow_global_mut<LibraAccount>(cap.account_address);
        Option::fill(&mut account.key_rotation_capability, cap)
    }

    // TODO: get rid of this and just use normal VASP creation
    // Creates a new testnet account at `fresh_address` with a balance of
    // zero `Token` type coins, and authentication key `auth_key_prefix` | `fresh_address`.
    // Trying to create an account at address 0x0 will cause runtime failure as it is a
    // reserved address for the MoveVM.
    public fun create_testnet_account<Token>(
        creator_account: &signer,
        parent_vasp_creation_capability: &Capability<LibraRootRole>,
        new_account_address: address,
        auth_key_prefix: vector<u8>
    ) {
        // TODO: refactor so that every attempt to create an existing account hits this check
        // cannot create an account at an address that already has one
        assert(!exists_at(new_account_address), 777777);
        create_parent_vasp_account<Token>(
            creator_account,
            parent_vasp_creation_capability,
            new_account_address,
            auth_key_prefix,
            b"testnet",
            b"https://libra.org",
            // A bogus (but valid ed25519) compliance public key
            x"b7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde",
            false // all_all_currencies
        )
    }

    fun add_currencies_for_account<Token>(
        new_account: &signer,
        add_all_currencies: bool,
    ) {
        let new_account_addr = Signer::address_of(new_account);
        add_currency<Token>(new_account);
        if (add_all_currencies) {
            if (!exists<Balance<Coin1>>(new_account_addr)) {
                add_currency<Coin1>(new_account);
            };
            if (!exists<Balance<Coin2>>(new_account_addr)) {
                add_currency<Coin2>(new_account);
            };
            if (!exists<Balance<LBR>>(new_account_addr)) {
                add_currency<LBR>(new_account);
            };
        };
    }


    /// Creates a new account with account at `new_account_address` with a balance of
    /// zero in `Token` and authentication key `auth_key_prefix` | `fresh_address`. If
    /// `add_all_currencies` is true, 0 balances for all available currencies in the system will
    /// also be added.
    /// Aborts if there is already an account at `new_account_address`.
    /// Creating an account at address 0x0 will abort as it is a reserved address for the MoveVM.
    fun make_account(
        new_account: signer,
        auth_key_prefix: vector<u8>,
    ) {
        let new_account_addr = Signer::address_of(&new_account);
        // cannot create an account at the reserved address 0x0
        assert(new_account_addr != CoreAddresses::VM_RESERVED_ADDRESS(), 0);

        // (1) publish LibraAccount
        let authentication_key = auth_key_prefix;
        Vector::append(
            &mut authentication_key, LCS::to_bytes(Signer::borrow_address(&new_account))
        );
        assert(Vector::length(&authentication_key) == 32, 12);
        move_to(
            &new_account,
            LibraAccount {
                authentication_key,
                withdrawal_capability: Option::some(
                    WithdrawCapability {
                        account_address: new_account_addr
                }),
                key_rotation_capability: Option::some(
                    KeyRotationCapability {
                        account_address: new_account_addr
                }),
                received_events: Event::new_event_handle<ReceivedPaymentEvent>(&new_account),
                sent_events: Event::new_event_handle<SentPaymentEvent>(&new_account),
                sequence_number: 0,
                is_frozen: false,
            }
        );

        // (2) TODO: publish account limits?
        destroy_signer(new_account);
    }

    /// Creates the root association account in genesis.
    public fun create_root_association_account(
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        LibraTimestamp::assert_is_genesis();
        assert(new_account_address == CoreAddresses::LIBRA_ROOT_ADDRESS(), 0);
        let new_account = create_signer(new_account_address);
        make_account(new_account, auth_key_prefix)
    }

    /// Create a treasury/compliance account at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`
    public fun create_treasury_compliance_account(
        _: &Capability<LibraRootRole>,
        tc_capability: &Capability<TreasuryComplianceRole>,
        sliding_nonce_creation_capability: &Capability<CreateSlidingNonce>,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        coin1_mint_cap: Libra::MintCapability<Coin1>,
        coin1_burn_cap: Libra::BurnCapability<Coin1>,
        coin2_mint_cap: Libra::MintCapability<Coin2>,
        coin2_burn_cap: Libra::BurnCapability<Coin2>,
    ) {
        LibraTimestamp::assert_is_genesis();
        let new_account = create_signer(new_account_address);
        Libra::publish_mint_capability<Coin1>(&new_account, coin1_mint_cap, tc_capability);
        Libra::publish_burn_capability<Coin1>(&new_account, coin1_burn_cap, tc_capability);
        Libra::publish_mint_capability<Coin2>(&new_account, coin2_mint_cap, tc_capability);
        Libra::publish_burn_capability<Coin2>(&new_account, coin2_burn_cap, tc_capability);
        SlidingNonce::publish_nonce_resource(sliding_nonce_creation_capability, &new_account);
        Event::publish_generator(&new_account);
        make_account(new_account, auth_key_prefix)
    }


    ///////////////////////////////////////////////////////////////////////////
    // Designated Dealer API
    ///////////////////////////////////////////////////////////////////////////

    /// Create a designated dealer account at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`, for non synthetic CoinType.
    /// Creates Preburn resource under account 'new_account_address'
    public fun create_designated_dealer<CoinType>(
        creator_account: &signer,
        tc_capability: &Capability<TreasuryComplianceRole>,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        let new_dd_account = create_signer(new_account_address);
        Event::publish_generator(&new_dd_account);
        Libra::publish_preburn_to_account<CoinType>(&new_dd_account, tc_capability);
        DesignatedDealer::publish_designated_dealer_credential(&new_dd_account, tc_capability);
        Roles::new_designated_dealer_role(creator_account, &new_dd_account);
        add_currencies_for_account<CoinType>(&new_dd_account, false);
        make_account(new_dd_account, auth_key_prefix)
    }

    ///////////////////////////////////////////////////////////////////////////
    // VASP methods
    ///////////////////////////////////////////////////////////////////////////

    /// Create an account with the ParentVASP role at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`.  If `add_all_currencies` is true, 0 balances for
    /// all available currencies in the system will also be added.
    public fun create_parent_vasp_account<Token>(
        creator_account: &signer,
        parent_vasp_creation_capability: &Capability<LibraRootRole>,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
        base_url: vector<u8>,
        compliance_public_key: vector<u8>,
        add_all_currencies: bool
    ) {
        let new_account = create_signer(new_account_address);
        Roles::new_parent_vasp_role(creator_account, &new_account);
        VASP::publish_parent_vasp_credential(
            &new_account,
            parent_vasp_creation_capability,
            human_name,
            base_url,
            compliance_public_key
        );
        Event::publish_generator(&new_account);
        add_currencies_for_account<Token>(&new_account, add_all_currencies);
        make_account(new_account, auth_key_prefix)
    }

    /// Create an account with the ChildVASP role at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address` and a 0 balance of type `Token`. If
    /// `add_all_currencies` is true, 0 balances for all avaialable currencies in the system will
    /// also be added. This account will be a child of `creator`, which must be a ParentVASP.
    public fun create_child_vasp_account<Token>(
        parent: &signer,
        child_vasp_creation_capability: &Capability<ParentVASPRole>,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        add_all_currencies: bool,
    ) {
        let new_account = create_signer(new_account_address);
        Roles::new_child_vasp_role(parent, &new_account);
        VASP::publish_child_vasp_credential(
            parent,
            &new_account,
            child_vasp_creation_capability,
        );
        Event::publish_generator(&new_account);
        add_currencies_for_account<Token>(&new_account, add_all_currencies);
        make_account(new_account, auth_key_prefix)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Unhosted methods
    ///////////////////////////////////////////////////////////////////////////

    // For now, only the association root can create an unhosted account, and it will choose not to
    // on mainnet
    // > TODO(tzakian): eventually, anyone will be able to create an unhosted wallet accunt
    public fun create_unhosted_account<Token>(
        creator_account: &signer,
        _: &Capability<LibraRootRole>,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        add_all_currencies: bool
    ) {
        assert(!exists_at(new_account_address), 777777);
        let new_account = create_signer(new_account_address);
        Roles::new_unhosted_role(creator_account, &new_account);
        Event::publish_generator(&new_account);
        add_currencies_for_account<Token>(&new_account, add_all_currencies);
        make_account(new_account, auth_key_prefix)
    }

    ///////////////////////////////////////////////////////////////////////////
    // General purpose methods
    ///////////////////////////////////////////////////////////////////////////

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
    public fun add_currency<Token>(account: &signer) {
        move_to(account, Balance<Token>{ coin: Libra::zero<Token>() })
    }

    // Return whether the account at `addr` accepts `Token` type coins
    public fun accepts_currency<Token>(addr: address): bool {
        exists<Balance<Token>>(addr)
    }

    // Helper to return the sequence number field for given `account`
    fun sequence_number_for_account(account: &LibraAccount): u64 {
        account.sequence_number
    }

    // Return the current sequence number at `addr`
    public fun sequence_number(addr: address): u64 acquires LibraAccount {
        sequence_number_for_account(borrow_global<LibraAccount>(addr))
    }

    // Return the authentication key for this account
    public fun authentication_key(addr: address): vector<u8> acquires LibraAccount {
        *&borrow_global<LibraAccount>(addr).authentication_key
    }

    // Return true if the account at `addr` has delegated its key rotation capability
    public fun delegated_key_rotation_capability(addr: address): bool
    acquires LibraAccount {
        Option::is_none(&borrow_global<LibraAccount>(addr).key_rotation_capability)
    }

    // Return true if the account at `addr` has delegated its withdraw capability
    public fun delegated_withdraw_capability(addr: address): bool
    acquires LibraAccount {
        Option::is_none(&borrow_global<LibraAccount>(addr).withdrawal_capability)
    }

    // Return a reference to the address associated with the given withdraw capability
    public fun withdraw_capability_address(cap: &WithdrawCapability): &address {
        &cap.account_address
    }

    // Return a reference to the address associated with the given key rotation capability
    public fun key_rotation_capability_address(cap: &KeyRotationCapability): &address {
        &cap.account_address
    }

    // Checks if an account exists at `check_addr`
    public fun exists_at(check_addr: address): bool {
        exists<LibraAccount>(check_addr)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Freezing
    ///////////////////////////////////////////////////////////////////////////

    // Freeze the account at `addr`.
    public fun freeze_account(
        account: &signer,
        _freezing_capability: &Capability<AccountFreezing>,
        frozen_address: address,
    )
    acquires LibraAccount, AccountOperationsCapability {
        let initiator_address = Signer::address_of(account);
        // The root association account cannot be frozen
        assert(frozen_address != CoreAddresses::LIBRA_ROOT_ADDRESS(), 14);
        borrow_global_mut<LibraAccount>(frozen_address).is_frozen = true;
        Event::emit_event<FreezeAccountEvent>(
            &mut borrow_global_mut<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()).freeze_event_handle,
            FreezeAccountEvent {
                initiator_address,
                frozen_address
            },
        );
    }

    // Unfreeze the account at `addr`.
    public fun unfreeze_account(
        account: &signer,
        _unfreezing_capability: &Capability<AccountUnfreezing>,
        unfrozen_address: address,
    )
    acquires LibraAccount, AccountOperationsCapability {
        let initiator_address = Signer::address_of(account);
        borrow_global_mut<LibraAccount>(unfrozen_address).is_frozen = false;
        Event::emit_event<UnfreezeAccountEvent>(
            &mut borrow_global_mut<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()).unfreeze_event_handle,
            UnfreezeAccountEvent {
                initiator_address,
                unfrozen_address
            },
        );
    }

    // Returns if the account at `addr` is frozen.
    public fun account_is_frozen(addr: address): bool
    acquires LibraAccount {
        borrow_global<LibraAccount>(addr).is_frozen
     }

    // The prologue is invoked at the beginning of every transaction
    // It verifies:
    // - The account's auth key matches the transaction's public key
    // - That the account has enough balance to pay for all of the gas
    // - That the sequence number matches the transaction's sequence key
    fun prologue<Token>(
        sender: &signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
    ) acquires LibraAccount, Balance {
        let transaction_sender = Signer::address_of(sender);

        // FUTURE: Make these error codes sequential
        // Verify that the transaction sender's account exists
        assert(exists_at(transaction_sender), 5);

        assert(!account_is_frozen(transaction_sender), 0);

        // Load the transaction sender's account
        let sender_account = borrow_global_mut<LibraAccount>(transaction_sender);

        // Check that the hash of the transaction's public key matches the account's auth key
        assert(
            Hash::sha3_256(txn_public_key) == *&sender_account.authentication_key,
            2
        );

        // Check that the account has enough balance for all of the gas
        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        // Don't grab the balance if the transaction fee is zero
        if (max_transaction_fee > 0) {
            let balance_amount = balance<Token>(transaction_sender);
            assert(balance_amount >= max_transaction_fee, 6);
        };

        // Check that the transaction sequence number matches the sequence number of the account
        assert(txn_sequence_number >= sender_account.sequence_number, 3);
        assert(txn_sequence_number == sender_account.sequence_number, 4);
        assert(LibraTransactionTimeout::is_valid_transaction_timestamp(txn_expiration_time), 7);
    }

    //  Collects gas and bumps the sequence number for executing a transaction
    fun epilogue<Token>(
        sender: address,
        transaction_fee_amount: u64,
        txn_sequence_number: u64,
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        // Load the transaction sender's account and balance resources
        let sender_account = borrow_global_mut<LibraAccount>(sender);

        // Bump the sequence number
        sender_account.sequence_number = txn_sequence_number + 1;

        if (transaction_fee_amount > 0) {
            let sender_balance = borrow_global_mut<Balance<Token>>(sender);
            TransactionFee::pay_fee(
                withdraw_from_balance(sender, sender_balance, transaction_fee_amount)
            )
        }
    }

    // The success_epilogue is invoked at the end of successfully executed transactions.
    fun success_epilogue<Token>(
        account: &signer,
        txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        let sender = Signer::address_of(account);

        // Charge for gas
        let transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);

        // Load the transaction sender's balance resource only if it exists. If it doesn't we default the value to 0
        let sender_balance = if (exists<Balance<Token>>(sender)) balance<Token>(sender) else 0;
        assert(sender_balance >= transaction_fee_amount, 6);
        epilogue<Token>(sender, transaction_fee_amount, txn_sequence_number);
    }

    // The failure_epilogue is invoked at the end of transactions when the transaction is aborted during execution or
    // during `success_epilogue`.
    fun failure_epilogue<Token>(
        account: &signer,
        txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        let sender = Signer::address_of(account);
        // Charge for gas
        let transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);

        epilogue<Token>(sender, transaction_fee_amount, txn_sequence_number);
    }

    // Bump the sequence number of an account. This function should be used only for bumping the sequence number when
    // a writeset transaction is committed.
    fun bump_sequence_number(signer: &signer) acquires LibraAccount {
        let sender_account = borrow_global_mut<LibraAccount>(Signer::address_of(signer));
        sender_account.sequence_number = sender_account.sequence_number + 1;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Proof of concept code used for Validator and ValidatorOperator roles management
    ///////////////////////////////////////////////////////////////////////////

    public fun create_validator_account(
        creator_account: &signer,
        assoc_root_capability: &Capability<LibraRootRole>,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        let new_account = create_signer(new_account_address);
        Event::publish_generator(&new_account);
        Roles::new_validator_role(creator_account, &new_account);
        ValidatorConfig::publish(&new_account, assoc_root_capability);
        make_account(new_account, auth_key_prefix)
    }

    public fun create_validator_operator_account(
        creator_account: &signer,
        _: &Capability<LibraRootRole>,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        let new_account = create_signer(new_account_address);
        Event::publish_generator(&new_account);
        Roles::new_validator_operator_role(creator_account, &new_account);
        make_account(new_account, auth_key_prefix)
    }

    ///////////////////////////////////////////////////////////////////////////
    // End of the proof of concept code
    ///////////////////////////////////////////////////////////////////////////

    // ****************** SPECIFICATIONS *******************

    spec module {
        /// Returns field `key_rotation_capability` of the
        /// LibraAccount under `addr`.
        define spec_get_key_rotation_cap(addr: address):
            Option<KeyRotationCapability> {
            global<LibraAccount>(addr).key_rotation_capability
        }

        /// Returns true if the LibraAccount at `addr` holds
        /// `KeyRotationCapability` for itself.
        define spec_holds_own_key_rotation_cap(addr: address): bool {
            Option::spec_is_some(spec_get_key_rotation_cap(addr))
            && addr == Option::spec_value_inside(
                spec_get_key_rotation_cap(addr)).account_address
        }
    }
}
}
