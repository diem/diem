address 0x1 {

// The module for the account resource that governs every Libra account
module LibraAccount {
    use 0x1::AccountFreezing;
    use 0x1::CoreAddresses;
    use 0x1::ChainId;
    use 0x1::AccountLimits::{Self, AccountLimitMutationCapability};
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::DualAttestation;
    use 0x1::Errors;
    use 0x1::Event::{Self, EventHandle};
    use 0x1::Hash;
    use 0x1::LBR::{Self, LBR};
    use 0x1::LCS;
    use 0x1::LibraTimestamp;
    use 0x1::LibraTransactionPublishingOption;
    use 0x1::LibraTransactionTimeout;
    use 0x1::Signer;
    use 0x1::SlidingNonce;
    use 0x1::TransactionFee;
    use 0x1::ValidatorConfig;
    use 0x1::ValidatorOperatorConfig;
    use 0x1::VASP;
    use 0x1::Vector;
    use 0x1::DesignatedDealer;
    use 0x1::Libra::{Self, Libra};
    use 0x1::Option::{Self, Option};
    use 0x1::Roles;

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

    /// A wrapper around an `AccountLimitMutationCapability` which is used to check for account limits
    /// and to record freeze/unfreeze events.
    resource struct AccountOperationsCapability {
        limits_cap: AccountLimitMutationCapability,
    }

    spec module {
        /// After genesis, the `AccountOperationsCapability` exists.
        invariant [global]
            LibraTimestamp::is_operating() ==> exists<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS());
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

    const MAX_U64: u128 = 18446744073709551615;

    const EACCOUNT: u64 = 0;
    const ESEQUENCE_NUMBER: u64 = 1;
    const ECOIN_DEPOSIT_IS_ZERO: u64 = 2;
    const EDEPOSIT_EXCEEDS_LIMITS: u64 = 3;
    const EROLE_CANT_STORE_BALANCE: u64 = 4;
    const EINSUFFICIENT_BALANCE: u64 = 5;
    const EWITHDRAWAL_EXCEEDS_LIMITS: u64 = 6;
    const EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED: u64 = 7;
    const EMALFORMED_AUTHENTICATION_KEY: u64 = 8;
    const EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED: u64 = 9;
    const ECANNOT_CREATE_AT_VM_RESERVED: u64 = 10;
    const EADD_EXISTING_CURRENCY: u64 = 15;
    /// Attempting to send funds to an account that does not exist
    const EPAYEE_DOES_NOT_EXIST: u64 = 17;
    /// Attempting to send funds in (e.g.) LBR to an account that exists, but does not have a
    /// Balance<LBR> resource
    const EPAYEE_CANT_ACCEPT_CURRENCY_TYPE: u64 = 18;
    const EPAYER_DOESNT_HOLD_CURRENCY: u64 = 19;
    const EGAS: u64 = 20;
    const EACCOUNT_OPERATIONS_CAPABILITY: u64 = 22;

    /// Prologue errors. These are separated out from the other errors in this
    /// module since they are mapped separately to major VM statuses, and are
    /// important to the semantics of the system. Those codes also need to be
    /// directly used in aborts instead of augmenting them with a category
    /// via the `Errors` module.
    const EPROLOGUE_ACCOUNT_FROZEN: u64 = 0;
    const EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY: u64 = 1;
    const EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD: u64 = 2;
    const EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW: u64 = 3;
    const EPROLOGUE_ACCOUNT_DNE: u64 = 4;
    const EPROLOGUE_CANT_PAY_GAS_DEPOSIT: u64 = 5;
    const EPROLOGUE_TRANSACTION_EXPIRED: u64 = 6;
    const EPROLOGUE_BAD_CHAIN_ID: u64 = 7;
    const EPROLOGUE_SCRIPT_NOT_ALLOWED: u64 = 8;
    const EPROLOGUE_MODULE_NOT_ALLOWED: u64 = 9;

    /// This error will not be translated it should be an invariant violation.
    const EPROLOGUE_UNEXPECTED_WRITESET: u64 = 10;

    const WRITESET_TRANSACTION_TAG: u8 = 0;
    const SCRIPT_TRANSACTION_TAG: u8 = 1;
    const MODULE_TRANSACTION_TAG: u8 = 2;


    /// Initialize this module. This is only callable from genesis.
    public fun initialize(
        lr_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        // Operational constraint, not a privilege constraint.
        CoreAddresses::assert_libra_root(lr_account);
        assert(
            !exists<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()),
            Errors::already_published(EACCOUNT_OPERATIONS_CAPABILITY)
        );
        move_to(
            lr_account,
            AccountOperationsCapability {
                limits_cap: AccountLimits::grant_mutation_capability(lr_account),
            }
        );
    }

    /// Return `true` if `addr` has already published account limits for `Token`
    fun has_published_account_limits<Token>(addr: address): bool {
        if (VASP::is_vasp(addr)) VASP::has_account_limits<Token>(addr)
        else AccountLimits::has_window_published<Token>(addr)
    }

    /// Returns whether we should track and record limits for the `payer` or `payee` account.
    /// Depending on the `is_withdrawal` flag passed in we determine whether the
    /// `payer` or `payee` account is being queried. `VASP->any` and
    /// `any->VASP` transfers are tracked in the VASP.
    fun should_track_limits_for_account<Token>(
        payer: address, payee: address, is_withdrawal: bool
    ): bool {
        if (is_withdrawal) {
            has_published_account_limits<Token>(payer) &&
            VASP::is_vasp(payer) &&
            (!VASP::is_vasp(payee) || !VASP::is_same_vasp(payer, payee))
        } else {
            has_published_account_limits<Token>(payee) &&
            VASP::is_vasp(payee) &&
            (!VASP::is_vasp(payer) || !VASP::is_same_vasp(payee, payer))
        }
    }
    spec fun should_track_limits_for_account {
        pragma opaque;
        aborts_if false;
        ensures result == spec_should_track_limits_for_account<Token>(payer, payee, is_withdrawal);
    }
    spec module {
        define spec_has_published_account_limits<Token>(addr: address): bool {
            if (VASP::is_vasp(addr)) VASP::spec_has_account_limits<Token>(addr)
            else AccountLimits::has_window_published<Token>(addr)
        }
        define spec_should_track_limits_for_account<Token>(
            payer: address, payee: address, is_withdrawal: bool
        ): bool {
            if (is_withdrawal) {
                spec_has_published_account_limits<Token>(payer) &&
                VASP::is_vasp(payer) &&
                (!VASP::is_vasp(payee) || !VASP::spec_is_same_vasp(payer, payee))
            } else {
                spec_has_published_account_limits<Token>(payee) &&
                VASP::is_vasp(payee) &&
                (!VASP::is_vasp(payer) || !VASP::spec_is_same_vasp(payee, payer))
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
        LibraTimestamp::assert_operating();
        let cap_address = cap.account_address;
        // use the LBR reserve address as `payee_address`
        let payee_address = LBR::reserve_address();
        let (amount_coin1, amount_coin2) = LBR::calculate_component_amounts_for_lbr(amount_lbr);
        let coin1 = withdraw_from<Coin1>(cap, payee_address, amount_coin1, x"");
        let coin2 = withdraw_from<Coin2>(cap, payee_address, amount_coin2, x"");
        // Create `amount_lbr` LBR
        let lbr = LBR::create(amount_lbr, coin1, coin2);
        // use the reserved address as the payer for the LBR payment because the funds did not come
        // from an existing balance
        deposit(CoreAddresses::VM_RESERVED_ADDRESS(), cap_address, lbr, x"", x"");
    }
    spec fun staple_lbr {
        /// > TODO: timeout
        pragma verify = false;
    }

    /// Use `cap` to withdraw `amount_lbr`, burn the LBR, withdraw the corresponding assets from the
    /// LBR reserve, and deposit them to `cap.address`.
    /// The `payer` address in the` RecievedPaymentEvent`s emitted by this function will be the LBR
    /// reserve address to signify that this was a special payment that credits
    /// `cap.address`'s balance and credits the LBR reserve.
    public fun unstaple_lbr(cap: &WithdrawCapability, amount_lbr: u64)
    acquires LibraAccount, Balance, AccountOperationsCapability {
        LibraTimestamp::assert_operating();
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
        /// > TODO: timeout
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
        LibraTimestamp::assert_operating();
        AccountFreezing::assert_not_frozen(payee);

        // Check that the `to_deposit` coin is non-zero
        let deposit_value = Libra::value(&to_deposit);
        assert(deposit_value > 0, Errors::invalid_argument(ECOIN_DEPOSIT_IS_ZERO));
        // Check that an account exists at `payee`
        assert(exists_at(payee), Errors::not_published(EPAYEE_DOES_NOT_EXIST));
        // Check that `payee` can accept payments in `Token`
        assert(exists<Balance<Token>>(payee), Errors::invalid_argument(EPAYEE_CANT_ACCEPT_CURRENCY_TYPE));

        // Check that the payment complies with dual attestation rules
        DualAttestation::assert_payment_ok<Token>(
            payer, payee, deposit_value, copy metadata, metadata_signature
        );
        // Ensure that this deposit is compliant with the account limits on
        // this account.
        if (should_track_limits_for_account<Token>(payer, payee, false)) {
            assert(
                AccountLimits::update_deposit_limits<Token>(
                    deposit_value,
                    VASP::parent_address(payee),
                    &borrow_global<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()).limits_cap
                ),
                Errors::limit_exceeded(EDEPOSIT_EXCEEDS_LIMITS)
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
                metadata
            }
        );
    }
    spec fun deposit {
        // TODO: reactivate after aborts_if soundness fix.
        pragma verify = false;
        include DepositAbortsIf<Token>{amount: to_deposit.value};
        include DepositEnsures<Token>{amount: to_deposit.value};
    }
    spec schema DepositAbortsIf<Token> {
        payer: address;
        payee: address;
        amount: u64;
        metadata_signature: vector<u8>;
        metadata: vector<u8>;
        include LibraTimestamp::AbortsIfNotOperating;
        include AccountFreezing::AbortsIfFrozen{account: payee};
        aborts_if amount == 0 with Errors::INVALID_ARGUMENT;
        include DualAttestation::AssertPaymentOkAbortsIf<Token>{value: amount};
        include
            spec_should_track_limits_for_account<Token>(payer, payee, false) ==>
            AccountLimits::UpdateDepositLimitsAbortsIf<Token> {
                addr: VASP::spec_parent_address(payee),
            };
        aborts_if
            spec_should_track_limits_for_account<Token>(payer, payee, false) &&
                !AccountLimits::spec_update_deposit_limits<Token>(amount, VASP::spec_parent_address(payee))
            with Errors::LIMIT_EXCEEDED;
        aborts_if !exists<Balance<Token>>(payee) with Errors::INVALID_ARGUMENT;
        aborts_if global<Balance<Token>>(payee).coin.value + amount > max_u64() with Errors::LIMIT_EXCEEDED;
        aborts_if !exists_at(payee) with Errors::NOT_PUBLISHED;
        include Libra::AbortsIfNoCurrency<Token>;
    }
    spec schema DepositEnsures<Token> {
        payer: address;
        payee: address;
        amount: u64;
        ensures global<Balance<Token>>(payee).coin.value == old(global<Balance<Token>>(payee).coin.value) + amount;
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
        pragma verify_duration_estimate = 100;
    }

    // Cancel the burn request from `preburn_address` and return the funds.
    // Fails if the sender does not have a published MintCapability.
    public fun cancel_burn<Token>(
        account: &signer,
        preburn_address: address,
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        let coin = Libra::cancel_burn<Token>(account, preburn_address);
        // record both sender and recipient as `preburn_address`: the coins are moving from
        // `preburn_address`'s `Preburn` resource to its balance
        deposit(preburn_address, preburn_address, coin, x"", x"")
    }
    spec fun cancel_burn {
        // TODO: reactivate after aborts_if soundness fix.
        pragma verify = false;
    }

    /// Helper to withdraw `amount` from the given account balance and return the withdrawn Libra<Token>
    fun withdraw_from_balance<Token>(
        payer: address,
        payee: address,
        balance: &mut Balance<Token>,
        amount: u64
    ): Libra<Token> acquires AccountOperationsCapability {
        LibraTimestamp::assert_operating();
        AccountFreezing::assert_not_frozen(payer);
        // Make sure that this withdrawal is compliant with the limits on
        // the account if it's a inter-VASP transfer,
        if (should_track_limits_for_account<Token>(payer, payee, true)) {
            let can_withdraw = AccountLimits::update_withdrawal_limits<Token>(
                    amount,
                    VASP::parent_address(payer),
                    &borrow_global<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()).limits_cap
            );
            assert(can_withdraw, Errors::limit_exceeded(EWITHDRAWAL_EXCEEDS_LIMITS));
        };
        let coin = &mut balance.coin;
        // Abort if this withdrawal would make the `payer`'s balance go negative
        assert(Libra::value(coin) >= amount, Errors::limit_exceeded(EINSUFFICIENT_BALANCE));
        Libra::withdraw(coin, amount)
    }
    spec fun withdraw_from_balance {
        // TODO: reactivate after aborts_if soundness fix.
        pragma verify = false;
        include WithdrawFromBalanceAbortsIf<Token>;
        include WithdrawFromBalanceEnsures<Token>;
    }
    spec schema WithdrawFromBalanceAbortsIf<Token> {
        payer: address;
        payee: address;
        balance: Balance<Token>;
        amount: u64;
        include LibraTimestamp::AbortsIfNotOperating;
        include AccountFreezing::AbortsIfFrozen{account: payer};
        include
            spec_should_track_limits_for_account<Token>(payer, payee, true) ==>
            AccountLimits::UpdateWithdrawalLimitsAbortsIf<Token> {
                addr: VASP::spec_parent_address(payer),
            };
        aborts_if
            spec_should_track_limits_for_account<Token>(payer, payee, true) &&
            (   !spec_has_account_operations_cap() ||
                !AccountLimits::spec_update_withdrawal_limits<Token>(amount, VASP::spec_parent_address(payer))
            )
            with Errors::LIMIT_EXCEEDED;
        aborts_if balance.coin.value < amount with Errors::LIMIT_EXCEEDED;
    }
    spec schema WithdrawFromBalanceEnsures<Token> {
        balance: Balance<Token>;
        amount: u64;
        result: Libra<Token>;
        ensures balance.coin.value == old(balance.coin.value) - amount;
        ensures result.value == amount;
    }

    /// Withdraw `amount` `Libra<Token>`'s from the account balance under
    /// `cap.account_address`
    fun withdraw_from<Token>(
        cap: &WithdrawCapability,
        payee: address,
        amount: u64,
        metadata: vector<u8>,
    ): Libra<Token> acquires Balance, AccountOperationsCapability, LibraAccount {
        LibraTimestamp::assert_operating();
        let payer = cap.account_address;
        assert(exists_at(payer), Errors::not_published(EACCOUNT));
        assert(exists<Balance<Token>>(payer), Errors::not_published(EPAYER_DOESNT_HOLD_CURRENCY));
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

    /// Withdraw `amount` `Libra<Token>`'s from `cap.address` and send them to the `Preburn`
    /// resource under `dd`.
    public fun preburn<Token>(
        dd: &signer, cap: &WithdrawCapability, amount: u64
    ) acquires Balance, AccountOperationsCapability, LibraAccount {
        LibraTimestamp::assert_operating();
        Libra::preburn_to<Token>(dd, withdraw_from(cap, Signer::address_of(dd), amount, x""))
    }
    spec fun preburn {
        pragma verify_duration_estimate = 100;
    }

    /// Return a unique capability granting permission to withdraw from the sender's account balance.
    public fun extract_withdraw_capability(
        sender: &signer
    ): WithdrawCapability acquires LibraAccount {
        let sender_addr = Signer::address_of(sender);
        // Abort if we already extracted the unique withdraw capability for this account.
        assert(
            !delegated_withdraw_capability(sender_addr),
            Errors::invalid_state(EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED)
        );
        assert(exists_at(sender_addr), Errors::not_published(EACCOUNT));
        let account = borrow_global_mut<LibraAccount>(sender_addr);
        Option::extract(&mut account.withdrawal_capability)
    }

    /// Return the withdraw capability to the account it originally came from
    public fun restore_withdraw_capability(cap: WithdrawCapability)
    acquires LibraAccount {
        assert(exists_at(cap.account_address), Errors::not_published(EACCOUNT));
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

    /// Rotate the authentication key for the account under cap.account_address
    public fun rotate_authentication_key(
        cap: &KeyRotationCapability,
        new_authentication_key: vector<u8>,
    ) acquires LibraAccount  {
        assert(exists_at(cap.account_address), Errors::not_published(EACCOUNT));
        let sender_account_resource = borrow_global_mut<LibraAccount>(cap.account_address);
        // Don't allow rotating to clearly invalid key
        assert(Vector::length(&new_authentication_key) == 32, Errors::invalid_argument(EMALFORMED_AUTHENTICATION_KEY));
        sender_account_resource.authentication_key = new_authentication_key;
    }
    spec fun rotate_authentication_key {
        include RotateAuthenticationKeyAbortsIf;
        ensures global<LibraAccount>(cap.account_address).authentication_key == new_authentication_key;
    }
    spec schema RotateAuthenticationKeyAbortsIf {
        cap: &KeyRotationCapability;
        new_authentication_key: vector<u8>;
        aborts_if !exists_at(cap.account_address) with Errors::NOT_PUBLISHED;
        aborts_if len(new_authentication_key) != 32 with Errors::INVALID_ARGUMENT;
    }
    spec define spec_rotate_authentication_key(addr: address, new_authentication_key: vector<u8>): bool {
        global<LibraAccount>(addr).authentication_key == new_authentication_key
    }


    /// Return a unique capability granting permission to rotate the sender's authentication key
    public fun extract_key_rotation_capability(account: &signer): KeyRotationCapability
    acquires LibraAccount {
        let account_address = Signer::address_of(account);
        // Abort if we already extracted the unique key rotation capability for this account.
        assert(
            !delegated_key_rotation_capability(account_address),
             Errors::invalid_state(EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED)
        );
        assert(exists_at(account_address), Errors::not_published(EACCOUNT));
        let account = borrow_global_mut<LibraAccount>(account_address);
        Option::extract(&mut account.key_rotation_capability)
    }
    spec fun extract_key_rotation_capability {
        let account_addr = Signer::spec_address_of(account);
        aborts_if !exists_at(account_addr) with Errors::NOT_PUBLISHED;
        aborts_if delegated_key_rotation_capability(account_addr) with Errors::INVALID_STATE;
        ensures delegated_key_rotation_capability(account_addr);
    }

    /// Return the key rotation capability to the account it originally came from
    public fun restore_key_rotation_capability(cap: KeyRotationCapability)
    acquires LibraAccount {
        assert(exists_at(cap.account_address), Errors::not_published(EACCOUNT));
        let account = borrow_global_mut<LibraAccount>(cap.account_address);
        Option::fill(&mut account.key_rotation_capability, cap)
    }
    spec fun restore_key_rotation_capability {
        aborts_if !exists_at(cap.account_address) with Errors::NOT_PUBLISHED;
        aborts_if !delegated_key_rotation_capability(cap.account_address) with Errors::INVALID_ARGUMENT;
        ensures spec_holds_own_key_rotation_cap(cap.account_address);
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
        assert(
            new_account_addr != CoreAddresses::VM_RESERVED_ADDRESS(),
            Errors::invalid_argument(ECANNOT_CREATE_AT_VM_RESERVED)
        );

        // (1) publish LibraAccount
        let authentication_key = auth_key_prefix;
        Vector::append(
            &mut authentication_key, LCS::to_bytes(Signer::borrow_address(&new_account))
        );
        assert(Vector::length(&authentication_key) == 32, Errors::invalid_argument(EMALFORMED_AUTHENTICATION_KEY));
        assert(!exists_at(new_account_addr), Errors::already_published(EACCOUNT));
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
            }
        );
        AccountFreezing::create(&new_account);
        destroy_signer(new_account);
    }

    /// Creates the libra root account in genesis.
    public fun create_libra_root_account(
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        LibraTimestamp::assert_genesis();
        let new_account = create_signer(new_account_address);
        CoreAddresses::assert_libra_root(&new_account);
        SlidingNonce::publish_nonce_resource(&new_account, &new_account);
        make_account(new_account, auth_key_prefix)
    }

    /// Create a treasury/compliance account at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`
    public fun create_treasury_compliance_account(
        lr_account: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
    ) {
        LibraTimestamp::assert_genesis();
        Roles::assert_libra_root(lr_account);
        let new_account = create_signer(new_account_address);
        SlidingNonce::publish_nonce_resource(lr_account, &new_account);
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
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
        add_all_currencies: bool,
    ) {
        let new_dd_account = create_signer(new_account_address);
        Event::publish_generator(&new_dd_account);
        Roles::new_designated_dealer_role(creator_account, &new_dd_account);
        DesignatedDealer::publish_designated_dealer_credential<CoinType>(&new_dd_account, creator_account, add_all_currencies);
        add_currencies_for_account<CoinType>(&new_dd_account, add_all_currencies);
        DualAttestation::publish_credential(&new_dd_account, creator_account, human_name);
        make_account(new_dd_account, auth_key_prefix)
    }
    spec fun create_designated_dealer {
        // TODO(wrwg): timeout
        pragma verify = false;
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
        add_all_currencies: bool
    ) {
        // TODO: restrictions on creator_account?
        let new_account = create_signer(new_account_address);
        Roles::new_parent_vasp_role(creator_account, &new_account);
        VASP::publish_parent_vasp_credential(&new_account, creator_account);
        DualAttestation::publish_credential(&new_account, creator_account, human_name);
        Event::publish_generator(&new_account);
        add_currencies_for_account<Token>(&new_account, add_all_currencies);
        make_account(new_account, auth_key_prefix)
    }
    spec fun create_parent_vasp_account {
        // TODO: reactivate after aborts_if soundness fix.
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
        // TODO: reactivate after aborts_if soundness fix.
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

    /// Add a balance of `Token` type to the sending account
    public fun add_currency<Token>(account: &signer) {
        // aborts if `Token` is not a currency type in the system
        Libra::assert_is_currency<Token>();
        // Check that an account with this role is allowed to hold funds
        assert(Roles::can_hold_balance(account), Errors::invalid_argument(EROLE_CANT_STORE_BALANCE));
        // aborts if this account already has a balance in `Token`
        let addr = Signer::address_of(account);
        assert(!exists<Balance<Token>>(addr), Errors::already_published(EADD_EXISTING_CURRENCY));

        move_to(account, Balance<Token>{ coin: Libra::zero<Token>() })
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
        assert(exists_at(addr), Errors::not_published(EACCOUNT));
        sequence_number_for_account(borrow_global<LibraAccount>(addr))
    }

    /// Return the authentication key for this account
    public fun authentication_key(addr: address): vector<u8> acquires LibraAccount {
        assert(exists_at(addr), Errors::not_published(EACCOUNT));
        *&borrow_global<LibraAccount>(addr).authentication_key
    }

    /// Return true if the account at `addr` has delegated its key rotation capability
    public fun delegated_key_rotation_capability(addr: address): bool
    acquires LibraAccount {
        assert(exists_at(addr), Errors::not_published(EACCOUNT));
        Option::is_none(&borrow_global<LibraAccount>(addr).key_rotation_capability)
    }

    /// Return true if the account at `addr` has delegated its withdraw capability
    public fun delegated_withdraw_capability(addr: address): bool
    acquires LibraAccount {
        assert(exists_at(addr), Errors::not_published(EACCOUNT));
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

    /// The prologue for module transaction
    fun module_prologue<Token>(
        sender: &signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
    ) acquires LibraAccount, Balance {
        assert(
            LibraTransactionPublishingOption::is_module_allowed(sender),
            EPROLOGUE_MODULE_NOT_ALLOWED
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
    ) acquires LibraAccount, Balance {
        assert(
            LibraTransactionPublishingOption::is_script_allowed(sender, &script_hash),
            EPROLOGUE_SCRIPT_NOT_ALLOWED
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
        txn_expiration_time: u64,
        chain_id: u8,
    ) acquires LibraAccount, Balance {
        let transaction_sender = Signer::address_of(sender);

        // Check that the chain ID stored on-chain matches the chain ID specified by the transaction
        assert(ChainId::get() == chain_id, EPROLOGUE_BAD_CHAIN_ID);

        // Verify that the transaction sender's account exists
        assert(exists_at(transaction_sender), EPROLOGUE_ACCOUNT_DNE);

        // We check whether this account is frozen, if it is no transaction can be sent from it.
        assert(
            !AccountFreezing::account_is_frozen(transaction_sender),
            EPROLOGUE_ACCOUNT_FROZEN
        );

        // Load the transaction sender's account
        let sender_account = borrow_global_mut<LibraAccount>(transaction_sender);

        // Check that the hash of the transaction's public key matches the account's auth key
        assert(
            Hash::sha3_256(txn_public_key) == *&sender_account.authentication_key,
            EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY
        );

        // Check that the account has enough balance for all of the gas
        assert(
            (txn_gas_price as u128) * (txn_max_gas_units as u128) <= MAX_U64,
             EPROLOGUE_CANT_PAY_GAS_DEPOSIT
        );
        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        // Don't grab the balance if the transaction fee is zero
        if (max_transaction_fee > 0) {
            let balance_amount = balance<Token>(transaction_sender);
            assert(balance_amount >= max_transaction_fee, EPROLOGUE_CANT_PAY_GAS_DEPOSIT);
        };

        // Check that the transaction sequence number matches the sequence number of the account
        // TODO: the below assertions overlap, fix this.
        assert(
            txn_sequence_number >= sender_account.sequence_number,
            EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD
        );
        assert(
            txn_sequence_number == sender_account.sequence_number,
            EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW
        );
        assert(
            LibraTransactionTimeout::is_valid_transaction_timestamp(txn_expiration_time),
            EPROLOGUE_TRANSACTION_EXPIRED
        );
    }

    /// Collects gas and bumps the sequence number for executing a transaction
    fun epilogue<Token>(
        sender: address,
        transaction_fee_amount: u64,
        txn_sequence_number: u64,
    ) acquires LibraAccount, Balance, AccountOperationsCapability {
        // Load the transaction sender's account and balance resources
        assert(exists_at(sender), Errors::not_published(EACCOUNT));
        let sender_account = borrow_global_mut<LibraAccount>(sender);

        // Bump the sequence number
        assert(sender_account.sequence_number < (MAX_U64 as u64), Errors::limit_exceeded(ESEQUENCE_NUMBER));
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
        assert(txn_max_gas_units >= gas_units_remaining, Errors::invalid_argument(EGAS));
        let gas_used = txn_max_gas_units - gas_units_remaining;
        assert((txn_gas_price as u128) * (gas_used as u128) <= MAX_U64, Errors::limit_exceeded(EGAS));
        let transaction_fee_amount = txn_gas_price * gas_used;

        // Load the transaction sender's balance resource only if it exists. If it doesn't we default the value to 0
        let sender_balance = if (exists<Balance<Token>>(sender)) balance<Token>(sender) else 0;
        assert(sender_balance >= transaction_fee_amount, EPROLOGUE_CANT_PAY_GAS_DEPOSIT);
        epilogue<Token>(sender, transaction_fee_amount, txn_sequence_number);
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
        assert(txn_max_gas_units >= gas_units_remaining, Errors::invalid_argument(EGAS));
        let gas_used = txn_max_gas_units - gas_units_remaining;
        assert((txn_gas_price as u128) * (gas_used as u128) <= MAX_U64, Errors::limit_exceeded(EGAS));
        let transaction_fee_amount = txn_gas_price * gas_used;

        epilogue<Token>(sender, transaction_fee_amount, txn_sequence_number);
    }

    /// Bump the sequence number of an account. This function should be used only for bumping the sequence number when
    /// a writeset transaction is committed.
    fun bump_sequence_number(signer: &signer) acquires LibraAccount {
        let addr = Signer::address_of(signer);
        assert(exists_at(addr), Errors::not_published(EACCOUNT));
        let sender_account = borrow_global_mut<LibraAccount>(addr);
        sender_account.sequence_number = sender_account.sequence_number + 1;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Proof of concept code used for Validator and ValidatorOperator roles management
    ///////////////////////////////////////////////////////////////////////////

    public fun create_validator_account(
        creator_account: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
    ) {
        let new_account = create_signer(new_account_address);
        // The creator account is verified to have the libra root role in `Roles::new_validator_role`
        Roles::new_validator_role(creator_account, &new_account);
        Event::publish_generator(&new_account);
        ValidatorConfig::publish(&new_account, creator_account, human_name);
        make_account(new_account, auth_key_prefix)
    }

    public fun create_validator_operator_account(
        creator_account: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
    ) {
        let new_account = create_signer(new_account_address);
        // The creator account is verified to have the libra root role in `Roles::new_validator_operator_role`
        Roles::new_validator_operator_role(creator_account, &new_account);
        Event::publish_generator(&new_account);
        ValidatorOperatorConfig::publish(&new_account, creator_account, human_name);
        make_account(new_account, auth_key_prefix)
    }

    ///////////////////////////////////////////////////////////////////////////
    // End of the proof of concept code
    ///////////////////////////////////////////////////////////////////////////

    // ****************** SPECIFICATIONS *******************

    spec module {
        pragma verify;

        /// Returns field `key_rotation_capability` of the
        /// LibraAccount under `addr`.
        define spec_get_key_rotation_cap(addr: address): Option<KeyRotationCapability> {
            global<LibraAccount>(addr).key_rotation_capability
        }

        /// Returns true if the LibraAccount at `addr` holds
        /// `KeyRotationCapability` for itself.
        define spec_holds_own_key_rotation_cap(addr: address): bool {
            Option::is_some(spec_get_key_rotation_cap(addr))
            && addr == Option::borrow(
                spec_get_key_rotation_cap(addr)).account_address
        }

        /// Returns true if `AccountOperationsCapability` is published.
        define spec_has_account_operations_cap(): bool {
            exists<AccountOperationsCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS())
        }

        define spec_has_key_rotation_cap(addr: address): bool {
            Option::is_some(global<LibraAccount>(addr).key_rotation_capability)
        }

        /// Returns field `withdrawal_capability` of LibraAccount under `addr`.
        define spec_get_withdraw_cap(addr: address): Option<WithdrawCapability> {
            global<LibraAccount>(addr).withdrawal_capability
        }

        /// Returns true if the LibraAccount at `addr` holds a
        /// `WithdrawCapability`.
        define spec_has_withdraw_cap(addr: address): bool {
            Option::is_some(spec_get_withdraw_cap(addr))
        }

        /// Returns true if the LibraAccount at `addr` holds
        /// `WithdrawCapability` for itself.
        define spec_holds_own_withdraw_cap(addr: address): bool {
            spec_has_withdraw_cap(addr)
            && addr == Option::borrow(spec_get_withdraw_cap(addr)).account_address
        }
    }

    spec schema EnsuresHasKeyRotationCap {
        account: signer;
        ensures spec_has_key_rotation_cap(Signer::spec_address_of(account));
    }

    spec schema EnsuresWithdrawalCap {
        account: signer;
        ensures spec_has_withdraw_cap(Signer::spec_address_of(account));
    }

    spec module {
        /// the permission "RotateAuthenticationKey(addr)" is granted to the account at addr [B27].
        apply EnsuresHasKeyRotationCap{account: new_account} to make_account;

        /// the permission "WithdrawalCapability(addr)" is granted to the account at addr [B28].
        apply EnsuresWithdrawalCap{account: new_account} to make_account;
    }

    spec module {
        /// The LibraAccount under addr holds either no withdraw capability
        /// (withdraw cap has been delegated) or the withdraw capability for addr itself.
        invariant [global] forall addr1: address where exists_at(addr1):
            delegated_withdraw_capability(addr1) || spec_holds_own_withdraw_cap(addr1);

        /// The LibraAccount under addr holds either no key rotation capability
        /// (key rotation cap has been delegated) or the key rotation capability for addr itself.
        invariant [global] forall addr1: address where exists_at(addr1):
            delegated_key_rotation_capability(addr1) || spec_holds_own_key_rotation_cap(addr1);
    }

}
}
