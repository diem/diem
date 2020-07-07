address 0x1 {

// The module for the account resource that governs every Libra account
module LibraAccount {
    use 0x1::CoreAddresses;
    use 0x1::AccountLimits;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::Event::{Self, EventHandle};
    use 0x1::Hash;
    use 0x1::LBR::{Self, LBR};
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
    use 0x1::Roles;

    resource struct PublishModule {}

    /// Every Libra account has a LibraAccount resource
    resource struct LibraAccount {
        /// The current authentication key.
        /// This can be different than the key used to create the account
        authentication_key: vector<u8>,
        /// A `withdrawal_capability` allows whoever holds this capability
        /// to withdraw from the account. At the time of account creation
        /// this capability is stored in this option. It can later be
        /// and can also be restored via `restore_withdraw_capability`.
        withdrawal_capability: Option<WithdrawCapability>,
        /// A `key_rotation_capability` allows whoever holds this capability
        /// the ability to rotate the authentication key for the account. At
        /// the time of account creation this capability is stored in this
        /// option. It can later be "extracted" from this field via
        /// `extract_key_rotation_capability`, and can also be restored via
        /// `restore_key_rotation_capability`.
        key_rotation_capability: Option<KeyRotationCapability>,
        /// Event handle for received event
        received_events: EventHandle<ReceivedPaymentEvent>,
        /// Event handle for sent event
        sent_events: EventHandle<SentPaymentEvent>,
        /// The current sequence number.
        /// Incremented by one each time a transaction is submitted
        sequence_number: u64,
        /// If true, the account cannot be used to send transactions or receive funds
        is_frozen: bool,
    }

    /// A resource that holds the coins stored in this account
    resource struct Balance<Token> {
        coin: Libra<Token>,
    }

    /// The holder of WithdrawCapability for account_address can withdraw Libra from
    /// account_address/LibraAccount/balance.
    /// There is at most one WithdrawCapability in existence for a given address.
    resource struct WithdrawCapability {
        account_address: address,
    }

    /// The holder of KeyRotationCapability for account_address can rotate the authentication key for
    /// account_address (i.e., write to account_address/LibraAccount/authentication_key).
    /// There is at most one KeyRotationCapability in existence for a given address.
    resource struct KeyRotationCapability {
        account_address: address,
    }

    /// A wrapper around an `AccountLimits::CallingCapability` which is used to check for account limits
    /// and to record freeze/unfreeze events.
    resource struct AccountOperationsCapability {
        limits_cap: AccountLimits::CallingCapability,
        freeze_event_handle: EventHandle<FreezeAccountEvent>,
        unfreeze_event_handle: EventHandle<UnfreezeAccountEvent>,
    }

    /// Message for sent events
    struct SentPaymentEvent {
        /// The amount of Libra<Token> sent
        amount: u64,
        /// The code symbol for the currency that was sent
        currency_code: vector<u8>,
        /// The address that was paid
        payee: address,
        /// Metadata associated with the payment
        metadata: vector<u8>,
    }

    /// Message for received events
    struct ReceivedPaymentEvent {
        /// The amount of Libra<Token> received
        amount: u64,
        /// The code symbol for the currency that was received
        currency_code: vector<u8>,
        /// The address that sent the coin
        payer: address,
        /// Metadata associated with the payment
        metadata: vector<u8>,
    }

    /// A privilege to allow the freezing of accounts.
    struct FreezingPrivilege { }

    /// Message for freeze account events
    struct FreezeAccountEvent {
        /// The address that initiated freeze txn
        initiator_address: address,
        /// The address that was frozen
        frozen_address: address,
    }

    /// Message for unfreeze account events
    struct UnfreezeAccountEvent {
        /// The address that initiated unfreeze txn
        initiator_address: address,
        /// The address that was unfrozen
        unfrozen_address: address,
    }


    const ENOT_GENESIS: u64 = 0;
    const EINVALID_SINGLETON_ADDRESS: u64 = 1;
    const ECOIN_DEPOSIT_IS_ZERO: u64 = 2;
    const EDEPOSIT_EXCEEDS_LIMITS: u64 = 3;
    const EMALFORMED_METADATA_SIGNATURE: u64 = 4;
    const EINVALID_METADATA_SIGNATURE: u64 = 5;
    const EWITHDRAWAL_EXCEEDS_LIMITS: u64 = 6;
    const EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED: u64 = 7;
    const EMALFORMED_AUTHENTICATION_KEY: u64 = 8;
    const EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED: u64 = 9;
    const ECANNOT_CREATE_AT_VM_RESERVED: u64 = 10;
    const ENOT_LIBRA_ROOT: u64 = 11;
    const EACCOUNT_ALREADY_EXISTS: u64 = 12;
    const EPARENT_VASP_CURRENCY_LIMITS_DNE: u64 = 13;
    const ENOT_TREASURY_COMPLIANCE: u64 = 14;
    const EACCOUNT_CANNOT_BE_FROZEN: u64 = 15;
    const ENOT_A_CURRENCY: u64 = 16;
    const EADD_EXISTING_CURRENCY: u64 = 17;

    /// Prologue errors. These are separated out from the other errors in this
    /// module since they are mapped separately to major VM statuses, and are
    /// important to the semantics of the system.
    const EPROLOGUE_ACCOUNT_FROZEN: u64 = 0;
    const EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY: u64 = 1;
    const EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD: u64 = 2;
    const EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW: u64 = 3;
    const EPROLOGUE_ACCOUNT_DNE: u64 = 4;
    const EPROLOGUE_CANT_PAY_GAS_DEPOSIT: u64 = 5;
    const EPROLOGUE_TRANSACTION_EXPIRED: u64 = 6;

    /// Grants the ability to publish modules to the libra root account. The VM
    /// will look for this resource to determine whether the sending account can publish modules.
    /// Aborts if the `account` does not have the correct role (libra root).
    public fun grant_module_publishing_privilege(account: &signer) {
        // This is also an operational constraint since the VM will look for
        // this specific address and publish these modules under the core code
        // address.
        assert(Roles::has_libra_root_role(account), ENOT_LIBRA_ROOT);
        assert(Signer::address_of(account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);
        move_to(account, PublishModule{});
    }

    /// Initialize this module. This is only callable from genesis.
    public fun initialize(
        lr_account: &signer,
    ) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        // Operational constraint, not a privilege constraint.
        assert(Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);
        let limits_cap = AccountLimits::grant_calling_capability(lr_account);
        AccountLimits::initialize(lr_account, &limits_cap);
        move_to(
            lr_account,
            AccountOperationsCapability {
                limits_cap,
                freeze_event_handle: Event::new_event_handle(lr_account),
                unfreeze_event_handle: Event::new_event_handle(lr_account),
            }
        );
    }

    /// Returns whether we should track and record limits for the `payer` or `payee` account.
    /// Depending on the `is_withdrawal` flag passed in we determine whether the
    /// `payer` or `payee` account is being queried. `VASP->any` and
    /// `any->VASP` transfers are tracked in the VASP.
    fun should_track_limits_for_account(payer: address, payee: address, is_withdrawal: bool): bool {
        if (is_withdrawal) {
            VASP::is_vasp(payer) && (!VASP::is_vasp(payee) || !VASP::is_same_vasp(payer, payee))
        } else {
            VASP::is_vasp(payee) && (!VASP::is_vasp(payee) || !VASP::is_same_vasp(payer, payee))
        }
    }
    spec fun should_track_limits_for_account {
        pragma opaque = true, verify = true;
        ensures result == spec_should_track_limits_for_account(payer, payee, is_withdrawal);
    }
    spec module {
        define spec_should_track_limits_for_account(payer: address, payee: address, is_withdrawal: bool): bool {
            if (is_withdrawal) {
                VASP::spec_is_vasp(payer) && (!VASP::spec_is_vasp(payee) || !VASP::spec_is_same_vasp(payer, payee))
            } else {
                VASP::spec_is_vasp(payee) && (!VASP::spec_is_vasp(payee) || !VASP::spec_is_same_vasp(payer, payee))
            }
        }
    }

    /// Use `cap` to mint `amount_lbr` LBR by withdrawing the appropriate quantity of reserve assets
    /// from `cap.address`, giving them to the LBR reserve, and depositing the LBR into
    /// `cap.address`.
    /// The `payee` address in the `SentPaymentEvent`s emitted by this function is the LBR reserve
    /// address to signify that this was a special payment that debits the `cap.addr`'s balance and
    /// credits the LBR reserve.
    public fun staple_lbr(cap: &WithdrawCapability, amount_lbr: u64)
    acquires LibraAccount, Balance, AccountOperationsCapability {
        let cap_address = cap.account_address;
        // withdraw all Coin1 and Coin2
        let coin1_balance = balance<Coin1>(cap_address);
        let coin2_balance = balance<Coin2>(cap_address);
        // use the LBR reserve address as `payee_address`
        let payee_address = LBR::reserve_address();
        let coin1 = withdraw_from<Coin1>(cap, payee_address, coin1_balance, x"");
        let coin2 = withdraw_from<Coin2>(cap, payee_address, coin2_balance, x"");
        // Create `amount_lbr` LBR
        let (lbr, coin1, coin2) = LBR::create(amount_lbr, coin1, coin2);
        // use the reserved address as the payer for the LBR payment because the funds did not come
        // from an existing balance
        deposit(CoreAddresses::VM_RESERVED_ADDRESS(), cap_address, lbr, x"", x"");
        // TODO: eliminate these self-deposits by withdrawing appropriate amounts up-front
        // Deposit the Coin1/Coin2 remainders
        deposit(cap_address, cap_address, coin1, x"", x"");
        deposit(cap_address, cap_address, coin2, x"", x"")
    }
    spec fun staple_lbr {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
    }

    /// Use `cap` to withdraw `amount_lbr`, burn the LBR, withdraw the corresponding assets from the
    /// LBR reserve, and deposit them to `cap.address`.
    /// The `payer` address in the` RecievedPaymentEvent`s emitted by this function will be the LBR
    /// reserve address to signify that this was a special payment that credits
    /// `cap.address`'s balance and credits the LBR reserve.
    public fun unstaple_lbr(cap: &WithdrawCapability, amount_lbr: u64)
    acquires LibraAccount, Balance, AccountOperationsCapability {
        // use the reserved address as the payee because the funds will be burned
        let lbr = withdraw_from<LBR>(cap, CoreAddresses::VM_RESERVED_ADDRESS(), amount_lbr, x"");
        let (coin1, coin2) = LBR::unpack(lbr);
        // These funds come from the LBR reserve, so use the LBR reserve address as the payer
        let payer_address = LBR::reserve_address();
        let payee_address = cap.account_address;
        deposit(payer_address, payee_address, coin1, x"", x"");
        deposit(payer_address, payee_address, coin2, x"", x"")
    }
    spec fun unstaple_lbr {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
    }

    /// Record a payment of `to_deposit` from `payer` to `payee` with the attached `metadata`
    fun deposit<Token>(
        payer: address,
        payee: address,
        to_deposit: Libra<Token>,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        // Check that the `to_deposit` coin is non-zero
        let deposit_value = Libra::value(&to_deposit);
        assert(deposit_value > 0, ECOIN_DEPOSIT_IS_ZERO);
        if (dual_attestation_required<Token>(payer, payee, deposit_value)) {
            assert_signature_is_valid(payer, payee, &metadata_signature, &metadata, deposit_value);
        };

        // Ensure that this deposit is compliant with the account limits on
        // this account.
        if (should_track_limits_for_account(payer, payee, false)) {
            assert(
                AccountLimits::update_deposit_limits<Token>(
                    deposit_value,
                    VASP::parent_address(payee),
                    &borrow_global<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()).limits_cap
                ),
                EDEPOSIT_EXCEEDS_LIMITS
            )
        };

        // Deposit the `to_deposit` coin
        Libra::deposit(&mut borrow_global_mut<Balance<Token>>(payee).coin, to_deposit);

        // Log a received event
        Event::emit_event<ReceivedPaymentEvent>(
            &mut borrow_global_mut<LibraAccount>(payee).received_events,
            ReceivedPaymentEvent {
                amount: deposit_value,
                currency_code: Libra::currency_code<Token>(),
                payer,
                metadata: metadata
            }
        );
    }
    spec fun deposit {
        pragma verify = true;
        include DepositAbortsIf<Token>;
        include DepositEnsures<Token>;
    }
    spec schema DepositAbortsIf<Token> {
        payer: address;
        payee: address;
        to_deposit: Libra<Token>;
        metadata_signature: vector<u8>;
        metadata: vector<u8>;
        aborts_if spec_should_track_limits_for_account(payer, payee, false)
                    && (!spec_has_account_operations_cap()
                       || !AccountLimits::spec_update_deposit_limits<Token>(
                              to_deposit.value,
                              VASP::spec_parent_address(payee)
                           )
                       );
        aborts_if to_deposit.value == 0;
        aborts_if !exists<LibraAccount>(payee);
        aborts_if !exists<Balance<Token>>(payee);
        aborts_if global<Balance<Token>>(payee).coin.value + to_deposit.value > max_u64();
        include TravelRuleAppliesAbortsIf<Token>;
        aborts_if spec_dual_attestation_required<Token>(payer, payee, to_deposit.value)
                        && !signature_is_valid(payer, payee, metadata_signature, metadata, to_deposit.value);
    }
    spec schema DepositEnsures<Token> {
        payer: address;
        payee: address;
        to_deposit: Libra<Token>;
        ensures global<Balance<Token>>(payee).coin.value
            == old(global<Balance<Token>>(payee).coin.value) + to_deposit.value;
    }

    /// Helper function to check validity of a signature when dual attestion is required.
    fun assert_signature_is_valid(
        payer: address,
        payee: address,
        metadata_signature: &vector<u8>,
        metadata: &vector<u8>,
        deposit_value: u64
    ) {
        // sanity check of signature validity
        assert(Vector::length(metadata_signature) == 64, EMALFORMED_METADATA_SIGNATURE);
        // cryptographic check of signature validity
        let message = dual_attestation_message(payer, metadata, deposit_value);
        assert(
            Signature::ed25519_verify(
                *metadata_signature,
                VASP::compliance_public_key(payee),
                message
            ),
            EINVALID_METADATA_SIGNATURE
        );
    }
    spec fun assert_signature_is_valid {
        pragma verify = true, opaque = true;
        aborts_if !signature_is_valid(payer, payee, metadata_signature, metadata, deposit_value);
    }
    spec module {
        /// Returns true if signature is valid.
        define signature_is_valid(
            payer: address,
            payee: address,
            metadata_signature: vector<u8>,
            metadata: vector<u8>,
            deposit_value: u64
        ): bool {
            len(metadata_signature) == 64
                && Signature::spec_ed25519_verify(
                        metadata_signature,
                        VASP::spec_compliance_public_key(payee),
                        spec_dual_attestation_message(payer, metadata, deposit_value)
                   )
        }
    }

    /// Helper which returns true if dual attestion is required for a deposit.
    fun dual_attestation_required<Token>(payer: address, payee: address, deposit_value: u64): bool {
        // travel rule applies for payments over a threshold
        let travel_rule_limit_microlibra = DualAttestationLimit::get_cur_microlibra_limit();
        let approx_lbr_microlibra_value = Libra::approx_lbr_for_value<Token>(deposit_value);
        let above_threshold = approx_lbr_microlibra_value >= travel_rule_limit_microlibra;
        // travel rule applies if the payer and payee are both VASP, but not for
        // intra-VASP transactions
        above_threshold
            && VASP::is_vasp(payer) && VASP::is_vasp(payee)
            && VASP::parent_address(payer) != VASP::parent_address(payee)
    }
    spec fun dual_attestation_required {
        pragma verify = true, opaque = true;
        include TravelRuleAppliesAbortsIf<Token>;
        ensures result == spec_dual_attestation_required<Token>(payer, payee, deposit_value);
    }
    spec schema TravelRuleAppliesAbortsIf<Token> {
        aborts_if !Libra::spec_is_currency<Token>();
        aborts_if !DualAttestationLimit::spec_is_published();
    }
    spec module {
        /// Helper functions which simulates `Self::dual_attestation_required`.
        define spec_dual_attestation_required<Token>(payer: address, payee: address, deposit_value: u64): bool {
            Libra::spec_approx_lbr_for_value<Token>(deposit_value)
                    >= DualAttestationLimit::spec_get_cur_microlibra_limit()
            && VASP::spec_is_vasp(payer) && VASP::spec_is_vasp(payee)
            && VASP::spec_parent_address(payer) != VASP::spec_parent_address(payee)
        }
    }

    /// Helper to construct a message for dual attestation.
    /// Message is `metadata | payer | amount | domain_separator`.
    fun dual_attestation_message(payer: address, metadata: &vector<u8>, deposit_value: u64): vector<u8> {
        let domain_separator = b"@@$$LIBRA_ATTEST$$@@";
        let message = *metadata;
        Vector::append(&mut message, LCS::to_bytes(&payer));
        Vector::append(&mut message, LCS::to_bytes(&deposit_value));
        Vector::append(&mut message, domain_separator);
        message
    }
    spec fun dual_attestation_message {
        /// Abstract from construction of message for the prover. Concatenation of results from `LCS::to_bytes`
        /// are difficult to reason about, so we avoid doing it. This is possible because the actual value of this
        /// message is not important for the verification problem, as long as the prover considers both
        /// messages which fail verification and which do not.
        pragma opaque = true, verify = false;
        ensures result == spec_dual_attestation_message(payer, metadata, deposit_value);
    }
    spec module {
        /// Uninterpreted function for `Self::dual_attestation_message`.
        define spec_dual_attestation_message(payer: address, metadata: vector<u8>, deposit_value: u64): vector<u8>;
    }

    /// Mint 'mint_amount' to 'designated_dealer_address' for 'tier_index' tier.
    /// Max valid tier index is 3 since there are max 4 tiers per DD.
    /// Sender should be treasury compliance account and receiver authorized DD.
    public fun tiered_mint<Token>(
        tc_account: &signer,
        designated_dealer_address: address,
        mint_amount: u64,
        tier_index: u64,
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        let coin = DesignatedDealer::tiered_mint<Token>(
            tc_account, mint_amount, designated_dealer_address, tier_index
        );
        // Use the reserved address as the payer because the funds did not come from an existing
        // balance
        deposit(CoreAddresses::VM_RESERVED_ADDRESS(), designated_dealer_address, coin, x"", x"")
    }
    spec fun tiered_mint {
        pragma verify = true;
    }

    /// Cancel the oldest burn request from `preburn_address` and return the funds.
    /// Fails if the sender does not have a published MintCapability.
    public fun cancel_burn<Token>(
        account: &signer,
        preburn_address: address,
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        let coin = Libra::cancel_burn<Token>(account, preburn_address);
        // record both sender and recipient as `preburn_address`: the coins are moving from
        // `preburn_address`'s `Preburn` resource to its balance
        deposit(preburn_address, preburn_address, coin, x"", x"")
    }
    spec fun tiered_mint {
        pragma verify = true;
    }

    /// Helper to withdraw `amount` from the given account balance and return the withdrawn Libra<Token>
    fun withdraw_from_balance<Token>(
        payer: address,
        payee: address,
        balance: &mut Balance<Token>,
        amount: u64
    ): Libra<Token> acquires AccountOperationsCapability {
        // Make sure that this withdrawal is compliant with the limits on
        // the account if it's a inter-VASP transfer,
        if (should_track_limits_for_account(payer, payee, true)) {
            let can_withdraw = AccountLimits::update_withdrawal_limits<Token>(
                    amount,
                    VASP::parent_address(payer),
                    &borrow_global<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()).limits_cap
            );
            assert(can_withdraw, EWITHDRAWAL_EXCEEDS_LIMITS);
        };
        Libra::withdraw(&mut balance.coin, amount)
    }
    spec fun withdraw_from_balance {
        pragma verify = true;
    }

    /// Withdraw `amount` `Libra<Token>`'s from the account balance under
    /// `cap.account_address`
    fun withdraw_from<Token>(
        cap: &WithdrawCapability,
        payee: address,
        amount: u64,
        metadata: vector<u8>,
    ): Libra<Token> acquires Balance, AccountOperationsCapability, LibraAccount {
        let payer = cap.account_address;
        let account_balance = borrow_global_mut<Balance<Token>>(payer);
        // Load the payer's account and emit an event to record the withdrawal
        Event::emit_event<SentPaymentEvent>(
            &mut borrow_global_mut<LibraAccount>(payer).sent_events,
            SentPaymentEvent {
                amount,
                currency_code: Libra::currency_code<Token>(),
                payee,
                metadata
            },
        );
        withdraw_from_balance<Token>(payer, payee, account_balance, amount)
    }
    spec fun withdraw_from {
        pragma verify = true;
    }

    /// Withdraw `amount` `Libra<Token>`'s from `cap.address` and send them to the `Preburn`
    /// resource under `dd`.
    public fun preburn<Token>(
        dd: &signer, cap: &WithdrawCapability, amount: u64
    ) acquires Balance, AccountOperationsCapability, LibraAccount {
        Libra::preburn_to<Token>(dd, withdraw_from(cap, Signer::address_of(dd), amount, x""))
    }

    /// Return a unique capability granting permission to withdraw from the sender's account balance.
    public fun extract_withdraw_capability(
        sender: &signer
    ): WithdrawCapability acquires LibraAccount {
        let sender_addr = Signer::address_of(sender);
        // Abort if we already extracted the unique withdraw capability for this account.
        assert(!delegated_withdraw_capability(sender_addr), EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED);
        let account = borrow_global_mut<LibraAccount>(sender_addr);
        Option::extract(&mut account.withdrawal_capability)
    }
    spec fun extract_withdraw_capability {
        pragma verify = true;
    }

    /// Return the withdraw capability to the account it originally came from
    public fun restore_withdraw_capability(cap: WithdrawCapability)
    acquires LibraAccount {
        let account = borrow_global_mut<LibraAccount>(cap.account_address);
        Option::fill(&mut account.withdrawal_capability, cap)
    }

    /// Withdraw `amount` Libra<Token> from the address embedded in `WithdrawCapability` and
    /// deposits it into the `payee`'s account balance.
    /// The included `metadata` will appear in the `SentPaymentEvent` and `ReceivedPaymentEvent`.
    /// The `metadata_signature` will only be checked if this payment is subject to the dual
    /// attestation protocol
    public fun pay_from<Token>(
        cap: &WithdrawCapability,
        payee: address,
        amount: u64,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        deposit<Token>(
            *&cap.account_address,
            payee,
            withdraw_from(cap, payee, amount, copy metadata),
            metadata,
            metadata_signature
        );
    }
    spec fun pay_from {
        pragma verify = true;
    }

    /// Rotate the authentication key for the account under cap.account_address
    public fun rotate_authentication_key(
        cap: &KeyRotationCapability,
        new_authentication_key: vector<u8>,
    ) acquires LibraAccount  {
        let sender_account_resource = borrow_global_mut<LibraAccount>(cap.account_address);
        // Don't allow rotating to clearly invalid key
        assert(Vector::length(&new_authentication_key) == 32, EMALFORMED_AUTHENTICATION_KEY);
        sender_account_resource.authentication_key = new_authentication_key;
    }
    spec fun rotate_authentication_key {
        pragma verify = true;
    }

    /// Return a unique capability granting permission to rotate the sender's authentication key
    public fun extract_key_rotation_capability(account: &signer): KeyRotationCapability
    acquires LibraAccount {
        let account_address = Signer::address_of(account);
        // Abort if we already extracted the unique key rotation capability for this account.
        assert(!delegated_key_rotation_capability(account_address), EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED);
        let account = borrow_global_mut<LibraAccount>(account_address);
        Option::extract(&mut account.key_rotation_capability)
    }
    spec fun extract_key_rotation_capability {
        pragma verify = true;
    }

    /// Return the key rotation capability to the account it originally came from
    public fun restore_key_rotation_capability(cap: KeyRotationCapability)
    acquires LibraAccount {
        let account = borrow_global_mut<LibraAccount>(cap.account_address);
        Option::fill(&mut account.key_rotation_capability, cap)
    }
    spec fun restore_key_rotation_capability {
        pragma verify = true;
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
    spec fun add_currencies_for_account {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
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
        assert(new_account_addr != CoreAddresses::VM_RESERVED_ADDRESS(), ECANNOT_CREATE_AT_VM_RESERVED);

        // (1) publish LibraAccount
        let authentication_key = auth_key_prefix;
        Vector::append(
            &mut authentication_key, LCS::to_bytes(Signer::borrow_address(&new_account))
        );
        assert(Vector::length(&authentication_key) == 32, EMALFORMED_AUTHENTICATION_KEY);
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
    spec fun make_account {
        pragma verify = true;
    }

    /// Creates the root association account in genesis.
    public fun create_root_association_account(
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(new_account_address == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);
        let new_account = create_signer(new_account_address);
        make_account(new_account, auth_key_prefix)
    }
    spec fun create_root_association_account {
        pragma verify = true;
    }

    /// Create a treasury/compliance account at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`
    public fun create_treasury_compliance_account(
        lr_account: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(Roles::has_libra_root_role(lr_account), ENOT_LIBRA_ROOT);
        let new_account = create_signer(new_account_address);
        SlidingNonce::publish_nonce_resource(lr_account, &new_account);
        Event::publish_generator(&new_account);
        make_account(new_account, auth_key_prefix)
    }
    spec fun create_treasury_compliance_account {
        pragma verify = true;
    }


    ///////////////////////////////////////////////////////////////////////////
    // Designated Dealer API
    ///////////////////////////////////////////////////////////////////////////

    /// Create a designated dealer account at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`, for non synthetic CoinType.
    /// Creates Preburn resource under account 'new_account_address'
    public fun create_designated_dealer<CoinType>(
        creator_account: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        add_all_currencies: bool,
    ) {
        let new_dd_account = create_signer(new_account_address);
        Event::publish_generator(&new_dd_account);
        Libra::publish_preburn_to_account<CoinType>(&new_dd_account, creator_account);
        DesignatedDealer::publish_designated_dealer_credential(&new_dd_account, creator_account);
        Roles::new_designated_dealer_role(creator_account, &new_dd_account);
        add_currencies_for_account<CoinType>(&new_dd_account, add_all_currencies);
        make_account(new_dd_account, auth_key_prefix)
    }
    spec fun create_designated_dealer {
        pragma verify = true;
    }

    ///////////////////////////////////////////////////////////////////////////
    // VASP methods
    ///////////////////////////////////////////////////////////////////////////

    /// Create an account with the ParentVASP role at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`.  If `add_all_currencies` is true, 0 balances for
    /// all available currencies in the system will also be added.
    public fun create_parent_vasp_account<Token>(
        creator_account: &signer,  // libra root
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
            creator_account,
            human_name,
            base_url,
            compliance_public_key
        );
        Event::publish_generator(&new_account);
        add_currencies_for_account<Token>(&new_account, add_all_currencies);
        make_account(new_account, auth_key_prefix)
    }
    spec fun create_parent_vasp_account {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
    }

    /// Create an account with the ChildVASP role at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address` and a 0 balance of type `Token`. If
    /// `add_all_currencies` is true, 0 balances for all avaialable currencies in the system will
    /// also be added. This account will be a child of `creator`, which must be a ParentVASP.
    public fun create_child_vasp_account<Token>(
        parent: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        add_all_currencies: bool,
    ) {
        let new_account = create_signer(new_account_address);
        Roles::new_child_vasp_role(parent, &new_account);
        VASP::publish_child_vasp_credential(
            parent,
            &new_account,
        );
        Event::publish_generator(&new_account);
        add_currencies_for_account<Token>(&new_account, add_all_currencies);
        make_account(new_account, auth_key_prefix)
    }
    spec fun create_child_vasp_account {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Unhosted methods
    ///////////////////////////////////////////////////////////////////////////

    /// For now, only the association root can create an unhosted account, and it will choose not to
    /// on mainnet
    /// > TODO(tzakian): eventually, anyone will be able to create an unhosted wallet accunt
    public fun create_unhosted_account<Token>(
        creator_account: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        add_all_currencies: bool
    ) {
        assert(Roles::has_libra_root_role(creator_account), ENOT_LIBRA_ROOT);
        assert(!exists_at(new_account_address), EACCOUNT_ALREADY_EXISTS);
        let new_account = create_signer(new_account_address);
        Roles::new_unhosted_role(creator_account, &new_account);
        Event::publish_generator(&new_account);
        add_currencies_for_account<Token>(&new_account, add_all_currencies);
        make_account(new_account, auth_key_prefix)
    }
    spec fun create_unhosted_account {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
    }

    ///////////////////////////////////////////////////////////////////////////
    // General purpose methods
    ///////////////////////////////////////////////////////////////////////////

    native fun create_signer(addr: address): signer;
    native fun destroy_signer(sig: signer);

    /// Helper to return the u64 value of the `balance` for `account`
    fun balance_for<Token>(balance: &Balance<Token>): u64 {
        Libra::value<Token>(&balance.coin)
    }

    /// Return the current balance of the account at `addr`.
    public fun balance<Token>(addr: address): u64 acquires Balance {
        balance_for(borrow_global<Balance<Token>>(addr))
    }

    /// Add a balance of `Token` type to the sending account.
    /// If the account is a VASP account, it must have (or be able to publish)
    /// a limits definition and window.
    public fun add_currency<Token>(account: &signer) {
        // aborts if `Token` is not a currency type in the system
        assert(Libra::is_currency<Token>(), ENOT_A_CURRENCY);
        // aborts if this account already has a balance in `Token`
        assert(!exists<Balance<Token>>(Signer::address_of(account)), EADD_EXISTING_CURRENCY);
        // aborts if this is a child VASP whose parent does not have an AccountLimits resource for
        // `Token`
        assert(VASP::try_allow_currency<Token>(account), EPARENT_VASP_CURRENCY_LIMITS_DNE);
        move_to(account, Balance<Token>{ coin: Libra::zero<Token>() })
    }
    spec fun add_currency {
        pragma verify = true;
    }

    /// Return whether the account at `addr` accepts `Token` type coins
    public fun accepts_currency<Token>(addr: address): bool {
        exists<Balance<Token>>(addr)
    }

    /// Helper to return the sequence number field for given `account`
    fun sequence_number_for_account(account: &LibraAccount): u64 {
        account.sequence_number
    }

    /// Return the current sequence number at `addr`
    public fun sequence_number(addr: address): u64 acquires LibraAccount {
        sequence_number_for_account(borrow_global<LibraAccount>(addr))
    }

    /// Return the authentication key for this account
    public fun authentication_key(addr: address): vector<u8> acquires LibraAccount {
        *&borrow_global<LibraAccount>(addr).authentication_key
    }

    /// Return true if the account at `addr` has delegated its key rotation capability
    public fun delegated_key_rotation_capability(addr: address): bool
    acquires LibraAccount {
        Option::is_none(&borrow_global<LibraAccount>(addr).key_rotation_capability)
    }

    /// Return true if the account at `addr` has delegated its withdraw capability
    public fun delegated_withdraw_capability(addr: address): bool
    acquires LibraAccount {
        Option::is_none(&borrow_global<LibraAccount>(addr).withdrawal_capability)
    }

    /// Return a reference to the address associated with the given withdraw capability
    public fun withdraw_capability_address(cap: &WithdrawCapability): &address {
        &cap.account_address
    }

    /// Return a reference to the address associated with the given key rotation capability
    public fun key_rotation_capability_address(cap: &KeyRotationCapability): &address {
        &cap.account_address
    }

    /// Checks if an account exists at `check_addr`
    public fun exists_at(check_addr: address): bool {
        exists<LibraAccount>(check_addr)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Freezing
    ///////////////////////////////////////////////////////////////////////////

    /// Freeze the account at `addr`.
    public fun freeze_account(
        account: &signer,
        frozen_address: address,
    )
    acquires LibraAccount, AccountOperationsCapability {
        assert(Roles::has_treasury_compliance_role(account), ENOT_TREASURY_COMPLIANCE);
        let initiator_address = Signer::address_of(account);
        // The libra root and TC accounts cannot be frozen
        assert(frozen_address != CoreAddresses::LIBRA_ROOT_ADDRESS(), EACCOUNT_CANNOT_BE_FROZEN);
        assert(frozen_address != CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(), EACCOUNT_CANNOT_BE_FROZEN);
        borrow_global_mut<LibraAccount>(frozen_address).is_frozen = true;
        Event::emit_event<FreezeAccountEvent>(
            &mut borrow_global_mut<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()).freeze_event_handle,
            FreezeAccountEvent {
                initiator_address,
                frozen_address
            },
        );
    }
    spec fun freeze_account {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
    }

    /// Unfreeze the account at `addr`.
    public fun unfreeze_account(
        account: &signer,
        unfrozen_address: address,
    )
    acquires LibraAccount, AccountOperationsCapability {
        assert(Roles::has_treasury_compliance_role(account), ENOT_TREASURY_COMPLIANCE);
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
    spec fun unfreeze_account {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
    }

    /// Returns if the account at `addr` is frozen.
    public fun account_is_frozen(addr: address): bool
    acquires LibraAccount {
        borrow_global<LibraAccount>(addr).is_frozen
     }

    /// The prologue is invoked at the beginning of every transaction
    /// It verifies:
    /// - The account's auth key matches the transaction's public key
    /// - That the account has enough balance to pay for all of the gas
    /// - That the sequence number matches the transaction's sequence key
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
        assert(exists_at(transaction_sender), EPROLOGUE_ACCOUNT_DNE);

        assert(!account_is_frozen(transaction_sender), EPROLOGUE_ACCOUNT_FROZEN);

        // Load the transaction sender's account
        let sender_account = borrow_global_mut<LibraAccount>(transaction_sender);

        // Check that the hash of the transaction's public key matches the account's auth key
        assert(
            Hash::sha3_256(txn_public_key) == *&sender_account.authentication_key,
            EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY
        );

        // Check that the account has enough balance for all of the gas
        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        // Don't grab the balance if the transaction fee is zero
        if (max_transaction_fee > 0) {
            let balance_amount = balance<Token>(transaction_sender);
            assert(balance_amount >= max_transaction_fee, EPROLOGUE_CANT_PAY_GAS_DEPOSIT);
        };

        // Check that the transaction sequence number matches the sequence number of the account
        assert(txn_sequence_number >= sender_account.sequence_number, EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD);
        assert(txn_sequence_number == sender_account.sequence_number, EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW);
        assert(LibraTransactionTimeout::is_valid_transaction_timestamp(txn_expiration_time), EPROLOGUE_TRANSACTION_EXPIRED);
    }
    spec fun prologue {
        /// TODO(wrwg): function takes very long to verify; investigate why
        pragma verify = false;
    }

    /// Collects gas and bumps the sequence number for executing a transaction
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
                withdraw_from_balance(
                    sender,
                    CoreAddresses::LIBRA_ROOT_ADDRESS(),
                    sender_balance,
                    transaction_fee_amount
                )
            )
        }
    }
    spec fun epilogue {
        pragma verify = true;
    }

    /// The success_epilogue is invoked at the end of successfully executed transactions.
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
        assert(sender_balance >= transaction_fee_amount, EPROLOGUE_CANT_PAY_GAS_DEPOSIT);
        epilogue<Token>(sender, transaction_fee_amount, txn_sequence_number);
    }
    spec fun success_epilogue {
        pragma verify = true;
    }


    /// The failure_epilogue is invoked at the end of transactions when the transaction is aborted during execution or
    /// during `success_epilogue`.
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
    spec fun failure_epilogue {
        pragma verify = true;
    }

    /// Bump the sequence number of an account. This function should be used only for bumping the sequence number when
    /// a writeset transaction is committed.
    fun bump_sequence_number(signer: &signer) acquires LibraAccount {
        let sender_account = borrow_global_mut<LibraAccount>(Signer::address_of(signer));
        sender_account.sequence_number = sender_account.sequence_number + 1;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Proof of concept code used for Validator and ValidatorOperator roles management
    ///////////////////////////////////////////////////////////////////////////

    public fun create_validator_account(
        creator_account: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        assert(Roles::has_libra_root_role(creator_account), ENOT_LIBRA_ROOT);
        let new_account = create_signer(new_account_address);
        Event::publish_generator(&new_account);
        ValidatorConfig::publish(&new_account, creator_account);
        make_account(new_account, auth_key_prefix)
    }
    spec fun create_validator_account {
        pragma verify = true;
    }

    public fun create_validator_operator_account(
        creator_account: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        assert(Roles::has_libra_root_role(creator_account), ENOT_LIBRA_ROOT);
        let new_account = create_signer(new_account_address);
        Event::publish_generator(&new_account);
        make_account(new_account, auth_key_prefix)
    }
    spec fun create_validator_operator_account {
        pragma verify = true;
    }

    ///////////////////////////////////////////////////////////////////////////
    // End of the proof of concept code
    ///////////////////////////////////////////////////////////////////////////

    // ****************** SPECIFICATIONS *******************

    spec module {
        pragma verify = true;

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

        /// Returns true if `AccountOperationsCapability` is published.
        define spec_has_account_operations_cap(): bool {
            exists<AccountOperationsCapability>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS())
        }
    }
}
}
