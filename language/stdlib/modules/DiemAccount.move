address 0x1 {

/// The `DiemAccount` module manages accounts. It defines the `DiemAccount` resource and
/// numerous auxiliary data structures. It also defines the prolog and epilog that run
/// before and after every transaction.

module DiemAccount {
    use 0x1::AccountFreezing;
    use 0x1::CoreAddresses;
    use 0x1::ChainId;
    use 0x1::AccountLimits::{Self, AccountLimitMutationCapability};
    use 0x1::XUS::XUS;
    use 0x1::DualAttestation;
    use 0x1::Errors;
    use 0x1::Event::{Self, EventHandle};
    use 0x1::Hash;
    use 0x1::XDX::XDX;
    use 0x1::BCS;
    use 0x1::DiemConfig;
    use 0x1::DiemTimestamp;
    use 0x1::DiemTransactionPublishingOption;
    use 0x1::Signer;
    use 0x1::SlidingNonce;
    use 0x1::TransactionFee;
    use 0x1::ValidatorConfig;
    use 0x1::ValidatorOperatorConfig;
    use 0x1::VASP;
    use 0x1::Vector;
    use 0x1::DesignatedDealer;
    use 0x1::Diem::{Self, Diem};
    use 0x1::Option::{Self, Option};
    use 0x1::Roles;

    /// An `address` is a Diem Account iff it has a published DiemAccount resource.
    resource struct DiemAccount {
        /// The current authentication key.
        /// This can be different from the key used to create the account
        authentication_key: vector<u8>,
        /// A `withdraw_capability` allows whoever holds this capability
        /// to withdraw from the account. At the time of account creation
        /// this capability is stored in this option. It can later be removed
        /// by `extract_withdraw_capability` and also restored via `restore_withdraw_capability`.
        withdraw_capability: Option<WithdrawCapability>,
        /// A `key_rotation_capability` allows whoever holds this capability
        /// the ability to rotate the authentication key for the account. At
        /// the time of account creation this capability is stored in this
        /// option. It can later be "extracted" from this field via
        /// `extract_key_rotation_capability`, and can also be restored via
        /// `restore_key_rotation_capability`.
        key_rotation_capability: Option<KeyRotationCapability>,
        /// Event handle to which ReceivePaymentEvents are emitted when
        /// payments are received.
        received_events: EventHandle<ReceivedPaymentEvent>,
        /// Event handle to which SentPaymentEvents are emitted when
        /// payments are sent.
        sent_events: EventHandle<SentPaymentEvent>,
        /// The current sequence number of the account.
        /// Incremented by one each time a transaction is submitted by
        /// this account.
        sequence_number: u64,
    }

    /// A resource that holds the total value of currency of type `Token`
    /// currently held by the account.
    resource struct Balance<Token> {
        /// Stores the value of the balance in its balance field. A coin has
        /// a `value` field. The amount of money in the balance is changed
        /// by modifying this field.
        coin: Diem<Token>,
    }

    /// The holder of WithdrawCapability for account_address can withdraw Diem from
    /// account_address/DiemAccount/balance.
    /// There is at most one WithdrawCapability in existence for a given address.
    resource struct WithdrawCapability {
        /// Address that WithdrawCapability was associated with when it was created.
        /// This field does not change.
        account_address: address,
    }

    /// The holder of KeyRotationCapability for account_address can rotate the authentication key for
    /// account_address (i.e., write to account_address/DiemAccount/authentication_key).
    /// There is at most one KeyRotationCapability in existence for a given address.
    resource struct KeyRotationCapability {
        /// Address that KeyRotationCapability was associated with when it was created.
        /// This field does not change.
        account_address: address,
    }

    /// A wrapper around an `AccountLimitMutationCapability` which is used to check for account limits
    /// and to record freeze/unfreeze events.
    resource struct AccountOperationsCapability {
        limits_cap: AccountLimitMutationCapability,
        creation_events: Event::EventHandle<CreateAccountEvent>,
    }

    /// A resource that holds the event handle for all the past WriteSet transactions that have been committed on chain.
    resource struct DiemWriteSetManager {
        upgrade_events: Event::EventHandle<Self::AdminTransactionEvent>,
    }


    /// Message for sent events
    struct SentPaymentEvent {
        /// The amount of Diem<Token> sent
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
        /// The amount of Diem<Token> received
        amount: u64,
        /// The code symbol for the currency that was received
        currency_code: vector<u8>,
        /// The address that sent the coin
        payer: address,
        /// Metadata associated with the payment
        metadata: vector<u8>,
    }

    /// Message for committed WriteSet transaction.
    struct AdminTransactionEvent {
        // The block time when this WriteSet is committed.
        committed_timestamp_secs: u64,
    }

    /// Message for creation of a new account
    struct CreateAccountEvent {
        /// Address of the created account
        created: address,
        /// Role of the created account
        role_id: u64
    }

    const MAX_U64: u128 = 18446744073709551615;

    /// The `DiemAccount` resource is not in the required state
    const EACCOUNT: u64 = 0;
    /// The account's sequence number has exceeded the maximum representable value
    const ESEQUENCE_NUMBER: u64 = 1;
    /// Tried to deposit a coin whose value was zero
    const ECOIN_DEPOSIT_IS_ZERO: u64 = 2;
    /// Tried to deposit funds that would have surpassed the account's limits
    const EDEPOSIT_EXCEEDS_LIMITS: u64 = 3;
    /// Tried to create a balance for an account whose role does not allow holding balances
    const EROLE_CANT_STORE_BALANCE: u64 = 4;
    /// The account does not hold a large enough balance in the specified currency
    const EINSUFFICIENT_BALANCE: u64 = 5;
    /// The withdrawal of funds would have exceeded the the account's limits
    const EWITHDRAWAL_EXCEEDS_LIMITS: u64 = 6;
    /// The `WithdrawCapability` for this account has already been extracted
    const EWITHDRAW_CAPABILITY_ALREADY_EXTRACTED: u64 = 7;
    /// The provided authentication had an invalid length
    const EMALFORMED_AUTHENTICATION_KEY: u64 = 8;
    /// The `KeyRotationCapability` for this account has already been extracted
    const EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED: u64 = 9;
    /// An account cannot be created at the reserved VM address of 0x0
    const ECANNOT_CREATE_AT_VM_RESERVED: u64 = 10;
    /// The `WithdrawCapability` for this account is not extracted
    const EWITHDRAW_CAPABILITY_NOT_EXTRACTED: u64 = 11;
    /// Tried to add a balance in a currency that this account already has
    const EADD_EXISTING_CURRENCY: u64 = 15;
    /// Attempted to send funds to an account that does not exist
    const EPAYEE_DOES_NOT_EXIST: u64 = 17;
    /// Attempted to send funds in a currency that the receiving account does not hold.
    /// e.g., `Diem<XDX>` to an account that exists, but does not have a `Balance<XDX>` resource
    const EPAYEE_CANT_ACCEPT_CURRENCY_TYPE: u64 = 18;
    /// Tried to withdraw funds in a currency that the account does hold
    const EPAYER_DOESNT_HOLD_CURRENCY: u64 = 19;
    /// An invalid amount of gas units was provided for execution of the transaction
    const EGAS: u64 = 20;
    /// The `AccountOperationsCapability` was not in the required state
    const EACCOUNT_OPERATIONS_CAPABILITY: u64 = 22;
    /// The `DiemWriteSetManager` was not in the required state
    const EWRITESET_MANAGER: u64 = 23;
    /// An account cannot be created at the reserved core code address of 0x1
    const ECANNOT_CREATE_AT_CORE_CODE: u64 = 24;

    /// Prologue errors. These are separated out from the other errors in this
    /// module since they are mapped separately to major VM statuses, and are
    /// important to the semantics of the system. Those codes also need to be
    /// directly used in aborts instead of augmenting them with a category
    /// via the `Errors` module.
    const PROLOGUE_EACCOUNT_FROZEN: u64 = 1000;
    const PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY: u64 = 1001;
    const PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD: u64 = 1002;
    const PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW: u64 = 1003;
    const PROLOGUE_EACCOUNT_DNE: u64 = 1004;
    const PROLOGUE_ECANT_PAY_GAS_DEPOSIT: u64 = 1005;
    const PROLOGUE_ETRANSACTION_EXPIRED: u64 = 1006;
    const PROLOGUE_EBAD_CHAIN_ID: u64 = 1007;
    const PROLOGUE_ESCRIPT_NOT_ALLOWED: u64 = 1008;
    const PROLOGUE_EMODULE_NOT_ALLOWED: u64 = 1009;
    const PROLOGUE_INVALID_WRITESET_SENDER: u64 = 1010;
    const PROLOGUE_ESEQUENCE_NUMBER_TOO_BIG: u64 = 1011;
    const EPILOGUE_INVALID_WRITESET_SENDER: u64 = 1012;

    /// Initialize this module. This is only callable from genesis.
    public fun initialize(
        dr_account: &signer,
        dummy_auth_key_prefix: vector<u8>,
    ) acquires AccountOperationsCapability {
        DiemTimestamp::assert_genesis();
        // Operational constraint, not a privilege constraint.
        CoreAddresses::assert_diem_root(dr_account);

        create_diem_root_account(
            copy dummy_auth_key_prefix,
        );
        create_treasury_compliance_account(
            dr_account,
            copy dummy_auth_key_prefix,
        );
    }

    spec fun initialize {
        pragma opaque;
        include CoreAddresses::AbortsIfNotDiemRoot{account: dr_account};
        include CreateDiemRootAccountAbortsIf{auth_key_prefix: dummy_auth_key_prefix};
        include CreateTreasuryComplianceAccountAbortsIf{auth_key_prefix: dummy_auth_key_prefix};
        aborts_if exists<AccountFreezing::FreezingBit>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS())
            with Errors::ALREADY_PUBLISHED;

        // modifies and ensures needed to make this function opaque.
        include CreateDiemRootAccountModifies;
        include CreateDiemRootAccountEnsures;
        include CreateTreasuryComplianceAccountModifies;
        include CreateTreasuryComplianceAccountEnsures;
    }

    /// Return `true` if `addr` has already published account limits for `Token`
    fun has_published_account_limits<Token>(addr: address): bool {
        if (VASP::is_vasp(addr)) {
            VASP::has_account_limits<Token>(addr)
        }
        else {
            AccountLimits::has_window_published<Token>(addr)
        }
    }
    spec define spec_has_published_account_limits<Token>(addr: address): bool {
        if (VASP::is_vasp(addr)) VASP::spec_has_account_limits<Token>(addr)
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
            !VASP::is_same_vasp(payer, payee)
        } else {
            has_published_account_limits<Token>(payee) &&
            VASP::is_vasp(payee) &&
            !VASP::is_same_vasp(payee, payer)
        }
    }
    spec fun should_track_limits_for_account {
        pragma opaque;
        aborts_if false;
        ensures result == spec_should_track_limits_for_account<Token>(payer, payee, is_withdrawal);
    }
    spec define spec_should_track_limits_for_account<Token>(
        payer: address, payee: address, is_withdrawal: bool
    ): bool {
        if (is_withdrawal) {
            spec_has_published_account_limits<Token>(payer) &&
            VASP::is_vasp(payer) &&
            !VASP::spec_is_same_vasp(payer, payee)
        } else {
            spec_has_published_account_limits<Token>(payee) &&
            VASP::is_vasp(payee) &&
            !VASP::spec_is_same_vasp(payee, payer)
        }
    }

    /// Record a payment of `to_deposit` from `payer` to `payee` with the attached `metadata`
    fun deposit<Token>(
        payer: address,
        payee: address,
        to_deposit: Diem<Token>,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires DiemAccount, Balance, AccountOperationsCapability {
        DiemTimestamp::assert_operating();
        AccountFreezing::assert_not_frozen(payee);

        // Check that the `to_deposit` coin is non-zero
        let deposit_value = Diem::value(&to_deposit);
        assert(deposit_value > 0, Errors::invalid_argument(ECOIN_DEPOSIT_IS_ZERO));
        // Check that an account exists at `payee`
        assert(exists_at(payee), Errors::not_published(EPAYEE_DOES_NOT_EXIST));
        // Check that `payee` can accept payments in `Token`
        assert(
            exists<Balance<Token>>(payee),
            Errors::invalid_argument(EPAYEE_CANT_ACCEPT_CURRENCY_TYPE)
        );

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
                    &borrow_global<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS()).limits_cap
                ),
                Errors::limit_exceeded(EDEPOSIT_EXCEEDS_LIMITS)
            )
        };

        // Deposit the `to_deposit` coin
        Diem::deposit(&mut borrow_global_mut<Balance<Token>>(payee).coin, to_deposit);

        // Log a received event
        Event::emit_event<ReceivedPaymentEvent>(
            &mut borrow_global_mut<DiemAccount>(payee).received_events,
            ReceivedPaymentEvent {
                amount: deposit_value,
                currency_code: Diem::currency_code<Token>(),
                payer,
                metadata
            }
        );
    }
    spec fun deposit {
        pragma opaque;
        modifies global<Balance<Token>>(payee);
        modifies global<DiemAccount>(payee);
        modifies global<AccountLimits::Window<Token>>(VASP::spec_parent_address(payee));
        ensures exists<DiemAccount>(payee);
        ensures exists<Balance<Token>>(payee);
        ensures global<DiemAccount>(payee).withdraw_capability
            == old(global<DiemAccount>(payee).withdraw_capability);
        include DepositAbortsIf<Token>{amount: to_deposit.value};
        include DepositOverflowAbortsIf<Token>{amount: to_deposit.value};
        include DepositEnsures<Token>{amount: to_deposit.value};
    }
    spec schema DepositAbortsIf<Token> {
        payer: address;
        payee: address;
        amount: u64;
        metadata_signature: vector<u8>;
        metadata: vector<u8>;
        include DepositAbortsIfRestricted<Token>;
        include AccountFreezing::AbortsIfFrozen{account: payee};
        aborts_if !exists<Balance<Token>>(payee) with Errors::INVALID_ARGUMENT;
        aborts_if !exists_at(payee) with Errors::NOT_PUBLISHED;
    }
    spec schema DepositOverflowAbortsIf<Token> {
        payee: address;
        amount: u64;
        aborts_if balance<Token>(payee) + amount > max_u64() with Errors::LIMIT_EXCEEDED;
    }
    spec schema DepositAbortsIfRestricted<Token> {
        payer: address;
        payee: address;
        amount: u64;
        metadata_signature: vector<u8>;
        metadata: vector<u8>;
        include DiemTimestamp::AbortsIfNotOperating;
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
        include Diem::AbortsIfNoCurrency<Token>;
    }
    spec schema DepositEnsures<Token> {
        payee: address;
        amount: u64;
        ensures balance<Token>(payee) == old(balance<Token>(payee)) + amount;
    }

    /// Mint 'mint_amount' to 'designated_dealer_address' for 'tier_index' tier.
    /// Max valid tier index is 3 since there are max 4 tiers per DD.
    /// Sender should be treasury compliance account and receiver authorized DD.
    public fun tiered_mint<Token>(
        tc_account: &signer,
        designated_dealer_address: address,
        mint_amount: u64,
        tier_index: u64,
    ) acquires DiemAccount, Balance, AccountOperationsCapability {
        let coin = DesignatedDealer::tiered_mint<Token>(
            tc_account, mint_amount, designated_dealer_address, tier_index
        );
        // Use the reserved address as the payer because the funds did not come from an existing
        // balance
        deposit(CoreAddresses::VM_RESERVED_ADDRESS(), designated_dealer_address, coin, x"", x"")
    }

    spec fun tiered_mint {
        pragma opaque;
        modifies global<Balance<Token>>(designated_dealer_address);
        modifies global<Diem::CurrencyInfo<Token>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        include TieredMintAbortsIf<Token>;
        include TieredMintEnsures<Token>;
    }

    spec schema TieredMintAbortsIf<Token> {
        tc_account: signer;
        designated_dealer_address: address;
        mint_amount: u64;
        tier_index: u64;
        include DesignatedDealer::TieredMintAbortsIf<Token>{dd_addr: designated_dealer_address, amount: mint_amount};
        include DepositAbortsIf<Token>{payer: CoreAddresses::VM_RESERVED_ADDRESS(),
            payee: designated_dealer_address, amount: mint_amount, metadata: x"", metadata_signature: x""};
        include DepositOverflowAbortsIf<Token>{payee: designated_dealer_address, amount: mint_amount};
    }

    spec schema TieredMintEnsures<Token> {
        designated_dealer_address: address;
        mint_amount: u64;
        let dealer_balance = global<Balance<Token>>(designated_dealer_address).coin.value;
        let currency_info = global<Diem::CurrencyInfo<Token>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        /// Total value of the currency increases by `amount`.
        ensures currency_info == update_field(old(currency_info), total_value, old(currency_info.total_value) + mint_amount);
        /// The balance of designated dealer increases by `amount`.
        ensures dealer_balance == old(dealer_balance) + mint_amount;
    }

    // Cancel the burn request from `preburn_address` and return the funds.
    // Fails if the sender does not have a published MintCapability.
    public fun cancel_burn<Token>(
        account: &signer,
        preburn_address: address,
    ) acquires DiemAccount, Balance, AccountOperationsCapability {
        let coin = Diem::cancel_burn<Token>(account, preburn_address);
        // record both sender and recipient as `preburn_address`: the coins are moving from
        // `preburn_address`'s `Preburn` resource to its balance
        deposit(preburn_address, preburn_address, coin, x"", x"")
    }

    spec fun cancel_burn {
        include CancelBurnAbortsIf<Token>;
        include Diem::CancelBurnWithCapEnsures<Token>;
        let preburn_value_at_addr = global<Diem::Preburn<Token>>(preburn_address).to_burn.value;
        let balance_at_addr = balance<Token>(preburn_address);
        ensures balance_at_addr == old(balance_at_addr) + old(preburn_value_at_addr);
    }

    spec schema CancelBurnAbortsIf<Token> {
        account: signer;
        preburn_address: address;
        let amount = global<Diem::Preburn<Token>>(preburn_address).to_burn.value;
        aborts_if !exists<Diem::BurnCapability<Token>>(Signer::spec_address_of(account))
            with Errors::REQUIRES_CAPABILITY;
        include Diem::CancelBurnWithCapAbortsIf<Token>;
        include DepositAbortsIf<Token>{
            payer: preburn_address,
            payee: preburn_address,
            amount: amount,
            metadata: x"",
            metadata_signature: x""
        };
        include DepositOverflowAbortsIf<Token>{payee: preburn_address, amount: amount};
    }

    /// Helper to withdraw `amount` from the given account balance and return the withdrawn Diem<Token>
    fun withdraw_from_balance<Token>(
        payer: address,
        payee: address,
        balance: &mut Balance<Token>,
        amount: u64
    ): Diem<Token> acquires AccountOperationsCapability {
        DiemTimestamp::assert_operating();
        AccountFreezing::assert_not_frozen(payer);
        // Make sure that this withdrawal is compliant with the limits on
        // the account if it's a inter-VASP transfer,
        if (should_track_limits_for_account<Token>(payer, payee, true)) {
            let can_withdraw = AccountLimits::update_withdrawal_limits<Token>(
                    amount,
                    VASP::parent_address(payer),
                    &borrow_global<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS()).limits_cap
            );
            assert(can_withdraw, Errors::limit_exceeded(EWITHDRAWAL_EXCEEDS_LIMITS));
        };
        let coin = &mut balance.coin;
        // Abort if this withdrawal would make the `payer`'s balance go negative
        assert(Diem::value(coin) >= amount, Errors::limit_exceeded(EINSUFFICIENT_BALANCE));
        Diem::withdraw(coin, amount)
    }
    spec fun withdraw_from_balance {
        modifies global<AccountLimits::Window<Token>>(VASP::spec_parent_address(payer));
        include WithdrawFromBalanceAbortsIf<Token>;
        include WithdrawFromBalanceEnsures<Token>;
    }
    spec schema WithdrawFromBalanceAbortsIf<Token> {
        payer: address;
        payee: address;
        balance: Balance<Token>;
        amount: u64;
        include WithdrawFromBalanceNoLimitsAbortsIf<Token>;
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
    }
    spec schema WithdrawFromBalanceNoLimitsAbortsIf<Token> {
          payer: address;
          payee: address;
          balance: Balance<Token>;
          amount: u64;
          include DiemTimestamp::AbortsIfNotOperating;
          include AccountFreezing::AbortsIfFrozen{account: payer};
          aborts_if balance.coin.value < amount with Errors::LIMIT_EXCEEDED;
    }
    spec schema WithdrawFromBalanceEnsures<Token> {
        balance: Balance<Token>;
        amount: u64;
        result: Diem<Token>;
        ensures balance.coin.value == old(balance.coin.value) - amount;
        ensures result.value == amount;
    }

    /// Withdraw `amount` `Diem<Token>`'s from the account balance under
    /// `cap.account_address`
    fun withdraw_from<Token>(
        cap: &WithdrawCapability,
        payee: address,
        amount: u64,
        metadata: vector<u8>,
    ): Diem<Token> acquires Balance, AccountOperationsCapability, DiemAccount {
        DiemTimestamp::assert_operating();
        let payer = cap.account_address;
        assert(exists_at(payer), Errors::not_published(EACCOUNT));
        assert(exists<Balance<Token>>(payer), Errors::not_published(EPAYER_DOESNT_HOLD_CURRENCY));
        let account_balance = borrow_global_mut<Balance<Token>>(payer);
        // Load the payer's account and emit an event to record the withdrawal
        Event::emit_event<SentPaymentEvent>(
            &mut borrow_global_mut<DiemAccount>(payer).sent_events,
            SentPaymentEvent {
                amount,
                currency_code: Diem::currency_code<Token>(),
                payee,
                metadata
            },
        );
        withdraw_from_balance<Token>(payer, payee, account_balance, amount)
    }

    spec fun withdraw_from {
        let payer = cap.account_address;
        modifies global<Balance<Token>>(payer);
        modifies global<DiemAccount>(payer);
        ensures exists<DiemAccount>(payer);
        ensures global<DiemAccount>(payer).withdraw_capability
                    == old(global<DiemAccount>(payer).withdraw_capability);
        include WithdrawFromAbortsIf<Token>;
        include WithdrawFromBalanceEnsures<Token>{balance: global<Balance<Token>>(payer)};
        include WithdrawOnlyFromCapAddress<Token>;
    }

    spec schema WithdrawFromAbortsIf<Token> {
        cap: WithdrawCapability;
        payee: address;
        amount: u64;
        let payer = cap.account_address;
        include DiemTimestamp::AbortsIfNotOperating;
        include Diem::AbortsIfNoCurrency<Token>;
        include WithdrawFromBalanceAbortsIf<Token>{payer, balance: global<Balance<Token>>(payer)};
        aborts_if !exists_at(payer) with Errors::NOT_PUBLISHED;
        aborts_if !exists<Balance<Token>>(payer) with Errors::NOT_PUBLISHED;
    }

    /// # Access Control
    spec schema WithdrawOnlyFromCapAddress<Token> {
        cap: WithdrawCapability;
        /// Can only withdraw from the balances of cap.account_address [[H18]][PERMISSION].
        ensures forall addr: address where old(exists<Balance<Token>>(addr)) && addr != cap.account_address:
            balance<Token>(addr) == old(balance<Token>(addr));
    }

    /// Withdraw `amount` `Diem<Token>`'s from `cap.address` and send them to the `Preburn`
    /// resource under `dd`.
    public fun preburn<Token>(
        dd: &signer,
        cap: &WithdrawCapability,
        amount: u64
    ) acquires Balance, AccountOperationsCapability, DiemAccount {
        DiemTimestamp::assert_operating();
        Diem::preburn_to<Token>(dd, withdraw_from(cap, Signer::address_of(dd), amount, x""))
    }

    spec fun preburn {
        pragma opaque;
        let dd_addr = Signer::spec_address_of(dd);
        let payer = cap.account_address;
        modifies global<DiemAccount>(payer);
        ensures exists<DiemAccount>(payer);
        ensures global<DiemAccount>(payer).withdraw_capability
                == old(global<DiemAccount>(payer).withdraw_capability);
        include PreburnAbortsIf<Token>;
        include PreburnEnsures<Token>{dd_addr, payer};
    }

    spec schema PreburnAbortsIf<Token> {
        dd: signer;
        cap: WithdrawCapability;
        amount: u64;
        include DiemTimestamp::AbortsIfNotOperating{};
        include WithdrawFromAbortsIf<Token>{payee: Signer::spec_address_of(dd)};
        include Diem::PreburnToAbortsIf<Token>{account: dd};
    }

    spec schema PreburnEnsures<Token> {
        dd_addr: address;
        payer: address;
        let payer_balance = global<Balance<Token>>(payer).coin.value;
        let preburn = global<Diem::Preburn<Token>>(dd_addr);
        /// The balance of payer decreases by `amount`.
        ensures payer_balance == old(payer_balance) - amount;
        /// The value of preburn at `dd_addr` increases by `amount`;
        include Diem::PreburnEnsures<Token>{preburn: preburn};
    }

    /// Return a unique capability granting permission to withdraw from the sender's account balance.
    public fun extract_withdraw_capability(
        sender: &signer
    ): WithdrawCapability acquires DiemAccount {
        let sender_addr = Signer::address_of(sender);
        // Abort if we already extracted the unique withdraw capability for this account.
        assert(
            !delegated_withdraw_capability(sender_addr),
            Errors::invalid_state(EWITHDRAW_CAPABILITY_ALREADY_EXTRACTED)
        );
        assert(exists_at(sender_addr), Errors::not_published(EACCOUNT));
        let account = borrow_global_mut<DiemAccount>(sender_addr);
        Option::extract(&mut account.withdraw_capability)
    }

    spec fun extract_withdraw_capability {
        pragma opaque;
        let sender_addr = Signer::spec_address_of(sender);
        modifies global<DiemAccount>(sender_addr);
        include ExtractWithdrawCapAbortsIf{sender_addr};
        ensures exists<DiemAccount>(sender_addr);
        ensures result == old(spec_get_withdraw_cap(sender_addr));
        ensures global<DiemAccount>(sender_addr) == update_field(old(global<DiemAccount>(sender_addr)),
            withdraw_capability, Option::spec_none());
        ensures result.account_address == sender_addr;
    }

    spec schema ExtractWithdrawCapAbortsIf {
        sender_addr: address;
        aborts_if !exists_at(sender_addr) with Errors::NOT_PUBLISHED;
        aborts_if spec_holds_delegated_withdraw_capability(sender_addr) with Errors::INVALID_STATE;
    }

    /// Return the withdraw capability to the account it originally came from
    public fun restore_withdraw_capability(cap: WithdrawCapability)
    acquires DiemAccount {
        assert(exists_at(cap.account_address), Errors::not_published(EACCOUNT));
        // Abort if the withdraw capability for this account is not extracted,
        // indicating that the withdraw capability is not unique.
        assert(
            delegated_withdraw_capability(cap.account_address),
            Errors::invalid_state(EWITHDRAW_CAPABILITY_NOT_EXTRACTED)
        );
        let account = borrow_global_mut<DiemAccount>(cap.account_address);
        Option::fill(&mut account.withdraw_capability, cap)
    }

    spec fun restore_withdraw_capability {
        pragma opaque;
        let cap_addr = cap.account_address;
        modifies global<DiemAccount>(cap_addr);
        aborts_if !exists_at(cap_addr) with Errors::NOT_PUBLISHED;
        aborts_if !delegated_withdraw_capability(cap_addr) with Errors::INVALID_STATE;
        ensures spec_holds_own_withdraw_cap(cap_addr);
    }

    /// Withdraw `amount` Diem<Token> from the address embedded in `WithdrawCapability` and
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
    ) acquires DiemAccount, Balance, AccountOperationsCapability {
        deposit<Token>(
            *&cap.account_address,
            payee,
            withdraw_from(cap, payee, amount, copy metadata),
            metadata,
            metadata_signature
        );
    }

    spec fun pay_from {
        pragma opaque;
        let payer = cap.account_address;
        modifies global<DiemAccount>(payer);
        modifies global<DiemAccount>(payee);
        modifies global<Balance<Token>>(payer);
        modifies global<Balance<Token>>(payee);
        ensures exists_at(payer);
        ensures exists_at(payee);
        ensures exists<Balance<Token>>(payer);
        ensures exists<Balance<Token>>(payee);
        ensures global<DiemAccount>(payer).withdraw_capability ==
            old(global<DiemAccount>(payer).withdraw_capability);
        include PayFromAbortsIf<Token>;
        include PayFromEnsures<Token>{payer};

    }
    spec schema PayFromAbortsIf<Token> {
        cap: WithdrawCapability;
        payee: address;
        amount: u64;
        metadata: vector<u8>;
        metadata_signature: vector<u8> ;
        include DepositAbortsIf<Token>{payer: cap.account_address};
        include cap.account_address != payee ==> DepositOverflowAbortsIf<Token>;
        include WithdrawFromAbortsIf<Token>;
    }
    spec schema PayFromAbortsIfRestricted<Token> {
        cap: WithdrawCapability;
        payee: address;
        amount: u64;
        metadata: vector<u8>;
        metadata_signature: vector<u8> ;
        let payer = cap.account_address;
        include DepositAbortsIfRestricted<Token>{payer: cap.account_address};
        include WithdrawFromBalanceNoLimitsAbortsIf<Token>{payer, balance: global<Balance<Token>>(payer)};
        aborts_if !exists<Balance<Token>>(payer) with Errors::NOT_PUBLISHED;
    }
    spec schema PayFromEnsures<Token> {
        payer: address;
        payee: address;
        amount: u64;
        ensures payer == payee ==> balance<Token>(payer) == old(balance<Token>(payer));
        ensures payer != payee ==> balance<Token>(payer) == old(balance<Token>(payer)) - amount;
        ensures payer != payee ==> balance<Token>(payee) == old(balance<Token>(payee)) + amount;
    }

    /// Rotate the authentication key for the account under cap.account_address
    public fun rotate_authentication_key(
        cap: &KeyRotationCapability,
        new_authentication_key: vector<u8>,
    ) acquires DiemAccount  {
        assert(exists_at(cap.account_address), Errors::not_published(EACCOUNT));
        let sender_account_resource = borrow_global_mut<DiemAccount>(cap.account_address);
        // Don't allow rotating to clearly invalid key
        assert(
            Vector::length(&new_authentication_key) == 32,
            Errors::invalid_argument(EMALFORMED_AUTHENTICATION_KEY)
        );
        sender_account_resource.authentication_key = new_authentication_key;
    }
    spec fun rotate_authentication_key {
        include RotateAuthenticationKeyAbortsIf;
        include RotateAuthenticationKeyEnsures{addr: cap.account_address};
        include RotateOnlyKeyOfCapAddress;
    }
    spec schema RotateAuthenticationKeyAbortsIf {
        cap: &KeyRotationCapability;
        new_authentication_key: vector<u8>;
        aborts_if !exists_at(cap.account_address) with Errors::NOT_PUBLISHED;
        aborts_if len(new_authentication_key) != 32 with Errors::INVALID_ARGUMENT;
    }
    spec schema RotateAuthenticationKeyEnsures {
        addr: address;
        new_authentication_key: vector<u8>;
        ensures global<DiemAccount>(addr).authentication_key == new_authentication_key;
    }

    /// # Access Control
    spec schema RotateOnlyKeyOfCapAddress {
        cap: KeyRotationCapability;
        /// Can only rotate the authentication_key of cap.account_address [[H17]][PERMISSION].
        ensures forall addr: address where addr != cap.account_address && old(exists_at(addr)):
            global<DiemAccount>(addr).authentication_key == old(global<DiemAccount>(addr).authentication_key);
    }

    /// Return a unique capability granting permission to rotate the sender's authentication key
    public fun extract_key_rotation_capability(account: &signer): KeyRotationCapability
    acquires DiemAccount {
        let account_address = Signer::address_of(account);
        // Abort if we already extracted the unique key rotation capability for this account.
        assert(
            !delegated_key_rotation_capability(account_address),
            Errors::invalid_state(EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED)
        );
        assert(exists_at(account_address), Errors::not_published(EACCOUNT));
        let account = borrow_global_mut<DiemAccount>(account_address);
        Option::extract(&mut account.key_rotation_capability)
    }
    spec fun extract_key_rotation_capability {
        include ExtractKeyRotationCapabilityAbortsIf;
        include ExtractKeyRotationCapabilityEnsures;
    }
    spec schema ExtractKeyRotationCapabilityAbortsIf {
        account: signer;
        let account_addr = Signer::spec_address_of(account);
        aborts_if !exists_at(account_addr) with Errors::NOT_PUBLISHED;
        include AbortsIfDelegatedKeyRotationCapability;
    }
    spec schema AbortsIfDelegatedKeyRotationCapability {
        account: signer;
        aborts_if delegated_key_rotation_capability(Signer::spec_address_of(account)) with Errors::INVALID_STATE;
    }
    spec schema ExtractKeyRotationCapabilityEnsures {
        account: signer;
        ensures delegated_key_rotation_capability(Signer::spec_address_of(account));
    }

    /// Return the key rotation capability to the account it originally came from
    public fun restore_key_rotation_capability(cap: KeyRotationCapability)
    acquires DiemAccount {
        assert(exists_at(cap.account_address), Errors::not_published(EACCOUNT));
        let account = borrow_global_mut<DiemAccount>(cap.account_address);
        Option::fill(&mut account.key_rotation_capability, cap)
    }
    spec fun restore_key_rotation_capability {
        include RestoreKeyRotationCapabilityAbortsIf;
        include RestoreKeyRotationCapabilityEnsures;
    }
    spec schema RestoreKeyRotationCapabilityAbortsIf {
        cap: KeyRotationCapability;
        aborts_if !exists_at(cap.account_address) with Errors::NOT_PUBLISHED;
        aborts_if !delegated_key_rotation_capability(cap.account_address) with Errors::INVALID_ARGUMENT;
    }
    spec schema RestoreKeyRotationCapabilityEnsures {
        cap: KeyRotationCapability;
        ensures spec_holds_own_key_rotation_cap(cap.account_address);
    }

    /// Add balances for `Token` to `new_account`.  If `add_all_currencies` is true,
    /// then add for both token types.
    fun add_currencies_for_account<Token>(
        new_account: &signer,
        add_all_currencies: bool,
    ) {
        let new_account_addr = Signer::address_of(new_account);
        add_currency<Token>(new_account);
        if (add_all_currencies) {
            if (!exists<Balance<XUS>>(new_account_addr)) {
                add_currency<XUS>(new_account);
            };
            if (!exists<Balance<XDX>>(new_account_addr)) {
                add_currency<XDX>(new_account);
            };
        };
    }

    spec fun add_currencies_for_account {
        let new_account_addr = Signer::spec_address_of(new_account);
        aborts_if !Roles::spec_can_hold_balance_addr(new_account_addr) with Errors::INVALID_ARGUMENT;
        include AddCurrencyForAccountAbortsIf<Token>{addr: new_account_addr};
        include AddCurrencyForAccountEnsures<Token>{addr: new_account_addr};
    }

    spec schema AddCurrencyForAccountAbortsIf<Token> {
        addr: address;
        add_all_currencies: bool;
        include Diem::AbortsIfNoCurrency<Token>;
        aborts_if exists<Balance<Token>>(addr) with Errors::ALREADY_PUBLISHED;
        include add_all_currencies && !exists<Balance<XUS>>(addr)
            ==> Diem::AbortsIfNoCurrency<XUS>;
        include add_all_currencies && !exists<Balance<XDX>>(addr)
            ==> Diem::AbortsIfNoCurrency<XDX>;
    }

    spec schema AddCurrencyForAccountEnsures<Token> {
        addr: address;
        add_all_currencies: bool;
        include AddCurrencyEnsures<Token>;
        include add_all_currencies && !exists<Balance<XUS>>(addr)
            ==> AddCurrencyEnsures<XUS>;
        include add_all_currencies && !exists<Balance<XDX>>(addr)
            ==> AddCurrencyEnsures<XDX>;
    }


    /// Creates a new account with account at `new_account_address` with
    /// authentication key `auth_key_prefix` | `fresh_address`.
    /// Aborts if there is already an account at `new_account_address`.
    ///
    /// Creating an account at address 0x0 will abort as it is a reserved address for the MoveVM.
    fun make_account(
        new_account: signer,
        auth_key_prefix: vector<u8>,
    ) acquires AccountOperationsCapability {
        let new_account_addr = Signer::address_of(&new_account);
        // cannot create an account at the reserved address 0x0
        assert(
            new_account_addr != CoreAddresses::VM_RESERVED_ADDRESS(),
            Errors::invalid_argument(ECANNOT_CREATE_AT_VM_RESERVED)
        );
        assert(
            new_account_addr != CoreAddresses::CORE_CODE_ADDRESS(),
            Errors::invalid_argument(ECANNOT_CREATE_AT_CORE_CODE)
        );

        // Construct authentication key.
        let authentication_key = create_authentication_key(&new_account, auth_key_prefix);

        // Publish AccountFreezing::FreezingBit (initially not frozen)
        AccountFreezing::create(&new_account);
        // The AccountOperationsCapability is published during Genesis, so it should
        // always exist.  This is a sanity check.
        assert(
            exists<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS()),
            Errors::not_published(EACCOUNT_OPERATIONS_CAPABILITY)
        );
        // Emit the CreateAccountEvent
        Event::emit_event(
            &mut borrow_global_mut<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS()).creation_events,
            CreateAccountEvent { created: new_account_addr, role_id: Roles::get_role_id(new_account_addr) },
        );
        // Publishing the account resource last makes it possible to prove invariants that simplify
        // aborts_if's, etc.
        move_to(
            &new_account,
            DiemAccount {
                authentication_key,
                withdraw_capability: Option::some(
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
        destroy_signer(new_account);
    }

    spec fun make_account {
        pragma opaque;
        let new_account_addr = Signer::address_of(new_account);
        modifies global<DiemAccount>(new_account_addr);
        modifies global<AccountFreezing::FreezingBit>(new_account_addr);
        modifies global<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS());
        ensures exists<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS());
        // Next requires is needed to prove invariant
        requires exists<Roles::RoleId>(new_account_addr);
        include MakeAccountAbortsIf{addr: new_account_addr};
        ensures exists_at(new_account_addr);
        ensures AccountFreezing::spec_account_is_not_frozen(new_account_addr);
        let account_ops_cap = global<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS());
        ensures account_ops_cap == update_field(old(account_ops_cap), creation_events, account_ops_cap.creation_events);
        ensures spec_holds_own_key_rotation_cap(new_account_addr);
        ensures spec_holds_own_withdraw_cap(new_account_addr);
    }
    spec schema MakeAccountAbortsIf {
        addr: address;
        auth_key_prefix: vector<u8>;
        aborts_if addr == CoreAddresses::VM_RESERVED_ADDRESS() with Errors::INVALID_ARGUMENT;
        aborts_if addr == CoreAddresses::CORE_CODE_ADDRESS() with Errors::INVALID_ARGUMENT;
        aborts_if exists<AccountFreezing::FreezingBit>(addr) with Errors::ALREADY_PUBLISHED;
        // There is an invariant below that says that there is always an AccountOperationsCapability
        // after Genesis, so this can only abort during Genesis.
        aborts_if DiemTimestamp::is_genesis()
            && !exists<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS())
            with Errors::NOT_PUBLISHED;
        include CreateAuthenticationKeyAbortsIf;
        // We do not need to specify aborts_if if account already exists, because make_account will
        // abort because of a published FreezingBit, first.
    }

    /// Construct an authentication key, aborting if the prefix is not valid.
    fun create_authentication_key(account: &signer, auth_key_prefix: vector<u8>): vector<u8> {
        let authentication_key = auth_key_prefix;
        Vector::append(
            &mut authentication_key, BCS::to_bytes(Signer::borrow_address(account))
        );
        assert(
            Vector::length(&authentication_key) == 32,
            Errors::invalid_argument(EMALFORMED_AUTHENTICATION_KEY)
        );
        authentication_key
    }
    spec fun create_authentication_key {
        /// The specification of this function is abstracted to avoid the complexity of
        /// vector concatenation of serialization results. The actual value of the key
        /// is assumed to be irrelevant for callers. Instead the uninterpreted function
        /// `spec_abstract_create_authentication_key` is used to represent the key value.
        /// The aborts behavior is, however, preserved: the caller must provide a
        /// key prefix of a specific length.
        pragma opaque;
        include [abstract] CreateAuthenticationKeyAbortsIf;
        ensures [abstract]
            result == spec_abstract_create_authentication_key(auth_key_prefix) &&
            len(result) == 32;
    }
    spec schema CreateAuthenticationKeyAbortsIf {
        auth_key_prefix: vector<u8>;
        aborts_if 16 + len(auth_key_prefix) != 32 with Errors::INVALID_ARGUMENT;
    }
    spec define spec_abstract_create_authentication_key(auth_key_prefix: vector<u8>): vector<u8>;


    /// Creates the diem root account (during genesis). Publishes the Diem root role,
    /// Publishes a SlidingNonce resource, sets up event generator, publishes
    /// AccountOperationsCapability, WriteSetManager, and finally makes the account.
    fun create_diem_root_account(
        auth_key_prefix: vector<u8>,
    ) acquires AccountOperationsCapability {
        DiemTimestamp::assert_genesis();
        let dr_account = create_signer(CoreAddresses::DIEM_ROOT_ADDRESS());
        CoreAddresses::assert_diem_root(&dr_account);
        Roles::grant_diem_root_role(&dr_account);
        SlidingNonce::publish(&dr_account);
        Event::publish_generator(&dr_account);

        assert(
            !exists<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS()),
            Errors::already_published(EACCOUNT_OPERATIONS_CAPABILITY)
        );
        move_to(
            &dr_account,
            AccountOperationsCapability {
                limits_cap: AccountLimits::grant_mutation_capability(&dr_account),
                creation_events: Event::new_event_handle<CreateAccountEvent>(&dr_account),
            }
        );
        assert(
            !exists<DiemWriteSetManager>(CoreAddresses::DIEM_ROOT_ADDRESS()),
            Errors::already_published(EWRITESET_MANAGER)
        );
        move_to(
            &dr_account,
            DiemWriteSetManager {
                upgrade_events: Event::new_event_handle<Self::AdminTransactionEvent>(&dr_account),
            }
        );
        make_account(dr_account, auth_key_prefix)
    }

    spec fun create_diem_root_account {
        pragma opaque;
        include CreateDiemRootAccountModifies;
        include CreateDiemRootAccountAbortsIf;
        include CreateDiemRootAccountEnsures;
    }

    spec schema CreateDiemRootAccountModifies {
        let dr_addr = CoreAddresses::DIEM_ROOT_ADDRESS();
        modifies global<DiemAccount>(dr_addr);
        modifies global<AccountOperationsCapability>(dr_addr);
        modifies global<DiemWriteSetManager>(dr_addr);
        modifies global<SlidingNonce::SlidingNonce>(dr_addr);
        modifies global<Roles::RoleId>(dr_addr);
        modifies global<AccountFreezing::FreezingBit>(dr_addr);
    }
    spec schema CreateDiemRootAccountAbortsIf {
        auth_key_prefix: vector<u8>;
        include DiemTimestamp::AbortsIfNotGenesis;
        include Roles::GrantRole{addr: CoreAddresses::DIEM_ROOT_ADDRESS(), role_id: Roles::DIEM_ROOT_ROLE_ID};
        aborts_if exists<SlidingNonce::SlidingNonce>(CoreAddresses::DIEM_ROOT_ADDRESS())
            with Errors::ALREADY_PUBLISHED;
        aborts_if exists<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS())
            with Errors::ALREADY_PUBLISHED;
        aborts_if exists<DiemWriteSetManager>(CoreAddresses::DIEM_ROOT_ADDRESS())
            with Errors::ALREADY_PUBLISHED;
        aborts_if exists<AccountFreezing::FreezingBit>(CoreAddresses::DIEM_ROOT_ADDRESS())
            with Errors::ALREADY_PUBLISHED;
        include CreateAuthenticationKeyAbortsIf;
    }
    spec schema CreateDiemRootAccountEnsures {
        let dr_addr = CoreAddresses::DIEM_ROOT_ADDRESS();
        ensures exists<AccountOperationsCapability>(dr_addr);
        ensures exists<DiemWriteSetManager>(dr_addr);
        ensures exists<SlidingNonce::SlidingNonce>(dr_addr);
        ensures Roles::spec_has_diem_root_role_addr(dr_addr);
        ensures exists_at(dr_addr);
        ensures AccountFreezing::spec_account_is_not_frozen(dr_addr);
        ensures spec_holds_own_key_rotation_cap(dr_addr);
        ensures spec_holds_own_withdraw_cap(dr_addr);
    }

    /// Create a treasury/compliance account at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`.  Can only be called during genesis.
    /// Also, publishes the treasury compliance role, the SlidingNonce resource, and
    /// event handle generator, then makes the account.
    fun create_treasury_compliance_account(
        dr_account: &signer,
        auth_key_prefix: vector<u8>,
    ) acquires AccountOperationsCapability {
        DiemTimestamp::assert_genesis();
        Roles::assert_diem_root(dr_account);
        let new_account_address = CoreAddresses::TREASURY_COMPLIANCE_ADDRESS();
        let new_account = create_signer(new_account_address);
        Roles::grant_treasury_compliance_role(&new_account, dr_account);
        SlidingNonce::publish(&new_account);
        Event::publish_generator(&new_account);
        make_account(new_account, auth_key_prefix)
    }

    spec fun create_treasury_compliance_account {
        pragma opaque;
        let tc_addr = CoreAddresses::TREASURY_COMPLIANCE_ADDRESS();
        include CreateTreasuryComplianceAccountModifies;
        include CreateTreasuryComplianceAccountAbortsIf;
        include Roles::AbortsIfNotDiemRoot{account: dr_account};
        include MakeAccountAbortsIf{addr: CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()};
        include CreateTreasuryComplianceAccountEnsures;

        let account_ops_cap = global<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS());
        ensures account_ops_cap == update_field(old(account_ops_cap), creation_events, account_ops_cap.creation_events);
    }

    spec schema CreateTreasuryComplianceAccountModifies {
        let tc_addr = CoreAddresses::TREASURY_COMPLIANCE_ADDRESS();
        modifies global<DiemAccount>(tc_addr);
        modifies global<SlidingNonce::SlidingNonce>(tc_addr);
        modifies global<Roles::RoleId>(tc_addr);
        modifies global<AccountFreezing::FreezingBit>(tc_addr);
        modifies global<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS());
        ensures exists<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS());
    }

    spec schema CreateTreasuryComplianceAccountAbortsIf {
        dr_account: signer;
        auth_key_prefix: vector<u8>;
        include DiemTimestamp::AbortsIfNotGenesis;
        include Roles::GrantRole{addr: CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(), role_id: Roles::TREASURY_COMPLIANCE_ROLE_ID};
        aborts_if exists<SlidingNonce::SlidingNonce>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS())
            with Errors::ALREADY_PUBLISHED;
    }

    spec schema CreateTreasuryComplianceAccountEnsures {
        let tc_addr = CoreAddresses::TREASURY_COMPLIANCE_ADDRESS();
        ensures Roles::spec_has_treasury_compliance_role_addr(tc_addr);
        ensures exists_at(tc_addr);
        ensures exists<SlidingNonce::SlidingNonce>(tc_addr);
        ensures AccountFreezing::spec_account_is_not_frozen(tc_addr);
        ensures spec_holds_own_key_rotation_cap(tc_addr);
        ensures spec_holds_own_withdraw_cap(tc_addr);
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
    ) acquires AccountOperationsCapability {
        let new_dd_account = create_signer(new_account_address);
        Event::publish_generator(&new_dd_account);
        Roles::new_designated_dealer_role(creator_account, &new_dd_account);
        DesignatedDealer::publish_designated_dealer_credential<CoinType>(&new_dd_account, creator_account, add_all_currencies);
        add_currencies_for_account<CoinType>(&new_dd_account, add_all_currencies);
        DualAttestation::publish_credential(&new_dd_account, creator_account, human_name);
        make_account(new_dd_account, auth_key_prefix)
    }

    spec fun create_designated_dealer {
        include CreateDesignatedDealerAbortsIf<CoinType>;
        include CreateDesignatedDealerEnsures<CoinType>;
    }

    spec schema CreateDesignatedDealerAbortsIf<CoinType> {
        creator_account: signer;
        new_account_address: address;
        auth_key_prefix: vector<u8>;
        add_all_currencies: bool;
        include DiemTimestamp::AbortsIfNotOperating;
        include Roles::AbortsIfNotTreasuryCompliance{account: creator_account};
        aborts_if exists<Roles::RoleId>(new_account_address) with Errors::ALREADY_PUBLISHED;
        aborts_if exists<DesignatedDealer::Dealer>(new_account_address) with Errors::ALREADY_PUBLISHED;
        include if (add_all_currencies) DesignatedDealer::AddCurrencyAbortsIf<XUS>{dd_addr: new_account_address}
                else DesignatedDealer::AddCurrencyAbortsIf<CoinType>{dd_addr: new_account_address};
        include AddCurrencyForAccountAbortsIf<CoinType>{addr: new_account_address};
        include MakeAccountAbortsIf{addr: new_account_address};
    }

    spec schema CreateDesignatedDealerEnsures<CoinType> {
        new_account_address: address;
        ensures exists<DesignatedDealer::Dealer>(new_account_address);
        ensures exists_at(new_account_address);
        ensures Roles::spec_has_designated_dealer_role_addr(new_account_address);
        include AddCurrencyForAccountEnsures<CoinType>{addr: new_account_address};
    }
    ///////////////////////////////////////////////////////////////////////////
    // VASP methods
    ///////////////////////////////////////////////////////////////////////////

    /// Create an account with the ParentVASP role at `new_account_address` with authentication key
    /// `auth_key_prefix` | `new_account_address`.  If `add_all_currencies` is true, 0 balances for
    /// all available currencies in the system will also be added.
    public fun create_parent_vasp_account<Token>(
        creator_account: &signer,  // TreasuryCompliance
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
        add_all_currencies: bool
    ) acquires AccountOperationsCapability {
        let new_account = create_signer(new_account_address);
        Roles::new_parent_vasp_role(creator_account, &new_account);
        VASP::publish_parent_vasp_credential(&new_account, creator_account);
        Event::publish_generator(&new_account);
        DualAttestation::publish_credential(&new_account, creator_account, human_name);
        add_currencies_for_account<Token>(&new_account, add_all_currencies);
        make_account(new_account, auth_key_prefix)
    }

    spec fun create_parent_vasp_account {
        include CreateParentVASPAccountAbortsIf<Token>;
        include CreateParentVASPAccountEnsures<Token>;
    }

    spec schema CreateParentVASPAccountAbortsIf<Token> {
        creator_account: signer;
        new_account_address: address;
        auth_key_prefix: vector<u8>;
        add_all_currencies: bool;
        include DiemTimestamp::AbortsIfNotOperating;
        include Roles::AbortsIfNotTreasuryCompliance{account: creator_account};
        aborts_if exists<Roles::RoleId>(new_account_address) with Errors::ALREADY_PUBLISHED;
        aborts_if VASP::is_vasp(new_account_address) with Errors::ALREADY_PUBLISHED;
        include AddCurrencyForAccountAbortsIf<Token>{addr: new_account_address};
        include MakeAccountAbortsIf{addr: new_account_address};
    }

    spec schema CreateParentVASPAccountEnsures<Token> {
        new_account_address: address;
        include VASP::PublishParentVASPEnsures{vasp_addr: new_account_address};
        ensures exists_at(new_account_address);
        ensures Roles::spec_has_parent_VASP_role_addr(new_account_address);
        include AddCurrencyForAccountEnsures<Token>{addr: new_account_address};
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
    ) acquires AccountOperationsCapability {
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
        include CreateChildVASPAccountAbortsIf<Token>;
        include CreateChildVASPAccountEnsures<Token>{
            parent_addr: Signer::spec_address_of(parent),
            child_addr: new_account_address,
        };
        include AddCurrencyForAccountEnsures<Token>{addr: new_account_address};
    }

    spec schema CreateChildVASPAccountAbortsIf<Token> {
        parent: signer;
        new_account_address: address;
        auth_key_prefix: vector<u8>;
        add_all_currencies: bool;
        include Roles::AbortsIfNotParentVasp{account: parent};
        aborts_if exists<Roles::RoleId>(new_account_address) with Errors::ALREADY_PUBLISHED;
        include VASP::PublishChildVASPAbortsIf{child_addr: new_account_address};
        include AddCurrencyForAccountAbortsIf<Token>{addr: new_account_address};
        include MakeAccountAbortsIf{addr: new_account_address};
    }

    spec schema CreateChildVASPAccountEnsures<Token> {
        parent_addr: address;
        child_addr: address;
        add_all_currencies: bool;
        include VASP::PublishChildVASPEnsures;
        ensures exists_at(child_addr);
        ensures Roles::spec_has_child_VASP_role_addr(child_addr);
    }

    ///////////////////////////////////////////////////////////////////////////
    // General purpose methods
    ///////////////////////////////////////////////////////////////////////////

    native fun create_signer(addr: address): signer;
    native fun destroy_signer(sig: signer);

    /// Helper to return the u64 value of the `balance` for `account`
    fun balance_for<Token>(balance: &Balance<Token>): u64 {
        Diem::value<Token>(&balance.coin)
    }

    /// Return the current balance of the account at `addr`.
    public fun balance<Token>(addr: address): u64 acquires Balance {
        assert(exists<Balance<Token>>(addr), Errors::not_published(EPAYER_DOESNT_HOLD_CURRENCY));
        balance_for(borrow_global<Balance<Token>>(addr))
    }
    spec fun balance {
        aborts_if !exists<Balance<Token>>(addr) with Errors::NOT_PUBLISHED;
    }

    /// Add a balance of `Token` type to the sending account
    public fun add_currency<Token>(account: &signer) {
        // aborts if `Token` is not a currency type in the system
        Diem::assert_is_currency<Token>();
        // Check that an account with this role is allowed to hold funds
        assert(
            Roles::can_hold_balance(account),
            Errors::invalid_argument(EROLE_CANT_STORE_BALANCE)
        );
        // aborts if this account already has a balance in `Token`
        let addr = Signer::address_of(account);
        assert(!exists<Balance<Token>>(addr), Errors::already_published(EADD_EXISTING_CURRENCY));

        move_to(account, Balance<Token>{ coin: Diem::zero<Token>() })
    }
    spec fun add_currency {
        include AddCurrencyAbortsIf<Token>;
        include AddCurrencyEnsures<Token>{addr: Signer::spec_address_of(account)};
    }
    spec schema AddCurrencyAbortsIf<Token> {
        account: signer;
        /// `Currency` must be valid
        include Diem::AbortsIfNoCurrency<Token>;
        /// `account` cannot have an existing balance in `Currency`
        aborts_if exists<Balance<Token>>(Signer::address_of(account)) with Errors::ALREADY_PUBLISHED;
        /// `account` must be allowed to hold balances.
        include AbortsIfAccountCantHoldBalance;
    }

    spec schema AddCurrencyEnsures<Token> {
        addr: address;
        /// This publishes a `Balance<Currency>` to the caller's account
        ensures exists<Balance<Token>>(addr);
        ensures global<Balance<Token>>(addr)
            == Balance<Token>{ coin: Diem<Token> { value: 0 } };
    }

    /// # Access Control
    spec schema AbortsIfAccountCantHoldBalance {
        account: signer;
        /// This function must abort if the predicate `can_hold_balance` for `account` returns false
        /// [[D1]][ROLE][[D2]][ROLE][[D3]][ROLE][[D4]][ROLE][[D5]][ROLE][[D6]][ROLE][[D7]][ROLE].
        aborts_if !Roles::can_hold_balance(account) with Errors::INVALID_ARGUMENT;
    }

    /// Return whether the account at `addr` accepts `Token` type coins
    public fun accepts_currency<Token>(addr: address): bool {
        exists<Balance<Token>>(addr)
    }

    /// Helper to return the sequence number field for given `account`
    fun sequence_number_for_account(account: &DiemAccount): u64 {
        account.sequence_number
    }

    /// Return the current sequence number at `addr`
    public fun sequence_number(addr: address): u64 acquires DiemAccount {
        assert(exists_at(addr), Errors::not_published(EACCOUNT));
        sequence_number_for_account(borrow_global<DiemAccount>(addr))
    }

    /// Return the authentication key for this account
    public fun authentication_key(addr: address): vector<u8> acquires DiemAccount {
        assert(exists_at(addr), Errors::not_published(EACCOUNT));
        *&borrow_global<DiemAccount>(addr).authentication_key
    }

    /// Return true if the account at `addr` has delegated its key rotation capability
    public fun delegated_key_rotation_capability(addr: address): bool
    acquires DiemAccount {
        assert(exists_at(addr), Errors::not_published(EACCOUNT));
        Option::is_none(&borrow_global<DiemAccount>(addr).key_rotation_capability)
    }

    /// Return true if the account at `addr` has delegated its withdraw capability
    public fun delegated_withdraw_capability(addr: address): bool
    acquires DiemAccount {
        assert(exists_at(addr), Errors::not_published(EACCOUNT));
        Option::is_none(&borrow_global<DiemAccount>(addr).withdraw_capability)
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
        exists<DiemAccount>(check_addr)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Prologues and Epilogues of user signed transactions
    ///////////////////////////////////////////////////////////////////////////

    /// The prologue for module transaction
    fun module_prologue<Token>(
        sender: &signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        txn_expiration_time: u64,
        chain_id: u8,
    ) acquires DiemAccount, Balance {
        assert(
            DiemTransactionPublishingOption::is_module_allowed(sender),
            Errors::invalid_state(PROLOGUE_EMODULE_NOT_ALLOWED),
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
    spec fun module_prologue {
        let transaction_sender = Signer::spec_address_of(sender);
        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        include ModulePrologueAbortsIf<Token> {
            max_transaction_fee,
            txn_expiration_time_seconds: txn_expiration_time,
        };
        ensures prologue_guarantees(sender);
    }
    spec schema ModulePrologueAbortsIf<Token> {
        sender: signer;
        txn_sequence_number: u64;
        txn_public_key: vector<u8>;
        chain_id: u8;
        max_transaction_fee: u128;
        txn_expiration_time_seconds: u64;
        let transaction_sender = Signer::spec_address_of(sender);
        include PrologueCommonAbortsIf<Token> {
            transaction_sender,
            txn_sequence_number,
            txn_public_key,
            chain_id,
            max_transaction_fee,
            txn_expiration_time_seconds,
        };
        /// Aborts only in genesis. Does not need to be handled.
        include DiemTransactionPublishingOption::AbortsIfNoTransactionPublishingOption;
        /// Covered: L75 (Match 9)
        aborts_if !DiemTransactionPublishingOption::spec_is_module_allowed(sender) with Errors::INVALID_STATE;
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
    ) acquires DiemAccount, Balance {
        assert(
            DiemTransactionPublishingOption::is_script_allowed(sender, &script_hash),
            Errors::invalid_state(PROLOGUE_ESCRIPT_NOT_ALLOWED),
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
    spec fun script_prologue {
        let transaction_sender = Signer::spec_address_of(sender);
        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        include ScriptPrologueAbortsIf<Token>{
            max_transaction_fee,
            txn_expiration_time_seconds: txn_expiration_time,
        };
        ensures prologue_guarantees(sender);
    }
    spec schema ScriptPrologueAbortsIf<Token> {
        sender: signer;
        txn_sequence_number: u64;
        txn_public_key: vector<u8>;
        chain_id: u8;
        max_transaction_fee: u128;
        txn_expiration_time_seconds: u64;
        script_hash: vector<u8>;
        let transaction_sender = Signer::spec_address_of(sender);
        include PrologueCommonAbortsIf<Token> {transaction_sender};
        /// Aborts only in Genesis. Does not need to be handled.
        include DiemTransactionPublishingOption::AbortsIfNoTransactionPublishingOption;
        /// Covered: L74 (Match 8)
        aborts_if !DiemTransactionPublishingOption::spec_is_script_allowed(sender, script_hash) with Errors::INVALID_STATE;
    }

    /// The prologue for WriteSet transaction
    fun writeset_prologue(
        sender: &signer,
        txn_sequence_number: u64,
        txn_public_key: vector<u8>,
        txn_expiration_time: u64,
        chain_id: u8,
    ) acquires DiemAccount, Balance {
        assert(
            Signer::address_of(sender) == CoreAddresses::DIEM_ROOT_ADDRESS(),
            Errors::invalid_argument(PROLOGUE_INVALID_WRITESET_SENDER)
        );
        assert(Roles::has_diem_root_role(sender), Errors::invalid_argument(PROLOGUE_INVALID_WRITESET_SENDER));

        // Currency code don't matter here as it won't be charged anyway. Gas constants are ommitted.
        prologue_common<XUS>(
            sender,
            txn_sequence_number,
            txn_public_key,
            0,
            0,
            txn_expiration_time,
            chain_id,
        )
    }

    spec fun writeset_prologue {
        include WritesetPrologueAbortsIf {txn_expiration_time_seconds: txn_expiration_time};
        ensures prologue_guarantees(sender);
        ensures Roles::has_diem_root_role(sender);
    }

    spec schema WritesetPrologueAbortsIf {
        sender: signer;
        txn_sequence_number: u64;
        txn_public_key: vector<u8>;
        txn_expiration_time_seconds: u64;
        chain_id: u8;
        let transaction_sender = Signer::spec_address_of(sender);
        /// Covered: L146 (Match 0)
        aborts_if transaction_sender != CoreAddresses::DIEM_ROOT_ADDRESS() with Errors::INVALID_ARGUMENT;
        /// Must abort if the signer does not have the DiemRoot role [[H9]][PERMISSION].
        /// Covered: L146 (Match 0)
        aborts_if !Roles::spec_has_diem_root_role_addr(transaction_sender) with Errors::INVALID_ARGUMENT;
        include PrologueCommonAbortsIf<XUS>{
            transaction_sender,
            max_transaction_fee: 0,
        };
    }

    /// The common prologue is invoked at the beginning of every transaction
    /// The main properties that it verifies:
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
    ) acquires DiemAccount, Balance {
        let transaction_sender = Signer::address_of(sender);

        // [PCA1]: Check that the chain ID stored on-chain matches the chain ID specified by the transaction
        assert(ChainId::get() == chain_id, Errors::invalid_argument(PROLOGUE_EBAD_CHAIN_ID));

        // [PCA2]: Verify that the transaction sender's account exists
        assert(exists_at(transaction_sender), Errors::invalid_argument(PROLOGUE_EACCOUNT_DNE));

        // [PCA3]: We check whether this account is frozen, if it is no transaction can be sent from it.
        assert(
            !AccountFreezing::account_is_frozen(transaction_sender),
            Errors::invalid_state(PROLOGUE_EACCOUNT_FROZEN)
        );

        // Load the transaction sender's account
        let sender_account = borrow_global<DiemAccount>(transaction_sender);

        // [PCA4]: Check that the hash of the transaction's public key matches the account's auth key
        assert(
            Hash::sha3_256(txn_public_key) == *&sender_account.authentication_key,
            Errors::invalid_argument(PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY),
        );

        // [PCA5]: Check that the account has enough balance for all of the gas
        assert(
            (txn_gas_price as u128) * (txn_max_gas_units as u128) <= MAX_U64,
            Errors::invalid_argument(PROLOGUE_ECANT_PAY_GAS_DEPOSIT),
        );

        let max_transaction_fee = txn_gas_price * txn_max_gas_units;

        // Don't grab the balance if the transaction fee is zero
        if (max_transaction_fee > 0) {
            // [PCA6]: Check that the account has a balance in this currency
            assert(
                exists<Balance<Token>>(transaction_sender),
                Errors::invalid_argument(PROLOGUE_ECANT_PAY_GAS_DEPOSIT)
            );
            let balance_amount = balance<Token>(transaction_sender);
            // [PCA7]: Check that the account can cover the maximum transaction fee
            assert(
                balance_amount >= max_transaction_fee,
                Errors::invalid_argument(PROLOGUE_ECANT_PAY_GAS_DEPOSIT)
            );
            // [PCA8]: Check that the gas fee can be paid in this currency
            assert(
                TransactionFee::is_coin_initialized<Token>(),
                Errors::invalid_argument(PROLOGUE_ECANT_PAY_GAS_DEPOSIT)
            );
        };

        // [PCA9]: Check that the transaction hasn't expired
        assert(
            DiemTimestamp::now_seconds() < txn_expiration_time_seconds,
            Errors::invalid_argument(PROLOGUE_ETRANSACTION_EXPIRED)
        );

        // [PCA10]: Check that the transaction sequence number is not too old (in the past)
        assert(
            txn_sequence_number >= sender_account.sequence_number,
            Errors::invalid_argument(PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD)
        );

        // [PCA11]: Check that the transaction's sequence number matches the
        // current sequence number. Otherwise sequence number is too new by [PCA8].
        assert(
            txn_sequence_number == sender_account.sequence_number,
            Errors::invalid_argument(PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW)
        );

        // [PCA12]: Check that the transaction's sequence number will not overflow.
        assert(
            (txn_sequence_number as u128) < MAX_U64,
            Errors::limit_exceeded(PROLOGUE_ESEQUENCE_NUMBER_TOO_BIG)
        );

        // WARNING: No checks should be added here as the sequence number too new check should be the last check run
        // by the prologue.
    }
    spec fun prologue_common {
        let transaction_sender = Signer::spec_address_of(sender);
        let max_transaction_fee = txn_gas_price * txn_max_gas_units;
        include PrologueCommonAbortsIf<Token> {
            transaction_sender,
            max_transaction_fee,
        };
    }
    spec schema PrologueCommonAbortsIf<Token> {
        transaction_sender: address;
        txn_sequence_number: u64;
        txn_public_key: vector<u8>;
        chain_id: u8;
        max_transaction_fee: u128;
        txn_expiration_time_seconds: u64;
        /// Only happens if this is called in Genesis. Doesn't need to be handled.
        include DiemTimestamp::AbortsIfNotOperating;
        /// [PCA1] Covered: L73 (Match 7)
        aborts_if chain_id != ChainId::spec_get_chain_id() with Errors::INVALID_ARGUMENT;
        /// [PCA2] Covered: L65 (Match 4)
        aborts_if !exists_at(transaction_sender) with Errors::INVALID_ARGUMENT;
        /// [PCA3] Covered: L57 (Match 0)
        aborts_if AccountFreezing::spec_account_is_frozen(transaction_sender) with Errors::INVALID_STATE;
        /// [PCA4] Covered: L59 (Match 1)
        aborts_if Hash::sha3_256(txn_public_key) != global<DiemAccount>(transaction_sender).authentication_key with Errors::INVALID_ARGUMENT;
        /// [PCA5] Covered: L69 (Match 5)
        aborts_if max_transaction_fee > MAX_U64 with Errors::INVALID_ARGUMENT;
        /// [PCA6] Covered: L69 (Match 5)
        aborts_if max_transaction_fee > 0 && !exists<Balance<Token>>(transaction_sender) with Errors::INVALID_ARGUMENT;
        /// [PCA7] Covered: L69 (Match 5)
        aborts_if max_transaction_fee > 0 && balance<Token>(transaction_sender) < max_transaction_fee with Errors::INVALID_ARGUMENT;
        /// [PCA8] Covered: L69 (Match 5)
        aborts_if max_transaction_fee > 0 && !TransactionFee::is_coin_initialized<Token>() with Errors::INVALID_ARGUMENT;
        /// [PCA9] Covered: L72 (Match 6)
        aborts_if DiemTimestamp::spec_now_seconds() >= txn_expiration_time_seconds with Errors::INVALID_ARGUMENT;
        /// [PCA10] Covered: L61 (Match 2)
        aborts_if txn_sequence_number < global<DiemAccount>(transaction_sender).sequence_number with Errors::INVALID_ARGUMENT;
        /// [PCA11] Covered: L63 (match 3)
        aborts_if txn_sequence_number > global<DiemAccount>(transaction_sender).sequence_number with Errors::INVALID_ARGUMENT;
        /// [PCA12] Covered: L81 (match 11)
        aborts_if txn_sequence_number >= MAX_U64 with Errors::LIMIT_EXCEEDED;
    }

    /// Collects gas and bumps the sequence number for executing a transaction.
    /// The epilogue is invoked at the end of the transaction.
    /// If the exection of the epilogue fails, it is re-invoked with different arguments, and
    /// based on the conditions checked in the prologue, should never fail.
    fun epilogue<Token>(
        account: &signer,
        txn_sequence_number: u64,
        txn_gas_price: u64,
        txn_max_gas_units: u64,
        gas_units_remaining: u64
    ) acquires DiemAccount, Balance {
        let sender = Signer::address_of(account);

        // [EA1; Invariant]: Make sure that the transaction's `max_gas_units` is greater
        // than the number of gas units remaining after execution.
        assert(txn_max_gas_units >= gas_units_remaining, Errors::invalid_argument(EGAS));
        let gas_used = txn_max_gas_units - gas_units_remaining;

        // [EA2; Invariant]: Make sure that the transaction fee would not overflow maximum
        // number representable in a u64. Already checked in [PCA5].
        assert(
            (txn_gas_price as u128) * (gas_used as u128) <= MAX_U64,
            Errors::limit_exceeded(EGAS)
        );
        let transaction_fee_amount = txn_gas_price * gas_used;

        // [EA3; Invariant]: Make sure that account exists, and load the
        // transaction sender's account. Already checked in [PCA2].
        assert(exists_at(sender), Errors::not_published(EACCOUNT));
        let sender_account = borrow_global_mut<DiemAccount>(sender);

        // [EA4; Condition]: Make sure account's sequence number is within the
        // representable range of u64. Bump the sequence number
        assert(
            sender_account.sequence_number < (MAX_U64 as u64),
            Errors::limit_exceeded(ESEQUENCE_NUMBER)
        );

        // [EA4; Invariant]: Make sure passed-in `txn_sequence_number` matches
        // the `sender_account`'s `sequence_number`. Already checked in [PCA9].
        assert(
            sender_account.sequence_number == txn_sequence_number,
            Errors::invalid_argument(ESEQUENCE_NUMBER)
        );

        // The transaction sequence number is passed in to prevent any
        // possibility of the account's sequence number increasing by more than
        // one for any transaction.
        sender_account.sequence_number = txn_sequence_number + 1;

        if (transaction_fee_amount > 0) {
            // [Invariant Use]: Balance for `Token` verified to exist for non-zero transaction fee amounts by [PCA6].
            let sender_balance = borrow_global_mut<Balance<Token>>(sender);
            let coin = &mut sender_balance.coin;

            // [EA4; Condition]: Abort if this withdrawal would make the `sender_account`'s balance go negative
            assert(
                transaction_fee_amount <= Diem::value(coin),
                Errors::limit_exceeded(PROLOGUE_ECANT_PAY_GAS_DEPOSIT)
            );

            // NB: `withdraw_from_balance` is not used as limits do not apply to this transaction fee
            TransactionFee::pay_fee(Diem::withdraw(coin, transaction_fee_amount))
        }
    }

    /// Epilogue for WriteSet trasnaction
    fun writeset_epilogue(
        dr_account: &signer,
        txn_sequence_number: u64,
        should_trigger_reconfiguration: bool,
    ) acquires DiemWriteSetManager, DiemAccount, Balance {
        let writeset_events_ref = borrow_global_mut<DiemWriteSetManager>(CoreAddresses::DIEM_ROOT_ADDRESS());
        Event::emit_event<AdminTransactionEvent>(
            &mut writeset_events_ref.upgrade_events,
            AdminTransactionEvent { committed_timestamp_secs: DiemTimestamp::now_seconds() },
        );

        // Double check that the sender is the DiemRoot account at the `CoreAddresses::DIEM_ROOT_ADDRESS`
        assert(
            Signer::address_of(dr_account) == CoreAddresses::DIEM_ROOT_ADDRESS(),
            Errors::invalid_argument(EPILOGUE_INVALID_WRITESET_SENDER)
        );
        assert(Roles::has_diem_root_role(dr_account), Errors::invalid_argument(EPILOGUE_INVALID_WRITESET_SENDER));

        // Currency code don't matter here as it won't be charged anyway.
        epilogue<XUS>(dr_account, txn_sequence_number, 0, 0, 0);
        if (should_trigger_reconfiguration) DiemConfig::reconfigure(dr_account)
    }

    /// Create a Validator account
    public fun create_validator_account(
        dr_account: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
    ) acquires AccountOperationsCapability {
        let new_account = create_signer(new_account_address);
        // The dr_account account is verified to have the diem root role in `Roles::new_validator_role`
        Roles::new_validator_role(dr_account, &new_account);
        Event::publish_generator(&new_account);
        ValidatorConfig::publish(&new_account, dr_account, human_name);
        make_account(new_account, auth_key_prefix)
    }

    spec fun create_validator_account {
        include CreateValidatorAccountAbortsIf;
        include CreateValidatorAccountEnsures;
    }

    spec schema CreateValidatorAccountAbortsIf {
        dr_account: signer;
        new_account_address: address;
        // from `Roles::new_validator_role`
        include Roles::AbortsIfNotDiemRoot{account: dr_account};
        include MakeAccountAbortsIf{addr: new_account_address};
        // from `ValidatorConfig::publish`
        include DiemTimestamp::AbortsIfNotOperating;
        aborts_if ValidatorConfig::exists_config(new_account_address) with Errors::ALREADY_PUBLISHED;
    }

    spec schema CreateValidatorAccountEnsures {
        new_account_address: address;
        // Note: `Roles::GrantRole` has both ensure's and aborts_if's.
        include Roles::GrantRole{addr: new_account_address, role_id: Roles::VALIDATOR_ROLE_ID};
        ensures exists_at(new_account_address);
        ensures ValidatorConfig::exists_config(new_account_address);
    }

    /// Create a Validator Operator account
    public fun create_validator_operator_account(
        dr_account: &signer,
        new_account_address: address,
        auth_key_prefix: vector<u8>,
        human_name: vector<u8>,
    ) acquires AccountOperationsCapability {
        let new_account = create_signer(new_account_address);
        // The dr_account is verified to have the diem root role in `Roles::new_validator_operator_role`
        Roles::new_validator_operator_role(dr_account, &new_account);
        Event::publish_generator(&new_account);
        ValidatorOperatorConfig::publish(&new_account, dr_account, human_name);
        make_account(new_account, auth_key_prefix)
    }

    spec fun create_validator_operator_account {
        include CreateValidatorOperatorAccountAbortsIf;
        include CreateValidatorOperatorAccountEnsures;
    }

    spec schema CreateValidatorOperatorAccountAbortsIf {
        dr_account: signer;
        new_account_address: address;
        // from `Roles::new_validator_operator_role`
        include Roles::AbortsIfNotDiemRoot{account: dr_account};
        include MakeAccountAbortsIf{addr: new_account_address};
        // from `ValidatorConfig::publish`
        include DiemTimestamp::AbortsIfNotOperating;
        aborts_if ValidatorOperatorConfig::has_validator_operator_config(new_account_address) with Errors::ALREADY_PUBLISHED;
    }

    spec schema CreateValidatorOperatorAccountEnsures {
        new_account_address: address;
        include Roles::GrantRole{addr: new_account_address, role_id: Roles::VALIDATOR_OPERATOR_ROLE_ID};
        ensures exists_at(new_account_address);
        ensures ValidatorOperatorConfig::has_validator_operator_config(new_account_address);
    }

    // ****************** Module Specifications *******************
    spec module {} // switch documentation context back to module level

    /// # Access Control

    /// ## Key Rotation Capability
    spec module {
        /// the permission "RotateAuthenticationKey(addr)" is granted to the account at addr [[H17]][PERMISSION].
        /// When an account is created, its KeyRotationCapability is granted to the account.
        apply EnsuresHasKeyRotationCap{account: new_account} to make_account;

        /// Only `make_account` creates KeyRotationCap [[H17]][PERMISSION][[I17]][PERMISSION]. `create_*_account` only calls
        /// `make_account`, and does not pack KeyRotationCap by itself.
        /// `restore_key_rotation_capability` restores KeyRotationCap, and does not create new one.
        apply PreserveKeyRotationCapAbsence to * except make_account, create_*_account,
              restore_key_rotation_capability, initialize;

        /// Every account holds either no key rotation capability (because KeyRotationCapability has been delegated)
        /// or the key rotation capability for addr itself [[H17]][PERMISSION].
        invariant [global] forall addr: address where exists_at(addr):
            delegated_key_rotation_capability(addr) || spec_holds_own_key_rotation_cap(addr);
    }

    spec schema EnsuresHasKeyRotationCap {
        account: signer;
        let addr = Signer::spec_address_of(account);
        ensures spec_holds_own_key_rotation_cap(addr);
    }
    spec schema PreserveKeyRotationCapAbsence {
        /// The absence of KeyRotationCap is preserved.
        ensures forall addr: address:
            old(!exists<DiemAccount>(addr) || !spec_has_key_rotation_cap(addr)) ==>
                (!exists<DiemAccount>(addr) || !spec_has_key_rotation_cap(addr));
    }

    /// ## Withdraw Capability
    spec module {
        /// the permission "WithdrawCapability(addr)" is granted to the account at addr [[H18]][PERMISSION].
        /// When an account is created, its WithdrawCapability is granted to the account.
        apply EnsuresWithdrawCap{account: new_account} to make_account;

        /// Only `make_account` creates WithdrawCap [[H18]][PERMISSION][[I18]][PERMISSION]. `create_*_account` only calls
        /// `make_account`, and does not pack KeyRotationCap by itself.
        /// `restore_withdraw_capability` restores WithdrawCap, and does not create new one.
        apply PreserveWithdrawCapAbsence to * except make_account, create_*_account,
                restore_withdraw_capability, initialize;

        /// Every account holds either no withdraw capability (because withdraw cap has been delegated)
        /// or the withdraw capability for addr itself [[H18]][PERMISSION].
        invariant [global] forall addr: address where exists_at(addr):
            spec_holds_delegated_withdraw_capability(addr) || spec_holds_own_withdraw_cap(addr);
    }

    spec schema EnsuresWithdrawCap {
        account: signer;
        let addr = Signer::spec_address_of(account);
        ensures spec_holds_own_withdraw_cap(addr);
    }
    spec schema PreserveWithdrawCapAbsence {
        /// The absence of WithdrawCap is preserved.
        ensures forall addr: address:
            old(!exists<DiemAccount>(addr) || Option::is_none(global<DiemAccount>(addr).withdraw_capability)) ==>
                (!exists<DiemAccount>(addr) || Option::is_none(global<DiemAccount>(addr).withdraw_capability));
    }

    /// ## Authentication Key

    spec module {
        /// only `Self::rotate_authentication_key` can rotate authentication_key [[H17]][PERMISSION].
        apply AuthenticationKeyRemainsSame to *, *<T> except rotate_authentication_key;
    }

    spec schema AuthenticationKeyRemainsSame {
        ensures forall addr: address where old(exists_at(addr)):
            global<DiemAccount>(addr).authentication_key == old(global<DiemAccount>(addr).authentication_key);
    }

    /// ## Balance

    spec module {
        /// only `Self::withdraw_from` and its helper and clients can withdraw [[H18]][PERMISSION].
        apply BalanceNotDecrease<Token> to *<Token>
            except withdraw_from, withdraw_from_balance, staple_xdx, unstaple_xdx,
                preburn, pay_from, epilogue, failure_epilogue, success_epilogue;
    }

    spec schema BalanceNotDecrease<Token> {
        ensures forall addr: address where old(exists<Balance<Token>>(addr)):
            global<Balance<Token>>(addr).coin.value >= old(global<Balance<Token>>(addr).coin.value);
    }

    /// # Persistence of Resources

    spec module {
        /// Accounts are never deleted.
        invariant update [global] forall addr: address where old(exists_at(addr)): exists_at(addr);

        /// After genesis, the `AccountOperationsCapability` exists.
        invariant [global]
            DiemTimestamp::is_operating() ==> exists<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS());

        /// After genesis, the `DiemWriteSetManager` exists.
        invariant [global]
            DiemTimestamp::is_operating() ==> exists<DiemWriteSetManager>(CoreAddresses::DIEM_ROOT_ADDRESS());

        /// resource struct `Balance<CoinType>` is persistent
        invariant update [global] forall coin_type: type, addr: address
            where old(exists<Balance<coin_type>>(addr)):
                exists<Balance<coin_type>>(addr);

        /// resource struct `AccountOperationsCapability` is persistent
        invariant update [global] old(exists<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS()))
                ==> exists<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS());

        /// resource struct `AccountOperationsCapability` is persistent
        invariant update [global]
            old(exists<DiemWriteSetManager>(CoreAddresses::DIEM_ROOT_ADDRESS()))
                ==> exists<DiemWriteSetManager>(CoreAddresses::DIEM_ROOT_ADDRESS());
    }

    /// # Other invariants
    spec module {

        /// Every address that has a published account has a published RoleId
        invariant [global] forall addr: address where exists_at(addr): exists<Roles::RoleId>(addr);

        /// If an account has a balance, the role of the account is compatible with having a balance.
        invariant [global] forall token: type: forall addr: address where exists<Balance<token>>(addr):
            Roles::spec_can_hold_balance_addr(addr);

        /// If there is a `DesignatedDealer::Dealer resource published at addr, the addr has a
        /// DesignatedDealer role.
        // Verified with additional target DesignatedDealer.move
        invariant [global] forall addr: address where exists<DesignatedDealer::Dealer>(addr):
            Roles::spec_has_designated_dealer_role_addr(addr);

        /// If there is a DualAttestation credential, account has designated dealer role
        // Verified with additional target "VASP.move"
        invariant [global] forall addr: address where exists<DualAttestation::Credential>(addr):
            Roles::spec_has_designated_dealer_role_addr(addr)
            || Roles::spec_has_parent_VASP_role_addr(addr);

        /// Every address that has a published account has a published FreezingBit
        invariant [global] forall addr: address where exists_at(addr): exists<AccountFreezing::FreezingBit>(addr);
    }

    /// # Helper Functions and Schemas

    /// ## Capabilities

    spec module {
        /// Returns field `key_rotation_capability` of the DiemAccount under `addr`.
        define spec_get_key_rotation_cap_field(addr: address): Option<KeyRotationCapability> {
            global<DiemAccount>(addr).key_rotation_capability
        }

        /// Returns the KeyRotationCapability of the field `key_rotation_capability`.
        define spec_get_key_rotation_cap(addr: address): KeyRotationCapability {
            Option::borrow(spec_get_key_rotation_cap_field(addr))
        }

        // Returns if the account holds KeyRotationCapability.
        define spec_has_key_rotation_cap(addr: address): bool {
            Option::is_some(spec_get_key_rotation_cap_field(addr))
        }

        /// Returns true if the DiemAccount at `addr` holds
        /// `KeyRotationCapability` for itself.
        define spec_holds_own_key_rotation_cap(addr: address): bool {
            spec_has_key_rotation_cap(addr)
            && addr == spec_get_key_rotation_cap(addr).account_address
        }

        /// Returns true if `AccountOperationsCapability` is published.
        define spec_has_account_operations_cap(): bool {
            exists<AccountOperationsCapability>(CoreAddresses::DIEM_ROOT_ADDRESS())
        }

        /// Returns field `withdraw_capability` of DiemAccount under `addr`.
        define spec_get_withdraw_cap_field(addr: address): Option<WithdrawCapability> {
            global<DiemAccount>(addr).withdraw_capability
        }

        /// Returns the WithdrawCapability of the field `withdraw_capability`.
        define spec_get_withdraw_cap(addr: address): WithdrawCapability {
            Option::borrow(spec_get_withdraw_cap_field(addr))
        }

        /// Returns true if the DiemAccount at `addr` holds a `WithdrawCapability`.
        define spec_has_withdraw_cap(addr: address): bool {
            Option::is_some(spec_get_withdraw_cap_field(addr))
        }

        /// Returns true if the DiemAccount at `addr` holds `WithdrawCapability` for itself.
        define spec_holds_own_withdraw_cap(addr: address): bool {
            spec_has_withdraw_cap(addr)
            && addr == spec_get_withdraw_cap(addr).account_address
        }

        /// Returns true of the account holds a delegated withdraw capability.
        define spec_holds_delegated_withdraw_capability(addr: address): bool {
            exists_at(addr) && Option::is_none(global<DiemAccount>(addr).withdraw_capability)
        }

    }

    /// ## Prologue

    spec define prologue_guarantees(sender: signer) : bool {
        let addr = Signer::spec_address_of(sender);
        DiemTimestamp::is_operating() && exists_at(addr) && !AccountFreezing::account_is_frozen(addr)
    }

    /// Used in transaction script to specify properties checked by the prologue.
    spec schema TransactionChecks {
        sender: signer;
        requires prologue_guarantees(sender);
    }
}
}
