address 0x1 {

/// The `Diem` module describes the concept of a coin in the Diem framework. It introduces the
/// resource `Diem::Diem<CoinType>`, representing a coin of given coin type.
/// The module defines functions operating on coins as well as functionality like
/// minting and burning of coins.
module Diem {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::Event::{Self, EventHandle};
    use 0x1::FixedPoint32::{Self, FixedPoint32};
    use 0x1::RegisteredCurrencies;
    use 0x1::Signer;
    use 0x1::Roles;
    use 0x1::DiemTimestamp;

    /// The `Diem` resource defines the Diem coin for each currency in
    /// Diem. Each "coin" is coupled with a type `CoinType` specifying the
    /// currency of the coin, and a `value` field specifying the value
    /// of the coin (in the base units of the currency `CoinType`
    /// and specified in the `CurrencyInfo` resource for that `CoinType`
    /// published under the `CoreAddresses::CURRENCY_INFO_ADDRESS()` account address).
    resource struct Diem<CoinType> {
        /// The value of this coin in the base units for `CoinType`
        value: u64
    }

    /// The `MintCapability` resource defines a capability to allow minting
    /// of coins of `CoinType` currency by the holder of this capability.
    /// This capability is held only either by the `CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()`
    /// account or the `0x1::XDX` module (and `CoreAddresses::DIEM_ROOT_ADDRESS()` in testnet).
    resource struct MintCapability<CoinType> { }

    /// The `BurnCapability` resource defines a capability to allow coins
    /// of `CoinType` currency to be burned by the holder of it.
    resource struct BurnCapability<CoinType> { }

    /// A `MintEvent` is emitted every time a Diem coin is minted. This
    /// contains the `amount` minted (in base units of the currency being
    /// minted) along with the `currency_code` for the coin(s) being
    /// minted, and that is defined in the `currency_code` field of the
    /// `CurrencyInfo` resource for the currency.
    struct MintEvent {
        /// Funds added to the system
        amount: u64,
        /// ASCII encoded symbol for the coin type (e.g., "XDX")
        currency_code: vector<u8>,
    }

    /// A `BurnEvent` is emitted every time a non-synthetic Diem coin
    /// (i.e., a Diem coin with false `is_synthetic` field) is
    /// burned. It contains the `amount` burned in base units for the
    /// currency, along with the `currency_code` for the coins being burned
    /// (and as defined in the `CurrencyInfo` resource for that currency).
    /// It also contains the `preburn_address` from which the coin is
    /// extracted for burning.
    struct BurnEvent {
        /// Funds removed from the system
        amount: u64,
        /// ASCII encoded symbol for the coin type (e.g., "XDX")
        currency_code: vector<u8>,
        /// Address with the `Preburn` resource that stored the now-burned funds
        preburn_address: address,
    }

    /// A `PreburnEvent` is emitted every time an `amount` of funds with
    /// a coin type `currency_code` are moved to a `Preburn` resource under
    /// the account at the address `preburn_address`.
    struct PreburnEvent {
        /// The amount of funds waiting to be removed (burned) from the system
        amount: u64,
        /// ASCII encoded symbol for the coin type (e.g., "XDX")
        currency_code: vector<u8>,
        /// Address with the `Preburn` resource that now holds the funds
        preburn_address: address,
    }

    /// A `CancelBurnEvent` is emitted every time funds of `amount` in a `Preburn`
    /// resource at `preburn_address` is canceled (removed from the
    /// preburn, but not burned). The currency of the funds is given by the
    /// `currency_code` as defined in the `CurrencyInfo` for that currency.
    struct CancelBurnEvent {
        /// The amount of funds returned
        amount: u64,
        /// ASCII encoded symbol for the coin type (e.g., "XDX")
        currency_code: vector<u8>,
        /// Address of the `Preburn` resource that held the now-returned funds.
        preburn_address: address,
    }

    /// An `ToXDXExchangeRateUpdateEvent` is emitted every time the to-XDX exchange
    /// rate for the currency given by `currency_code` is updated.
    struct ToXDXExchangeRateUpdateEvent {
        /// The currency code of the currency whose exchange rate was updated.
        currency_code: vector<u8>,
        /// The new on-chain to-XDX exchange rate between the
        /// `currency_code` currency and XDX. Represented in conversion
        /// between the (on-chain) base-units for the currency and microdiem.
        new_to_xdx_exchange_rate: u64,
    }

    /// The `CurrencyInfo<CoinType>` resource stores the various
    /// pieces of information needed for a currency (`CoinType`) that is
    /// registered on-chain. This resource _must_ be published under the
    /// address given by `CoreAddresses::CURRENCY_INFO_ADDRESS()` in order for the registration of
    /// `CoinType` as a recognized currency on-chain to be successful. At
    /// the time of registration, the `MintCapability<CoinType>` and
    /// `BurnCapability<CoinType>` capabilities are returned to the caller.
    /// Unless they are specified otherwise the fields in this resource are immutable.
    resource struct CurrencyInfo<CoinType> {
        /// The total value for the currency represented by `CoinType`. Mutable.
        total_value: u128,
        /// Value of funds that are in the process of being burned.  Mutable.
        preburn_value: u64,
        /// The (rough) exchange rate from `CoinType` to `XDX`. Mutable.
        to_xdx_exchange_rate: FixedPoint32,
        /// Holds whether or not this currency is synthetic (contributes to the
        /// off-chain reserve) or not. An example of such a synthetic
        ///currency would be the XDX.
        is_synthetic: bool,
        /// The scaling factor for the coin (i.e. the amount to divide by
        /// to get to the human-readable representation for this currency).
        /// e.g. 10^6 for `XUS`
        scaling_factor: u64,
        /// The smallest fractional part (number of decimal places) to be
        /// used in the human-readable representation for the currency (e.g.
        /// 10^2 for `XUS` cents)
        fractional_part: u64,
        /// The code symbol for this `CoinType`. ASCII encoded.
        /// e.g. for "XDX" this is x"4C4252". No character limit.
        currency_code: vector<u8>,
        /// Minting of new currency of CoinType is allowed only if this field is true.
        /// We may want to disable the ability to mint further coins of a
        /// currency while that currency is still around. This allows us to
        /// keep the currency in circulation while disallowing further
        /// creation of coins in the `CoinType` currency. Mutable.
        can_mint: bool,
        /// Event stream for minting and where `MintEvent`s will be emitted.
        mint_events: EventHandle<MintEvent>,
        /// Event stream for burning, and where `BurnEvent`s will be emitted.
        burn_events: EventHandle<BurnEvent>,
        /// Event stream for preburn requests, and where all
        /// `PreburnEvent`s for this `CoinType` will be emitted.
        preburn_events: EventHandle<PreburnEvent>,
        /// Event stream for all cancelled preburn requests for this
        /// `CoinType`.
        cancel_burn_events: EventHandle<CancelBurnEvent>,
        /// Event stream for emiting exchange rate change events
        exchange_rate_update_events: EventHandle<ToXDXExchangeRateUpdateEvent>,
    }

    /// The maximum value for `CurrencyInfo.scaling_factor`
    const MAX_SCALING_FACTOR: u64 = 10000000000;

    /// Data structure invariant for CurrencyInfo.  Asserts that `CurrencyInfo.scaling_factor`
    /// is always greater than 0 and not greater than `MAX_SCALING_FACTOR`
    spec struct CurrencyInfo {
        invariant 0 < scaling_factor && scaling_factor <= MAX_SCALING_FACTOR;
    }

    /// A holding area where funds that will subsequently be burned wait while their underlying
    /// assets are moved off-chain.
    /// This resource can only be created by the holder of a `BurnCapability`. An account that
    /// contains this address has the authority to initiate a burn request. A burn request can be
    /// resolved by the holder of a `BurnCapability` by either (1) burning the funds, or (2)
    /// returning the funds to the account that initiated the burn request.
    /// Concurrent preburn requests are not allowed, only one request (in `to_burn`) can be handled at any time.
    resource struct Preburn<CoinType> {
        /// A single pending burn amount.
        /// There is no pending burn request if the value in `to_burn` is 0
        to_burn: Diem<CoinType>,
    }

    /// Maximum u64 value.
    const MAX_U64: u64 = 18446744073709551615;
    /// Maximum u128 value.
    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    /// A `BurnCapability` resource is in an unexpected state.
    const EBURN_CAPABILITY: u64 = 0;
    /// A property expected of a `CurrencyInfo` resource didn't hold
    const ECURRENCY_INFO: u64 = 1;
    /// A property expected of a `Preburn` resource didn't hold
    const EPREBURN: u64 = 2;
    /// The preburn slot is already occupied with coins to be burned.
    const EPREBURN_OCCUPIED: u64 = 3;
    /// A burn was attempted on `Preburn` resource that cointained no coins
    const EPREBURN_EMPTY: u64 = 4;
    /// Minting is not allowed for the specified currency
    const EMINTING_NOT_ALLOWED: u64 = 5;
    /// The currency specified is a synthetic (non-fiat) currency
    const EIS_SYNTHETIC_CURRENCY: u64 = 6;
    /// A property expected of the coin provided didn't hold
    const ECOIN: u64 = 7;
    /// The destruction of a non-zero coin was attempted. Non-zero coins must be burned.
    const EDESTRUCTION_OF_NONZERO_COIN: u64 = 8;
    /// A property expected of `MintCapability` didn't hold
    const EMINT_CAPABILITY: u64 = 9;
    /// A withdrawal greater than the value of the coin was attempted.
    const EAMOUNT_EXCEEDS_COIN_VALUE: u64 = 10;

    /// Initialization of the `Diem` module. Initializes the set of
    /// registered currencies in the `0x1::RegisteredCurrencies` on-chain
    /// config, and publishes the `CurrencyRegistrationCapability` under the
    /// `CoreAddresses::DIEM_ROOT_ADDRESS()`. This can only be called from genesis.
    public fun initialize(
        dr_account: &signer,
    ) {
        DiemTimestamp::assert_genesis();
        // Operational constraint
        CoreAddresses::assert_diem_root(dr_account);
        RegisteredCurrencies::initialize(dr_account);
    }
    spec fun initialize {
        include DiemTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotDiemRoot{account: dr_account};
        include RegisteredCurrencies::InitializeAbortsIf;
        include RegisteredCurrencies::InitializeEnsures;
    }

    /// Publishes the `BurnCapability` `cap` for the `CoinType` currency under `account`. `CoinType`
    /// must be a registered currency type. The caller must pass a treasury compliance account.
    public fun publish_burn_capability<CoinType>(
        tc_account: &signer,
        cap: BurnCapability<CoinType>,
    ) {
        Roles::assert_treasury_compliance(tc_account);
        assert_is_currency<CoinType>();
        assert(
            !exists<BurnCapability<CoinType>>(Signer::address_of(tc_account)),
            Errors::already_published(EBURN_CAPABILITY)
        );
        move_to(tc_account, cap)
    }
    spec fun publish_burn_capability {
        aborts_if !spec_is_currency<CoinType>();
        include PublishBurnCapAbortsIfs<CoinType>;
    }
    spec schema PublishBurnCapAbortsIfs<CoinType> {
        tc_account: &signer;
        /// Must abort if tc_account does not have the TreasuryCompliance role.
        /// Only a TreasuryCompliance account can have the BurnCapability [[H3]][PERMISSION].
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        aborts_if exists<BurnCapability<CoinType>>(Signer::spec_address_of(tc_account)) with Errors::ALREADY_PUBLISHED;
    }
    spec schema PublishBurnCapEnsures<CoinType> {
        tc_account: &signer;
        ensures exists<BurnCapability<CoinType>>(Signer::spec_address_of(tc_account));
    }

    /// Mints `amount` of currency. The `account` must hold a
    /// `MintCapability<CoinType>` at the top-level in order for this call
    /// to be successful.
    public fun mint<CoinType>(account: &signer, value: u64): Diem<CoinType>
    acquires CurrencyInfo, MintCapability {
        let addr = Signer::address_of(account);
        assert(exists<MintCapability<CoinType>>(addr), Errors::requires_capability(EMINT_CAPABILITY));
        mint_with_capability(
            value,
            borrow_global<MintCapability<CoinType>>(addr)
        )
    }
    spec fun mint {
        modifies global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        ensures exists<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        /// Must abort if the account does not have the MintCapability [[H1]][PERMISSION].
        aborts_if !exists<MintCapability<CoinType>>(Signer::spec_address_of(account)) with Errors::REQUIRES_CAPABILITY;

        include MintAbortsIf<CoinType>;
        include MintEnsures<CoinType>;
    }

    /// Burns the coins currently held in the `Preburn` resource held under `preburn_address`.
    /// Calls to this functions will fail if the `account` does not have a
    /// published `BurnCapability` for the `CoinType` published under it.
    public fun burn<CoinType>(
        account: &signer,
        preburn_address: address
    ) acquires BurnCapability, CurrencyInfo, Preburn {
        let addr = Signer::address_of(account);
        assert(exists<BurnCapability<CoinType>>(addr), Errors::requires_capability(EBURN_CAPABILITY));
        burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<CoinType>>(addr)
        )
    }
    spec fun burn {
        include BurnAbortsIf<CoinType>;
        include BurnEnsures<CoinType>;
    }
    spec schema BurnAbortsIf<CoinType> {
        account: signer;
        preburn_address: address;

        /// Must abort if the account does not have the BurnCapability [[H3]][PERMISSION].
        aborts_if !exists<BurnCapability<CoinType>>(Signer::spec_address_of(account)) with Errors::REQUIRES_CAPABILITY;

        include AbortsIfNoPreburn<CoinType>;
        include BurnWithResourceCapAbortsIf<CoinType>{preburn: global<Preburn<CoinType>>(preburn_address)};
    }
    spec schema BurnEnsures<CoinType> {
        account: signer;
        preburn_address: address;

        include BurnWithResourceCapEnsures<CoinType>{preburn: global<Preburn<CoinType>>(preburn_address)};
    }
    spec schema AbortsIfNoPreburn<CoinType> {
        preburn_address: address;
        aborts_if !exists<Preburn<CoinType>>(preburn_address) with Errors::NOT_PUBLISHED;
    }

    /// Cancels the current burn request in the `Preburn` resource held
    /// under the `preburn_address`, and returns the coins.
    /// Calls to this will fail if the sender does not have a published
    /// `BurnCapability<CoinType>`, or if there is no preburn request
    /// outstanding in the `Preburn` resource under `preburn_address`.
    public fun cancel_burn<CoinType>(
        account: &signer,
        preburn_address: address
    ): Diem<CoinType> acquires BurnCapability, CurrencyInfo, Preburn {
        let addr = Signer::address_of(account);
        assert(exists<BurnCapability<CoinType>>(addr), Errors::requires_capability(EBURN_CAPABILITY));
        cancel_burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<CoinType>>(addr)
        )
    }
    spec fun cancel_burn {
        /// Must abort if the account does not have the BurnCapability [[H3]][PERMISSION].
        aborts_if !exists<BurnCapability<CoinType>>(Signer::spec_address_of(account)) with Errors::REQUIRES_CAPABILITY;
        include CancelBurnWithCapAbortsIf<CoinType>;
        include CancelBurnWithCapEnsures<CoinType>;
    }

    /// Mint a new `Diem` coin of `CoinType` currency worth `value`. The
    /// caller must have a reference to a `MintCapability<CoinType>`. Only
    /// the treasury compliance account or the `0x1::XDX` module can acquire such a
    /// reference.
    public fun mint_with_capability<CoinType>(
        value: u64,
        _capability: &MintCapability<CoinType>
    ): Diem<CoinType> acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        let currency_code = currency_code<CoinType>();
        // update market cap resource to reflect minting
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        assert(info.can_mint, Errors::invalid_state(EMINTING_NOT_ALLOWED));
        assert(MAX_U128 - info.total_value >= (value as u128), Errors::limit_exceeded(ECURRENCY_INFO));
        info.total_value = info.total_value + (value as u128);
        // don't emit mint events for synthetic currenices as this does not
        // change the total value of fiat currencies held on-chain.
        if (!info.is_synthetic) {
            Event::emit_event(
                &mut info.mint_events,
                MintEvent{
                    amount: value,
                    currency_code,
                }
            );
        };

        Diem<CoinType> { value }
    }
    spec fun mint_with_capability {
        pragma opaque;
        modifies global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        ensures exists<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        let currency_info = global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        include MintAbortsIf<CoinType>;
        include MintEnsures<CoinType>;
    }
    spec schema MintAbortsIf<CoinType> {
        value: u64;
        include AbortsIfNoCurrency<CoinType>;
        aborts_if !spec_currency_info<CoinType>().can_mint with Errors::INVALID_STATE;
        aborts_if spec_currency_info<CoinType>().total_value + value > max_u128() with Errors::LIMIT_EXCEEDED;
    }
    spec schema MintEnsures<CoinType> {
        value: u64;
        result: Diem<CoinType>;
        let currency_info = global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        ensures exists<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        ensures currency_info
            == update_field(old(currency_info), total_value, old(currency_info.total_value) + value);
        ensures result.value == value;
    }

    /// Add the `coin` to the `preburn.to_burn` field in the `Preburn` resource
    /// held at the address `preburn_address` if it is empty, otherwise raise
    /// a PendingPreburn Error (code 6). Emits a `PreburnEvent` to
    /// the `preburn_events` event stream in the `CurrencyInfo` for the
    /// `CoinType` passed in. However, if the currency being preburned has true
    /// `is_synthetic` then no `PreburnEvent` event will be emitted.
    public fun preburn_with_resource<CoinType>(
        coin: Diem<CoinType>,
        preburn: &mut Preburn<CoinType>,
        preburn_address: address,
    ) acquires CurrencyInfo {
        let coin_value = value(&coin);
        // Throw if already occupied
        assert(value(&preburn.to_burn) == 0, Errors::invalid_state(EPREBURN_OCCUPIED));
        deposit(&mut preburn.to_burn, coin);
        let currency_code = currency_code<CoinType>();
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        assert(MAX_U64 - info.preburn_value >= coin_value, Errors::limit_exceeded(ECOIN));
        info.preburn_value = info.preburn_value + coin_value;
        // don't emit preburn events for synthetic currenices as this does not
        // change the total value of fiat currencies held on-chain, and
        // therefore no off-chain movement of the backing coins needs to be
        // performed.
        if (!info.is_synthetic) {
            Event::emit_event(
                &mut info.preburn_events,
                PreburnEvent{
                    amount: coin_value,
                    currency_code,
                    preburn_address,
                }
            );
        };
    }
    spec fun preburn_with_resource {
        include PreburnWithResourceAbortsIf<CoinType>{amount: coin.value};
        include PreburnEnsures<CoinType>{amount: coin.value};
    }
    spec schema PreburnWithResourceAbortsIf<CoinType> {
        amount: u64;
        preburn: Preburn<CoinType>;
        aborts_if preburn.to_burn.value != 0 with Errors::INVALID_STATE;
        include PreburnAbortsIf<CoinType>;
    }
    spec schema PreburnAbortsIf<CoinType> {
        amount: u64;
        include AbortsIfNoCurrency<CoinType>;
        aborts_if spec_currency_info<CoinType>().preburn_value + amount > MAX_U64 with Errors::LIMIT_EXCEEDED;
    }
    spec schema PreburnEnsures<CoinType> {
        amount: u64;
        preburn: Preburn<CoinType>;
        ensures spec_currency_info<CoinType>().preburn_value
                    == old(spec_currency_info<CoinType>().preburn_value) + amount;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Treasury Compliance specific methods for DDs
    ///////////////////////////////////////////////////////////////////////////

    /// Create a `Preburn<CoinType>` resource
    public fun create_preburn<CoinType>(
        tc_account: &signer
    ): Preburn<CoinType> {
        Roles::assert_treasury_compliance(tc_account);
        assert_is_currency<CoinType>();
        Preburn<CoinType> { to_burn: zero<CoinType>() }
    }
    spec fun create_preburn {
        include CreatePreburnAbortsIf<CoinType>;
    }
    spec schema CreatePreburnAbortsIf<CoinType> {
        tc_account: signer;
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include AbortsIfNoCurrency<CoinType>;
    }

    /// Publishes a `Preburn` resource under `account`. This function is
    /// used for bootstrapping the designated dealer at account-creation
    /// time, and the association TC account `creator` (at `CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()`) is creating
    /// this resource for the designated dealer.
    public fun publish_preburn_to_account<CoinType>(
        account: &signer,
        tc_account: &signer
    ) acquires CurrencyInfo {
        Roles::assert_designated_dealer(account);
        Roles::assert_treasury_compliance(tc_account);
        assert(!is_synthetic_currency<CoinType>(), Errors::invalid_argument(EIS_SYNTHETIC_CURRENCY));
        assert(!exists<Preburn<CoinType>>(Signer::address_of(account)), Errors::already_published(EPREBURN));
        move_to(account, create_preburn<CoinType>(tc_account))
    }
    spec fun publish_preburn_to_account {
        modifies global<Preburn<CoinType>>(Signer::spec_address_of(account));
        /// The premission "PreburnCurrency" is granted to DesignatedDealer [[H4]][PERMISSION].
        /// Must abort if the account does not have the DesignatedDealer role.
        include Roles::AbortsIfNotDesignatedDealer;
        /// Preburn is published under the DesignatedDealer account.
        ensures exists<Preburn<CoinType>>(Signer::spec_address_of(account));

        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include AbortsIfNoCurrency<CoinType>;
        aborts_if is_synthetic_currency<CoinType>() with Errors::INVALID_ARGUMENT;
        aborts_if exists<Preburn<CoinType>>(Signer::spec_address_of(account)) with Errors::ALREADY_PUBLISHED;

    }

    ///////////////////////////////////////////////////////////////////////////

    /// Sends `coin` to the preburn queue for `account`, where it will wait to either be burned
    /// or returned to the balance of `account`.
    /// Calls to this function will fail if `account` does not have a
    /// `Preburn<CoinType>` resource published under it.
    public fun preburn_to<CoinType>(
        account: &signer,
        coin: Diem<CoinType>
    ) acquires CurrencyInfo, Preburn {
        Roles::assert_designated_dealer(account);
        let sender = Signer::address_of(account);
        assert(exists<Preburn<CoinType>>(sender), Errors::not_published(EPREBURN));
        preburn_with_resource(coin, borrow_global_mut<Preburn<CoinType>>(sender), sender);
    }
    spec fun preburn_to {
        let preburn = global<Preburn<CoinType>>(Signer::spec_address_of(account));
        include PreburnToAbortsIf<CoinType>{amount: coin.value};
        include PreburnEnsures<CoinType>{preburn: preburn, amount: coin.value};
    }

    spec schema PreburnToAbortsIf<CoinType> {
        account: signer;
        amount: u64;
        let account_addr = Signer::spec_address_of(account);
        let preburn = global<Preburn<CoinType>>(account_addr);
        /// Must abort if the account does have the Preburn resource [[H4]][PERMISSION].
        include Roles::AbortsIfNotDesignatedDealer;
        include AbortsIfNoPreburn<CoinType>{preburn_address: account_addr};
        include PreburnWithResourceAbortsIf<CoinType>{preburn: preburn};
    }

    /// Permanently removes the coins held in the `Preburn` resource (in `to_burn` field)
    /// stored at `preburn_address` and updates the market cap accordingly.
    /// This function can only be called by the holder of a `BurnCapability<CoinType>`.
    /// Calls to this function will fail if the there is no `Preburn<CoinType>`
    /// resource under `preburn_address`, or, if the `preburn.to_burn` area for
    /// `CoinType` is empty.
    public fun burn_with_capability<CoinType>(
        preburn_address: address,
        capability: &BurnCapability<CoinType>
    ) acquires CurrencyInfo, Preburn {
        // destroy the coin in the preburn to_burn area
        assert(exists<Preburn<CoinType>>(preburn_address), Errors::not_published(EPREBURN));
        burn_with_resource_cap(
            borrow_global_mut<Preburn<CoinType>>(preburn_address),
            preburn_address,
            capability
        )
    }
    spec fun burn_with_capability {
        include AbortsIfNoPreburn<CoinType>;
        include BurnWithResourceCapAbortsIf<CoinType>{preburn: global<Preburn<CoinType>>(preburn_address)};
        include BurnWithResourceCapEnsures<CoinType>{preburn: global<Preburn<CoinType>>(preburn_address)};
    }

    /// Permanently removes the coins held in the `Preburn` resource (in `to_burn` field)
    /// stored at `preburn_address` and updates the market cap accordingly.
    /// This function can only be called by the holder of a `BurnCapability<CoinType>`.
    /// Calls to this function will fail if the there is no `Preburn<CoinType>`
    /// resource under `preburn_address`, or, if the preburn `to_burn` area for
    /// `CoinType` is empty (error code 7).
    fun burn_with_resource_cap<CoinType>(
        preburn: &mut Preburn<CoinType>,
        preburn_address: address,
        _capability: &BurnCapability<CoinType>
    ) acquires CurrencyInfo {
        let currency_code = currency_code<CoinType>();
        // Abort if no coin present in preburn area
        assert(preburn.to_burn.value > 0, Errors::invalid_state(EPREBURN_EMPTY));
        // destroy the coin in Preburn area
        let Diem { value } = withdraw_all<CoinType>(&mut preburn.to_burn);
        // update the market cap
        assert_is_currency<CoinType>();
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        assert(info.total_value >= (value as u128), Errors::limit_exceeded(ECURRENCY_INFO));
        info.total_value = info.total_value - (value as u128);
        assert(info.preburn_value >= value, Errors::limit_exceeded(EPREBURN));
        info.preburn_value = info.preburn_value - value;
        // don't emit burn events for synthetic currenices as this does not
        // change the total value of fiat currencies held on-chain.
        if (!info.is_synthetic) {
            Event::emit_event(
                &mut info.burn_events,
                BurnEvent {
                    amount: value,
                    currency_code,
                    preburn_address,
                }
            );
        };
    }
    spec fun burn_with_resource_cap {
        include BurnWithResourceCapAbortsIf<CoinType>;
        include BurnWithResourceCapEnsures<CoinType>;
    }

    spec schema BurnWithResourceCapAbortsIf<CoinType> {
        preburn: Preburn<CoinType>;
        include AbortsIfNoCurrency<CoinType>;
        let to_burn = preburn.to_burn.value;
        let info = spec_currency_info<CoinType>();
        aborts_if to_burn == 0 with Errors::INVALID_STATE;
        aborts_if info.total_value < to_burn with Errors::LIMIT_EXCEEDED;
        aborts_if info.preburn_value < to_burn with Errors::LIMIT_EXCEEDED;
    }

    spec schema BurnWithResourceCapEnsures<CoinType> {
        preburn: Preburn<CoinType>;
        ensures spec_currency_info<CoinType>().total_value
                == old(spec_currency_info<CoinType>().total_value) - old(preburn.to_burn.value);
        ensures spec_currency_info<CoinType>().preburn_value
                == old(spec_currency_info<CoinType>().preburn_value) - old(preburn.to_burn.value);
    }

    /// Cancels the burn request in the `Preburn` resource stored at `preburn_address` and
    /// return the coins to the caller.
    /// This function can only be called by the holder of a
    /// `BurnCapability<CoinType>`, and will fail if the `Preburn<CoinType>` resource
    /// at `preburn_address` does not contain a pending burn request.
    public fun cancel_burn_with_capability<CoinType>(
        preburn_address: address,
        _capability: &BurnCapability<CoinType>
    ): Diem<CoinType> acquires CurrencyInfo, Preburn {
        // destroy the coin in the preburn area
        assert(exists<Preburn<CoinType>>(preburn_address), Errors::not_published(EPREBURN));
        let preburn = borrow_global_mut<Preburn<CoinType>>(preburn_address);
        let coin = withdraw_all<CoinType>(&mut preburn.to_burn);
        // update the market cap
        let currency_code = currency_code<CoinType>();
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        let amount = value(&coin);
        assert(info.preburn_value >= amount, Errors::limit_exceeded(EPREBURN));
        info.preburn_value = info.preburn_value - amount;
        // Don't emit cancel burn events for synthetic currencies. cancel_burn
        // shouldn't be be used for synthetic coins in the first place.
        if (!info.is_synthetic) {
            Event::emit_event(
                &mut info.cancel_burn_events,
                CancelBurnEvent {
                    amount,
                    currency_code,
                    preburn_address,
                }
            );
        };

        coin
    }

    spec fun cancel_burn_with_capability {
        include CancelBurnWithCapAbortsIf<CoinType>;
        include CancelBurnWithCapEnsures<CoinType>;
    }

    spec schema CancelBurnWithCapAbortsIf<CoinType> {
        preburn_address: address;
        let info = global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        let amount = global<Preburn<CoinType>>(preburn_address).to_burn.value;
        aborts_if !exists<Preburn<CoinType>>(preburn_address) with Errors::NOT_PUBLISHED;
        include AbortsIfNoCurrency<CoinType>;
        aborts_if info.preburn_value < amount with Errors::LIMIT_EXCEEDED;
    }

    spec schema CancelBurnWithCapEnsures<CoinType> {
        preburn_address: address;
        let preburn_value = global<Preburn<CoinType>>(preburn_address).to_burn.value;
        let total_preburn_value =
            global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).preburn_value;
        ensures preburn_value == 0;
        ensures total_preburn_value == old(total_preburn_value) - old(preburn_value);
    }

    /// A shortcut for immediately burning a coin. This calls preburn followed by a subsequent burn, and is
    /// used for administrative burns, like unpacking an XDX coin or charging fees.
    public fun burn_now<CoinType>(
        coin: Diem<CoinType>,
        preburn: &mut Preburn<CoinType>,
        preburn_address: address,
        capability: &BurnCapability<CoinType>
    ) acquires CurrencyInfo {
        assert(coin.value > 0, Errors::invalid_argument(ECOIN));
        preburn_with_resource(coin, preburn, preburn_address);
        burn_with_resource_cap(preburn, preburn_address, capability);
    }
    spec fun burn_now {
        include BurnNowAbortsIf<CoinType>;
        ensures preburn.to_burn.value == 0;
        let info = spec_currency_info<CoinType>();
        ensures info.total_value == old(info.total_value) - coin.value;
    }
    spec schema BurnNowAbortsIf<CoinType> {
        coin: Diem<CoinType>;
        preburn: Preburn<CoinType>;
        aborts_if coin.value == 0 with Errors::INVALID_ARGUMENT;
        include PreburnWithResourceAbortsIf<CoinType>{amount: coin.value};
        // The aborts condition for the burn is simplified because of previous call to preburn.
        let info = spec_currency_info<CoinType>();
        aborts_if info.total_value < coin.value with Errors::LIMIT_EXCEEDED;
    }

    /// Removes and returns the `BurnCapability<CoinType>` from `account`.
    /// Calls to this function will fail if `account` does  not have a
    /// published `BurnCapability<CoinType>` resource at the top-level.
    public fun remove_burn_capability<CoinType>(account: &signer): BurnCapability<CoinType>
    acquires BurnCapability {
        let addr = Signer::address_of(account);
        assert(exists<BurnCapability<CoinType>>(addr), Errors::requires_capability(EBURN_CAPABILITY));
        move_from<BurnCapability<CoinType>>(addr)
    }
    spec fun remove_burn_capability {
        include AbortsIfNoBurnCapability<CoinType>;
    }
    spec schema AbortsIfNoBurnCapability<CoinType> {
        account: signer;
        aborts_if !exists<BurnCapability<CoinType>>(Signer::spec_address_of(account)) with Errors::REQUIRES_CAPABILITY;
    }

    /// Returns the total value of `Diem<CoinType>` that is waiting to be
    /// burned throughout the system (i.e. the sum of all outstanding
    /// preburn requests across all preburn resources for the `CoinType`
    /// currency).
    public fun preburn_value<CoinType>(): u64 acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).preburn_value
    }

    /// Create a new `Diem<CoinType>` with a value of `0`. Anyone can call
    /// this and it will be successful as long as `CoinType` is a registered currency.
    public fun zero<CoinType>(): Diem<CoinType> {
        assert_is_currency<CoinType>();
        Diem<CoinType> { value: 0 }
    }

    /// Returns the `value` of the passed in `coin`. The value is
    /// represented in the base units for the currency represented by
    /// `CoinType`.
    public fun value<CoinType>(coin: &Diem<CoinType>): u64 {
        coin.value
    }

    /// Removes `amount` of value from the passed in `coin`. Returns the
    /// remaining balance of the passed in `coin`, along with another coin
    /// with value equal to `amount`. Calls will fail if `amount > Diem::value(&coin)`.
    public fun split<CoinType>(coin: Diem<CoinType>, amount: u64): (Diem<CoinType>, Diem<CoinType>) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }
    spec fun split {
        aborts_if coin.value < amount with Errors::LIMIT_EXCEEDED;
        ensures result_1.value == coin.value - amount;
        ensures result_2.value == amount;
    }


    /// Withdraw `amount` from the passed-in `coin`, where the original coin is modified in place.
    /// After this function is executed, the original `coin` will have
    /// `value = original_value - amount`, and the new coin will have a `value = amount`.
    /// Calls will abort if the passed-in `amount` is greater than the
    /// value of the passed-in `coin`.
    public fun withdraw<CoinType>(coin: &mut Diem<CoinType>, amount: u64): Diem<CoinType> {
        // Check that `amount` is less than the coin's value
        assert(coin.value >= amount, Errors::limit_exceeded(EAMOUNT_EXCEEDS_COIN_VALUE));
        coin.value = coin.value - amount;
        Diem { value: amount }
    }
    spec fun withdraw {
        pragma opaque;
        include WithdrawAbortsIf<CoinType>;
        ensures coin.value == old(coin.value) - amount;
        ensures result.value == amount;
    }
    spec schema WithdrawAbortsIf<CoinType> {
        coin: Diem<CoinType>;
        amount: u64;
        aborts_if coin.value < amount with Errors::LIMIT_EXCEEDED;
    }

    /// Return a `Diem<CoinType>` worth `coin.value` and reduces the `value` of the input `coin` to
    /// zero. Does not abort.
    public fun withdraw_all<CoinType>(coin: &mut Diem<CoinType>): Diem<CoinType> {
        let val = coin.value;
        withdraw(coin, val)
    }
    spec fun withdraw_all {
        pragma opaque;
        aborts_if false;
        ensures result.value == old(coin.value);
        ensures coin.value == 0;
    }

    /// Takes two coins as input, returns a single coin with the total value of both coins.
    /// Destroys on of the input coins.
    public fun join<CoinType>(coin1: Diem<CoinType>, coin2: Diem<CoinType>): Diem<CoinType>  {
        deposit(&mut coin1, coin2);
        coin1
    }
    spec fun join {
        pragma opaque;
        aborts_if coin1.value + coin2.value > max_u64() with Errors::LIMIT_EXCEEDED;
        ensures result.value == coin1.value + coin2.value;
    }


    /// "Merges" the two coins.
    /// The coin passed in by reference will have a value equal to the sum of the two coins
    /// The `check` coin is consumed in the process
    public fun deposit<CoinType>(coin: &mut Diem<CoinType>, check: Diem<CoinType>) {
        let Diem { value } = check;
        assert(MAX_U64 - coin.value >= value, Errors::limit_exceeded(ECOIN));
        coin.value = coin.value + value;
    }
    spec fun deposit {
        pragma opaque;
        include DepositAbortsIf<CoinType>;
        ensures coin.value == old(coin.value) + check.value;
    }
    spec schema DepositAbortsIf<CoinType> {
        coin: Diem<CoinType>;
        check: Diem<CoinType>;
        aborts_if coin.value + check.value > MAX_U64 with Errors::LIMIT_EXCEEDED;
    }

    /// Destroy a zero-value coin. Calls will fail if the `value` in the passed-in `coin` is non-zero
    /// so it is impossible to "burn" any non-zero amount of `Diem` without having
    /// a `BurnCapability` for the specific `CoinType`.
    public fun destroy_zero<CoinType>(coin: Diem<CoinType>) {
        let Diem { value } = coin;
        assert(value == 0, Errors::invalid_argument(EDESTRUCTION_OF_NONZERO_COIN))
    }
    spec fun destroy_zero {
        pragma opaque;
        aborts_if coin.value > 0 with Errors::INVALID_ARGUMENT;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Definition of Currencies
    ///////////////////////////////////////////////////////////////////////////

    /// Register the type `CoinType` as a currency. Until the type is
    /// registered as a currency it cannot be used as a coin/currency unit in Diem.
    /// The passed-in `dr_account` must be a specific address (`CoreAddresses::CURRENCY_INFO_ADDRESS()`) and
    /// `dr_account` must also have the correct `DiemRoot` account role.
    /// After the first registration of `CoinType` as a
    /// currency, additional attempts to register `CoinType` as a currency
    /// will abort.
    /// When the `CoinType` is registered it publishes the
    /// `CurrencyInfo<CoinType>` resource under the `CoreAddresses::CURRENCY_INFO_ADDRESS()` and
    /// adds the currency to the set of `RegisteredCurrencies`. It returns
    /// `MintCapability<CoinType>` and `BurnCapability<CoinType>` resources.
    public fun register_currency<CoinType>(
        dr_account: &signer,
        to_xdx_exchange_rate: FixedPoint32,
        is_synthetic: bool,
        scaling_factor: u64,
        fractional_part: u64,
        currency_code: vector<u8>,
    ): (MintCapability<CoinType>, BurnCapability<CoinType>)
    {
        Roles::assert_diem_root(dr_account);
        // Operational constraint that it must be stored under a specific address.
        CoreAddresses::assert_currency_info(dr_account);
        assert(
            !exists<CurrencyInfo<CoinType>>(Signer::address_of(dr_account)),
            Errors::already_published(ECURRENCY_INFO)
        );
        assert(0 < scaling_factor && scaling_factor <= MAX_SCALING_FACTOR, Errors::invalid_argument(ECURRENCY_INFO));
        move_to(dr_account, CurrencyInfo<CoinType> {
            total_value: 0,
            preburn_value: 0,
            to_xdx_exchange_rate,
            is_synthetic,
            scaling_factor,
            fractional_part,
            currency_code: copy currency_code,
            can_mint: true,
            mint_events: Event::new_event_handle<MintEvent>(dr_account),
            burn_events: Event::new_event_handle<BurnEvent>(dr_account),
            preburn_events: Event::new_event_handle<PreburnEvent>(dr_account),
            cancel_burn_events: Event::new_event_handle<CancelBurnEvent>(dr_account),
            exchange_rate_update_events: Event::new_event_handle<ToXDXExchangeRateUpdateEvent>(dr_account)
        });
        RegisteredCurrencies::add_currency_code(
            dr_account,
            currency_code,
        );
        (MintCapability<CoinType>{}, BurnCapability<CoinType>{})
    }
    spec fun register_currency {
        include RegisterCurrencyAbortsIf<CoinType>;
        include RegisterCurrencyEnsures<CoinType>;
    }

    spec schema RegisterCurrencyAbortsIf<CoinType> {
        dr_account: signer;
        currency_code: vector<u8>;
        scaling_factor: u64;

        /// Must abort if the signer does not have the DiemRoot role [[H8]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        aborts_if scaling_factor == 0 || scaling_factor > MAX_SCALING_FACTOR with Errors::INVALID_ARGUMENT;
        include CoreAddresses::AbortsIfNotCurrencyInfo{account: dr_account};
        aborts_if exists<CurrencyInfo<CoinType>>(Signer::spec_address_of(dr_account))
            with Errors::ALREADY_PUBLISHED;
        include RegisteredCurrencies::AddCurrencyCodeAbortsIf;
    }

    spec schema RegisterCurrencyEnsures<CoinType> {
        ensures spec_is_currency<CoinType>();
        ensures spec_currency_info<CoinType>().total_value == 0;
    }

    /// Registers a stable currency (SCS) coin -- i.e., a non-synthetic currency.
    /// Resources are published on two distinct
    /// accounts: The `CoinInfo` is published on the Diem root account, and the mint and
    /// burn capabilities are published on a treasury compliance account.
    /// This code allows different currencies to have different treasury compliance
    /// accounts.
    public fun register_SCS_currency<CoinType>(
        dr_account: &signer,
        tc_account: &signer,
        to_xdx_exchange_rate: FixedPoint32,
        scaling_factor: u64,
        fractional_part: u64,
        currency_code: vector<u8>,
    ) {
        Roles::assert_treasury_compliance(tc_account);
        let (mint_cap, burn_cap) =
            register_currency<CoinType>(
                dr_account,
                to_xdx_exchange_rate,
                false,   // is_synthetic
                scaling_factor,
                fractional_part,
                currency_code,
            );
        assert(
            !exists<MintCapability<CoinType>>(Signer::address_of(tc_account)),
            Errors::already_published(EMINT_CAPABILITY)
        );
        move_to(tc_account, mint_cap);
        publish_burn_capability<CoinType>(tc_account, burn_cap);
    }

    spec fun register_SCS_currency {
        include RegisterSCSCurrencyAbortsIf<CoinType>;
        include RegisterSCSCurrencyEnsures<CoinType>;
    }
    spec schema RegisterSCSCurrencyAbortsIf<CoinType> {
        tc_account: signer;
        dr_account: signer;
        currency_code: vector<u8>;
        scaling_factor: u64;

        /// Must abort if tc_account does not have the TreasuryCompliance role.
        /// Only a TreasuryCompliance account can have the MintCapability [[H1]][PERMISSION].
        /// Only a TreasuryCompliance account can have the BurnCapability [[H3]][PERMISSION].
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};

        aborts_if exists<MintCapability<CoinType>>(Signer::spec_address_of(tc_account)) with Errors::ALREADY_PUBLISHED;
        include RegisterCurrencyAbortsIf<CoinType>;
        include PublishBurnCapAbortsIfs<CoinType>;
    }
    spec schema RegisterSCSCurrencyEnsures<CoinType> {
        tc_account: signer;
        ensures spec_has_mint_capability<CoinType>(Signer::spec_address_of(tc_account));
    }

    /// Returns the total amount of currency minted of type `CoinType`.
    public fun market_cap<CoinType>(): u128
    acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).total_value
    }
    /// Returns the market cap of CoinType.
    spec define spec_market_cap<CoinType>(): u128 {
        global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).total_value
    }

    /// Returns the value of the coin in the `FromCoinType` currency in XDX.
    /// This should only be used where a _rough_ approximation of the exchange
    /// rate is needed.
    public fun approx_xdx_for_value<FromCoinType>(from_value: u64): u64
    acquires CurrencyInfo {
        let xdx_exchange_rate = xdx_exchange_rate<FromCoinType>();
        FixedPoint32::multiply_u64(from_value, xdx_exchange_rate)
    }
    spec fun approx_xdx_for_value {
        pragma opaque;
        include ApproxXdmForValueAbortsIf<FromCoinType>;
        ensures result == spec_approx_xdx_for_value<FromCoinType>(from_value);
    }
    spec schema ApproxXdmForValueAbortsIf<CoinType> {
        from_value: num;
        include AbortsIfNoCurrency<CoinType>;
        let xdx_exchange_rate = spec_xdx_exchange_rate<CoinType>();
        include FixedPoint32::MultiplyAbortsIf{val: from_value, multiplier: xdx_exchange_rate};
    }

    /// Returns the value of the coin in the `FromCoinType` currency in XDX.
    /// This should only be used where a rough approximation of the exchange
    /// rate is needed.
    public fun approx_xdx_for_coin<FromCoinType>(coin: &Diem<FromCoinType>): u64
    acquires CurrencyInfo {
        let from_value = value(coin);
        approx_xdx_for_value<FromCoinType>(from_value)
    }

    /// Returns `true` if the type `CoinType` is a registered currency.
    /// Returns `false` otherwise.
    public fun is_currency<CoinType>(): bool {
        exists<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS())
    }

    public fun is_SCS_currency<CoinType>(): bool acquires CurrencyInfo {
        is_currency<CoinType>() &&
        !borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).is_synthetic
    }


    /// Returns `true` if `CoinType` is a synthetic currency as defined in
    /// its `CurrencyInfo`. Returns `false` otherwise.
    public fun is_synthetic_currency<CoinType>(): bool
    acquires CurrencyInfo {
        let addr = CoreAddresses::CURRENCY_INFO_ADDRESS();
        exists<CurrencyInfo<CoinType>>(addr) &&
            borrow_global<CurrencyInfo<CoinType>>(addr).is_synthetic
    }

    /// Returns the scaling factor for the `CoinType` currency as defined
    /// in its `CurrencyInfo`.
    public fun scaling_factor<CoinType>(): u64
    acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).scaling_factor
    }
    spec define spec_scaling_factor<CoinType>(): u64 {
        global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).scaling_factor
    }

    /// Returns the representable (i.e. real-world) fractional part for the
    /// `CoinType` currency as defined in its `CurrencyInfo`.
    public fun fractional_part<CoinType>(): u64
    acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).fractional_part
    }

    /// Returns the currency code for the registered currency as defined in
    /// its `CurrencyInfo` resource.
    public fun currency_code<CoinType>(): vector<u8>
    acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        *&borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).currency_code
    }
    spec fun currency_code {
        pragma opaque;
        include AbortsIfNoCurrency<CoinType>;
        ensures result == spec_currency_info<CoinType>().currency_code;
    }

    /// Updates the `to_xdx_exchange_rate` held in the `CurrencyInfo` for
    /// `FromCoinType` to the new passed-in `xdx_exchange_rate`.
    public fun update_xdx_exchange_rate<FromCoinType>(
        tc_account: &signer,
        xdx_exchange_rate: FixedPoint32
    ) acquires CurrencyInfo {
        Roles::assert_treasury_compliance(tc_account);
        assert_is_currency<FromCoinType>();
        let currency_info = borrow_global_mut<CurrencyInfo<FromCoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        currency_info.to_xdx_exchange_rate = xdx_exchange_rate;
        Event::emit_event(
            &mut currency_info.exchange_rate_update_events,
            ToXDXExchangeRateUpdateEvent {
                currency_code: *&currency_info.currency_code,
                new_to_xdx_exchange_rate: FixedPoint32::get_raw_value(*&currency_info.to_xdx_exchange_rate),
            }
        );
    }
    spec fun update_xdx_exchange_rate {
        include UpdateXDXExchangeRateAbortsIf<FromCoinType>;
        include UpdateXDXExchangeRateEnsures<FromCoinType>;
    }
    spec schema UpdateXDXExchangeRateAbortsIf<FromCoinType> {
        tc_account: signer;
        /// Must abort if the account does not have the TreasuryCompliance Role [[H5]][PERMISSION].
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};

        include AbortsIfNoCurrency<FromCoinType>;
    }
    spec schema UpdateXDXExchangeRateEnsures<FromCoinType> {
        xdx_exchange_rate: FixedPoint32;
        ensures spec_currency_info<FromCoinType>().to_xdx_exchange_rate == xdx_exchange_rate;
    }

    /// Returns the (rough) exchange rate between `CoinType` and `XDX`
    public fun xdx_exchange_rate<CoinType>(): FixedPoint32
    acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        *&borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).to_xdx_exchange_rate
    }

    /// There may be situations in which we disallow the further minting of
    /// coins in the system without removing the currency. This function
    /// allows the association treasury compliance account to control whether or not further coins of
    /// `CoinType` can be minted or not. If this is called with `can_mint = true`,
    /// then minting is allowed, if `can_mint = false` then minting is
    /// disallowed until it is turned back on via this function. All coins
    /// start out in the default state of `can_mint = true`.
    public fun update_minting_ability<CoinType>(
        tc_account: &signer,
        can_mint: bool,
        )
    acquires CurrencyInfo {
        Roles::assert_treasury_compliance(tc_account);
        assert_is_currency<CoinType>();
        let currency_info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        currency_info.can_mint = can_mint;
    }
    spec fun update_minting_ability {
        include UpdateMintingAbilityAbortsIf<CoinType>;
        include UpdateMintingAbilityEnsures<CoinType>;
    }
    spec schema UpdateMintingAbilityAbortsIf<CoinType> {
        tc_account: signer;
        include AbortsIfNoCurrency<CoinType>;
        /// Only the TreasuryCompliance role can enable/disable minting [[H2]][PERMISSION].
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
    }
    spec schema UpdateMintingAbilityEnsures<CoinType> {
        tc_account: signer;
        can_mint: bool;
        ensures spec_currency_info<CoinType>().can_mint == can_mint;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Helper functions
    ///////////////////////////////////////////////////////////////////////////

    /// Asserts that `CoinType` is a registered currency.
    public fun assert_is_currency<CoinType>() {
        assert(is_currency<CoinType>(), Errors::not_published(ECURRENCY_INFO));
    }
    spec fun assert_is_currency {
        pragma opaque;
        include AbortsIfNoCurrency<CoinType>;
    }
    spec schema AbortsIfNoCurrency<CoinType> {
        aborts_if !spec_is_currency<CoinType>() with Errors::NOT_PUBLISHED;
    }

    public fun assert_is_SCS_currency<CoinType>() acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        assert(is_SCS_currency<CoinType>(), Errors::invalid_state(ECURRENCY_INFO));
    }
    spec schema AbortsIfNoSCSCurrency<CoinType> {
        include AbortsIfNoCurrency<CoinType>;
        aborts_if !is_SCS_currency<CoinType>() with Errors::INVALID_STATE;
    }

    // =================================================================
    // Module Specification

    spec module {} // switch documentation context back to module level

    /// # Access Control

    spec module {
        /// Only mint functions can increase the total amount of currency [[H1]][PERMISSION].
        apply TotalValueNotIncrease<CoinType> to *<CoinType>
            except mint<CoinType>, mint_with_capability<CoinType>;

        /// In order to successfully call `mint` and `mint_with_capability`, MintCapability is
        /// required. MintCapability must be only granted to a TreasuryCompliance account [[H1]][PERMISSION].
        /// Only `register_SCS_currency` creates MintCapability, which must abort if the account
        /// does not have the TreasuryCompliance role [[H1]][PERMISSION].
        apply PreserveMintCapAbsence<CoinType> to *<CoinType> except register_SCS_currency<CoinType>;
        apply Roles::AbortsIfNotTreasuryCompliance{account: tc_account} to register_SCS_currency<CoinType>;

        /// Only TreasuryCompliance can have MintCapability [[H1]][PERMISSION].
        /// If an account has MintCapability, it is a TreasuryCompliance account.
        invariant [global] forall coin_type: type:
            forall mint_cap_owner: address where exists<MintCapability<coin_type>>(mint_cap_owner):
                Roles::spec_has_treasury_compliance_role_addr(mint_cap_owner);

        /// MintCapability is not transferrable [[J1]][PERMISSION].
        apply PreserveMintCapExistence<CoinType> to *<CoinType>;

        /// The permission "MintCurrency" is unique per currency [[I1]][PERMISSION].
        /// At most one address has a mint capability for SCS CoinType
        invariant [global, isolated]
            forall coin_type: type where is_SCS_currency<coin_type>():
                forall mint_cap_owner1: address, mint_cap_owner2: address
                     where exists<MintCapability<coin_type>>(mint_cap_owner1)
                                && exists<MintCapability<coin_type>>(mint_cap_owner2):
                          mint_cap_owner1 == mint_cap_owner2;

        /// If an address has a mint capability, it is an SCS currency.
        invariant [global]
            forall coin_type: type, addr3: address where spec_has_mint_capability<coin_type>(addr3):
                is_SCS_currency<coin_type>();
    }

    /// ## Minting
    // > TODO These can be simplified using global invariants

    spec schema TotalValueNotIncrease<CoinType> {
        /// The total amount of currency does not increase.
        ensures old(spec_is_currency<CoinType>())
            ==> spec_currency_info<CoinType>().total_value <= old(spec_currency_info<CoinType>().total_value);
    }
    spec schema PreserveMintCapExistence<CoinType> {
        /// The existence of MintCapability is preserved.
        ensures forall addr: address:
            old(exists<MintCapability<CoinType>>(addr)) ==>
                exists<MintCapability<CoinType>>(addr);
    }
    spec schema PreserveMintCapAbsence<CoinType> {
        /// The absence of MintCapability is preserved.
        ensures forall addr: address:
            old(!exists<MintCapability<CoinType>>(addr)) ==>
                !exists<MintCapability<CoinType>>(addr);
    }

    /// ## Burning

    spec schema TotalValueNotDecrease<CoinType> {
        /// The total amount of currency does not decrease.
        ensures old(spec_is_currency<CoinType>())
            ==> spec_currency_info<CoinType>().total_value >= old(spec_currency_info<CoinType>().total_value);
    }
    spec schema PreserveBurnCapExistence<CoinType> {
        /// The existence of BurnCapability is preserved.
        ensures forall addr: address:
            old(exists<BurnCapability<CoinType>>(addr)) ==>
                exists<BurnCapability<CoinType>>(addr);
    }
    spec schema PreserveBurnCapAbsence<CoinType> {
        /// The absence of BurnCapability is preserved.
        ensures forall addr: address:
            old(!exists<BurnCapability<CoinType>>(addr)) ==>
                !exists<BurnCapability<CoinType>>(addr);
    }
    spec module {
        /// Only burn functions can decrease the total amount of currency [[H3]][PERMISSION].
        apply TotalValueNotDecrease<CoinType> to *<CoinType>
            except burn<CoinType>, burn_with_capability<CoinType>, burn_with_resource_cap<CoinType>,
            burn_now<CoinType>;

        /// In order to successfully call the burn functions, BurnCapability is required.
        /// BurnCapability must be only granted to a TreasuryCompliance account [[H3]][PERMISSION].
        /// Only `register_SCS_currency` and `publish_burn_capability` publish BurnCapability,
        /// which must abort if the account does not have the TreasuryCompliance role [[H8]][PERMISSION].
        apply PreserveBurnCapAbsence<CoinType> to *<CoinType>
            except register_SCS_currency<CoinType>, publish_burn_capability<CoinType>;
        apply Roles::AbortsIfNotTreasuryCompliance{account: tc_account} to register_SCS_currency<CoinType>;

        /// Only TreasuryCompliance can have BurnCapability [[H3]][PERMISSION].
        /// If an account has BurnCapability, it is a TreasuryCompliance account.
        invariant [global] forall coin_type: type:
            forall addr1: address:
                exists<BurnCapability<coin_type>>(addr1) ==>
                    Roles::spec_has_treasury_compliance_role_addr(addr1);

        /// BurnCapability is not transferrable [[J3]][PERMISSION]. BurnCapability can be extracted from an
        /// account, but is always moved back to the original account. This is the case in
        /// `TransactionFee::burn_fees` which is the only user of `remove_burn_capability` and
        /// `publish_burn_capability`.
        apply PreserveBurnCapExistence<CoinType> to *<CoinType> except remove_burn_capability<CoinType>;

        // The permission "BurnCurrency" is unique per currency [[I3]][PERMISSION]. At most one BurnCapability
        // can exist for each SCS CoinType. It is because when a new CoinType is registered in
        // `register_currency`, BurnCapability<CoinType> is packed (i.e., its instance is created)
        // only one time.
    }


    /// ## Preburning

    spec schema PreburnValueNotIncrease<CoinType> {
        /// The preburn value of currency does not increase.
        ensures old(spec_is_currency<CoinType>())
            ==> spec_currency_info<CoinType>().preburn_value <= old(spec_currency_info<CoinType>().preburn_value);
    }
    spec schema PreburnValueNotDecrease<CoinType> {
        /// The the preburn value of currency does not decrease.
        ensures old(spec_is_currency<CoinType>())
            ==> spec_currency_info<CoinType>().preburn_value >= old(spec_currency_info<CoinType>().preburn_value);
    }
    spec schema PreservePreburnExistence<CoinType> {
        /// The existence of Preburn is preserved.
        ensures forall addr: address:
            old(exists<Preburn<CoinType>>(addr)) ==>
                exists<Preburn<CoinType>>(addr);
    }
    spec schema PreservePreburnAbsence<CoinType> {
        /// The absence of Preburn is preserved.
        ensures forall addr: address:
            old(!exists<Preburn<CoinType>>(addr)) ==>
                !exists<Preburn<CoinType>>(addr);
    }

    spec module {
        /// Only burn functions can decrease the preburn value of currency [[H4]][PERMISSION].
        apply PreburnValueNotDecrease<CoinType> to *<CoinType>
            except burn<CoinType>, burn_with_capability<CoinType>, burn_with_resource_cap<CoinType>,
            burn_now<CoinType>, cancel_burn<CoinType>, cancel_burn_with_capability<CoinType>;

        /// Only preburn functions can increase the preburn value of currency [[H4]][PERMISSION].
        apply PreburnValueNotIncrease<CoinType> to *<CoinType>
            except preburn_to<CoinType>, preburn_with_resource<CoinType>;

        /// In order to successfully call the preburn functions, Preburn is required. Preburn must
        /// be only granted to a DesignatedDealer account [[H4]][PERMISSION]. Only `publish_preburn_to_account`
        /// publishes Preburn, which must abort if the account does not have the DesignatedDealer role [[H4]][PERMISSION].
        apply Roles::AbortsIfNotDesignatedDealer to publish_preburn_to_account<CoinType>;
        apply PreservePreburnAbsence<CoinType> to *<CoinType> except publish_preburn_to_account<CoinType>;

        /// Only DesignatedDealer can have Preburn [[H3]][PERMISSION].
        /// If an account has Preburn, it is a DesignatedDealer account.
        invariant [global] forall coin_type: type:
            forall addr1: address:
                exists<Preburn<coin_type>>(addr1) ==>
                    Roles::spec_has_designated_dealer_role_addr(addr1);

        /// Preburn is not transferrable [[J4]][PERMISSION].
        apply PreservePreburnExistence<CoinType> to *<CoinType>;

        /// resource struct `CurrencyInfo` is persistent
        invariant update [global] forall coin_type: type, dr_addr: address
            where old(exists<CurrencyInfo<coin_type>>(dr_addr)):
                exists<CurrencyInfo<coin_type>>(dr_addr);

        /// resource struct `Preburn<CoinType>` is persistent
        invariant update [global] forall coin_type: type, tc_addr: address
            where old(exists<Preburn<coin_type>>(tc_addr)):
                exists<Preburn<coin_type>>(tc_addr);

        /// resource struct `MintCapability<CoinType>` is persistent
        invariant update [global] forall coin_type: type, tc_addr: address
            where old(exists<MintCapability<coin_type>>(tc_addr)):
                exists<MintCapability<coin_type>>(tc_addr);
    }


    /// ## Update Exchange Rates
    spec schema ExchangeRateRemainsSame<CoinType> {
        /// The exchange rate to XDX stays constant.
        ensures old(spec_is_currency<CoinType>())
            ==> spec_currency_info<CoinType>().to_xdx_exchange_rate
                == old(spec_currency_info<CoinType>().to_xdx_exchange_rate);
    }
    spec module {
        /// The permission "UpdateExchangeRate(type)" is granted to TreasuryCompliance [[H5]][PERMISSION].
        apply Roles::AbortsIfNotTreasuryCompliance{account: tc_account} to update_xdx_exchange_rate<FromCoinType>;

        /// Only update_xdx_exchange_rate can change the exchange rate [[H5]][PERMISSION].
        apply ExchangeRateRemainsSame<CoinType> to *<CoinType>
            except update_xdx_exchange_rate<CoinType>;
    }

    /// # Helper Functions

    spec module {
        /// Checks whether currency is registered. Mirrors `Self::is_currency<CoinType>`.
        define spec_is_currency<CoinType>(): bool {
            exists<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS())
        }

        /// Returns currency information.
        define spec_currency_info<CoinType>(): CurrencyInfo<CoinType> {
            global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS())
        }

        /// Specification version of `Self::approx_xdx_for_value`.
        define spec_approx_xdx_for_value<CoinType>(value: num):  num {
            FixedPoint32::spec_multiply_u64(value, spec_xdx_exchange_rate<CoinType>())
        }

        define spec_xdx_exchange_rate<CoinType>(): FixedPoint32 {
            global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).to_xdx_exchange_rate
        }

        /// Checks whether the currency has a mint capability.  This is only relevant for
        /// SCS coins
        define spec_has_mint_capability<CoinType>(addr: address): bool {
            exists<MintCapability<CoinType>>(addr)
        }

        /// Returns true if a BurnCapability for CoinType exists at addr.
        define spec_has_burn_capability<CoinType>(addr: address): bool {
            exists<BurnCapability<CoinType>>(addr)
        }
    }

}
}
