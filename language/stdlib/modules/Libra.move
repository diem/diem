address 0x1 {

/// The `Libra` module describes the concept of a coin in the Libra framework. It introduces the
/// resource `Libra::Libra<CoinType>`, representing a coin of given coin type.
/// The module defines functions operating on coins as well as functionality like
/// minting and burning of coins.
module Libra {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::Event::{Self, EventHandle};
    use 0x1::FixedPoint32::{Self, FixedPoint32};
    use 0x1::RegisteredCurrencies;
    use 0x1::Signer;
    use 0x1::Roles;
    use 0x1::LibraTimestamp;

    resource struct RegisterNewCurrency {}

    /// The `Libra` resource defines the Libra coin for each currency in
    /// Libra. Each "coin" is coupled with a type `CoinType` specifying the
    /// currency of the coin, and a `value` field specifying the value
    /// of the coin (in the base units of the currency `CoinType`
    /// and specified in the `CurrencyInfo` resource for that `CoinType`
    /// published under the `CoreAddresses::CURRENCY_INFO_ADDRESS()` account address).
    resource struct Libra<CoinType> {
        /// The value of this coin in the base units for `CoinType`
        value: u64
    }

    /// The `MintCapability` resource defines a capability to allow minting
    /// of coins of `CoinType` currency by the holder of this capability.
    /// This capability is held only either by the `CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()`
    /// account or the `0x1::LBR` module (and `CoreAddresses::LIBRA_ROOT_ADDRESS()` in testnet).
    resource struct MintCapability<CoinType> { }

    /// The `BurnCapability` resource defines a capability to allow coins
    /// of `CoinType` currency to be burned by the holder of the
    /// and the `0x1::LBR` module (and `CoreAddresses::LIBRA_ROOT_ADDRESS()` in testnet).
    resource struct BurnCapability<CoinType> { }

    /// The `CurrencyRegistrationCapability` is a singleton resource
    /// published under the `CoreAddresses::LIBRA_ROOT_ADDRESS()` and grants
    /// the capability to the `0x1::Libra` module to add currencies to the
    /// `0x1::RegisteredCurrencies` on-chain config.

    /// A `MintEvent` is emitted every time a Libra coin is minted. This
    /// contains the `amount` minted (in base units of the currency being
    /// minted) along with the `currency_code` for the coin(s) being
    /// minted, and that is defined in the `currency_code` field of the
    /// `CurrencyInfo` resource for the currency.
    struct MintEvent {
        /// Funds added to the system
        amount: u64,
        /// ASCII encoded symbol for the coin type (e.g., "LBR")
        currency_code: vector<u8>,
    }

    /// A `BurnEvent` is emitted every time a non-synthetic[1] Libra coin is
    /// burned. It contains the `amount` burned in base units for the
    /// currency, along with the `currency_code` for the coins being burned
    /// (and as defined in the `CurrencyInfo` resource for that currency).
    /// It also contains the `preburn_address` from which the coin is
    /// extracted for burning.
    /// [1] As defined by the `is_synthetic` field in the `CurrencyInfo`
    /// for that currency.
    struct BurnEvent {
        /// Funds removed from the system
        amount: u64,
        /// ASCII encoded symbol for the coin type (e.g., "LBR")
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
        /// ASCII encoded symbol for the coin type (e.g., "LBR")
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
        /// ASCII encoded symbol for the coin type (e.g., "LBR")
        currency_code: vector<u8>,
        /// Address of the `Preburn` resource that held the now-returned funds.
        preburn_address: address,
    }

    /// An `ToLBRExchangeRateUpdateEvent` is emitted every time the to-LBR exchange
    /// rate for the currency given by `currency_code` is updated.
    struct ToLBRExchangeRateUpdateEvent {
        /// The currency code of the currency whose exchange rate was updated.
        currency_code: vector<u8>,
        /// The new on-chain to-LBR exchange rate between the
        /// `currency_code` currency and LBR. Represented in conversion
        /// between the (on-chain) base-units for the currency and microlibra.
        new_to_lbr_exchange_rate: u64,
    }

    /// The `CurrencyInfo<CoinType>` resource stores the various
    /// pieces of information needed for a currency (`CoinType`) that is
    /// registered on-chain. This resource _must_ be published under the
    /// address given by `CoreAddresses::CURRENCY_INFO_ADDRESS()` in order for the registration of
    /// `CoinType` as a recognized currency on-chain to be successful. At
    /// the time of registration the `MintCapability<CoinType>` and
    /// `BurnCapability<CoinType>` capabilities are returned to the caller.
    /// Unless they are specified otherwise the fields in this resource are immutable.
    resource struct CurrencyInfo<CoinType> {
        /// The total value for the currency represented by `CoinType`. Mutable.
        total_value: u128,
        /// Value of funds that are in the process of being burned.  Mutable.
        preburn_value: u64,
        /// The (rough) exchange rate from `CoinType` to `LBR`. Mutable.
        to_lbr_exchange_rate: FixedPoint32,
        /// Holds whether or not this currency is synthetic (contributes to the
        /// off-chain reserve) or not. An example of such a synthetic
        ///currency would be the LBR.
        is_synthetic: bool,
        /// The scaling factor for the coin (i.e. the amount to multiply by
        /// to get to the human-readable representation for this currency).
        /// e.g. 10^6 for `Coin1`
        ///
        /// > TODO(wrwg): should the above be "to divide by"?
        scaling_factor: u64,
        /// The smallest fractional part (number of decimal places) to be
        /// used in the human-readable representation for the currency (e.g.
        /// 10^2 for `Coin1` cents)
        fractional_part: u64,
        /// The code symbol for this `CoinType`. ASCII encoded.
        /// e.g. for "LBR" this is x"4C4252". No character limit.
        currency_code: vector<u8>,
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
        exchange_rate_update_events: EventHandle<ToLBRExchangeRateUpdateEvent>,
    }

    /// A holding area where funds that will subsequently be burned wait while their underlying
    /// assets are moved off-chain.
    /// This resource can only be created by the holder of a `BurnCapability`. An account that
    /// contains this address has the authority to initiate a burn request. A burn request can be
    /// resolved by the holder of a `BurnCapability` by either (1) burning the funds, or (2)
    /// returning the funds to the account that initiated the burn request.
    /// Concurrent preburn requests are not allowed, only one request (in to_burn) can be handled at any time.
    resource struct Preburn<CoinType> {
        /// A single pending burn amount.
        /// There is no pending burn request if the value in to_burn is 0
        to_burn: Libra<CoinType>,
    }

    /// TODO(wrwg): This should be provided somewhere centrally in the framework.
    const MAX_U64: u64 = 18446744073709551615;
    const MAX_U128: u128 = 340282366920938463463374607431768211455;

    const EBURN_CAPABILITY: u64 = 0;
    const ECURRENCY_INFO: u64 = 1;
    const EPREBURN: u64 = 2;
    const EPREBURN_OCCUPIED: u64 = 3;
    const EPREBURN_EMPTY: u64 = 4;
    const EMINTING_NOT_ALLOWED: u64 = 5;
    const EIS_SYNTHETIC_CURRENCY: u64 = 6;
    const ECOIN: u64 = 7;
    const EDESTRUCTION_OF_NONZERO_COIN: u64 = 8;
    const EREGISTRATION_PRIVILEGE: u64 = 9;
    const EMINT_CAPABILITY: u64 = 10;
    const EAMOUNT_EXCEEDS_COIN_VALUE: u64 = 11;

    ///////////////////////////////////////////////////////////////////////////
    // Initialization and granting of privileges
    ///////////////////////////////////////////////////////////////////////////

    /// Grants the `RegisterNewCurrency` privilege to
    /// the calling account as long as it has the correct role (TC).
    /// Aborts if `account` does not have a `RoleId` that corresponds with
    /// the treacury compliance role.
    // public fun grant_privileges(account: &signer) {
    // }

    /// Initialization of the `Libra` module; initializes the set of
    /// registered currencies in the `0x1::RegisteredCurrencies` on-chain
    /// config, and publishes the `CurrencyRegistrationCapability` under the
    /// `CoreAddresses::LIBRA_ROOT_ADDRESS()`. This can only be called from genesis.
    public fun initialize(
        config_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        // Operational constraint
        CoreAddresses::assert_libra_root(config_account);
        RegisteredCurrencies::initialize(config_account);
    }

    /// Publishes the `BurnCapability` `cap` for the `CoinType` currency under `account`. `CoinType`
    /// must be a registered currency type. The caller must pass a treasury compliance account.
    public fun publish_burn_capability<CoinType>(
        account: &signer,
        cap: BurnCapability<CoinType>,
        tc_account: &signer,
    ) {
        Roles::assert_treasury_compliance(tc_account);
        assert_is_currency<CoinType>();
        assert(
            !exists<BurnCapability<CoinType>>(Signer::address_of(account)),
            Errors::already_published(EBURN_CAPABILITY)
        );
        move_to(account, cap)
    }
    spec fun publish_burn_capability {
        aborts_if !spec_is_currency<CoinType>();
        include PublishBurnCapAbortsIfs<CoinType>;
    }
    spec schema PublishBurnCapAbortsIfs<CoinType> {
        account: &signer;
        tc_account: &signer;
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        aborts_if exists<BurnCapability<CoinType>>(Signer::spec_address_of(account)) with Errors::ALREADY_PUBLISHED;
    }
    /// Returns true if a BurnCapability for CoinType exists at addr.
    spec define spec_has_burn_cap<CoinType>(addr: address): bool {
        exists<BurnCapability<CoinType>>(addr)
    }

    /// Mints `amount` coins. The `account` must hold a
    /// `MintCapability<CoinType>` at the top-level in order for this call
    /// to be successful, and will fail with `MISSING_DATA` otherwise.
    public fun mint<CoinType>(account: &signer, value: u64): Libra<CoinType>
    acquires CurrencyInfo, MintCapability {
        mint_with_capability(
            value,
            borrow_global<MintCapability<CoinType>>(Signer::address_of(account))
        )
    }
    spec fun mint {
        aborts_if !exists<MintCapability<CoinType>>(Signer::spec_address_of(account));
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
        assert(exists<BurnCapability<CoinType>>(addr), Errors::not_published(EBURN_CAPABILITY));
        burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<CoinType>>(Signer::address_of(account))
        )
    }
    spec fun burn {
        // TODO: There was a timeout (> 40s) for this function in CI. Verification turned off.
        pragma verify = false;
        aborts_if !exists<BurnCapability<CoinType>>(Signer::spec_address_of(account)) with Errors::NOT_PUBLISHED;
    }

    /// Cancels the current burn request in the `Preburn` resource held
    /// under the `preburn_address`, and returns the coins.
    /// Calls to this will fail if the sender does not have a published
    /// `BurnCapability<CoinType>`, or if there is no preburn request
    /// outstanding in the `Preburn` resource under `preburn_address`.
    public fun cancel_burn<CoinType>(
        account: &signer,
        preburn_address: address
    ): Libra<CoinType> acquires BurnCapability, CurrencyInfo, Preburn {
        let addr = Signer::address_of(account);
        assert(exists<BurnCapability<CoinType>>(addr), Errors::not_published(EBURN_CAPABILITY));
        cancel_burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<CoinType>>(addr)
        )
    }

    /// Mint a new `Libra` coin of `CoinType` currency worth `value`. The
    /// caller must have a reference to a `MintCapability<CoinType>`. Only
    /// the treasury compliance account or the `0x1::LBR` module can acquire such a
    /// reference.
    public fun mint_with_capability<CoinType>(
        value: u64,
        _capability: &MintCapability<CoinType>
    ): Libra<CoinType> acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        let currency_code = currency_code<CoinType>();
        // update market cap resource to reflect minting
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        assert(info.can_mint, Errors::invalid_state(EMINTING_NOT_ALLOWED));
        assert(MAX_U128 - info.total_value >= (value as u128), Errors::limit_exceeded(ECURRENCY_INFO));
        info.total_value = info.total_value + (value as u128);
        // don't emit mint events for synthetic currenices
        if (!info.is_synthetic) {
            Event::emit_event(
                &mut info.mint_events,
                MintEvent{
                    amount: value,
                    currency_code,
                }
            );
        };

        Libra<CoinType> { value }
    }
    spec fun mint_with_capability {
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
        result: Libra<CoinType>;
        ensures spec_currency_info<CoinType>().total_value
                    == old(spec_currency_info<CoinType>().total_value) + value;
        ensures result.value == value;
    }

    /// Add the `coin` to the `preburn` to_burn field in the `Preburn` resource
    /// held at the address `preburn_address` if it is empty, otherwise raise
    /// a PendingPreburn Error (code 6). Emits a `PreburnEvent` to
    /// the `preburn_events` event stream in the `CurrencyInfo` for the
    /// `CoinType` passed in. However, if the currency being preburned is
    /// `synthetic` then no `PreburnEvent` event will be emitted.
    public fun preburn_with_resource<CoinType>(
        coin: Libra<CoinType>,
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
        // don't emit preburn events for synthetic currencies
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
        include PreburnWithResourceAbortsIf<CoinType>;
        include PreburnEnsures<CoinType>;
    }
    spec schema PreburnWithResourceAbortsIf<CoinType> {
        coin: Libra<CoinType>;
        preburn: Preburn<CoinType>;
        aborts_if preburn.to_burn.value != 0 with Errors::INVALID_STATE;
        include PreburnAbortsIf<CoinType>;
    }
    spec schema PreburnAbortsIf<CoinType> {
        coin: Libra<CoinType>;
        include AbortsIfNoCurrency<CoinType>;
        aborts_if spec_currency_info<CoinType>().preburn_value + coin.value > MAX_U64 with Errors::LIMIT_EXCEEDED;
    }
    spec schema PreburnEnsures<CoinType> {
        coin: Libra<CoinType>;
        preburn: Preburn<CoinType>;
        ensures spec_currency_info<CoinType>().preburn_value
                    == old(spec_currency_info<CoinType>().preburn_value) + coin.value;
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
        include Roles::AbortsIfNotDesignatedDealer;
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
        coin: Libra<CoinType>
    ) acquires CurrencyInfo, Preburn {
        let sender = Signer::address_of(account);
        assert(exists<Preburn<CoinType>>(sender), Errors::not_published(EPREBURN));
        preburn_with_resource(coin, borrow_global_mut<Preburn<CoinType>>(sender), sender);
    }
    spec fun preburn_to {
        aborts_if !exists<Preburn<CoinType>>(Signer::spec_address_of(account)) with Errors::NOT_PUBLISHED;
        aborts_if global<Preburn<CoinType>>(Signer::spec_address_of(account)).to_burn.value != 0
            with Errors::INVALID_STATE;
        include PreburnAbortsIf<CoinType>;
        include PreburnEnsures<CoinType>{preburn: global<Preburn<CoinType>>(Signer::spec_address_of(account))};
    }

    /// Permanently removes the coins held in the `Preburn` resource (in to_burn field)
    /// stored at `preburn_address` and updates the market cap accordingly.
    /// This function can only be called by the holder of a `BurnCapability<CoinType>`.
    /// Calls to this function will fail if the there is no `Preburn<CoinType>`
    /// resource under `preburn_address`, or, if the preburn to_burn area for
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
        aborts_if !exists<Preburn<CoinType>>(preburn_address) with Errors::NOT_PUBLISHED;
        include BurnAbortsIf<CoinType>{preburn: global<Preburn<CoinType>>(preburn_address)};
        include BurnEnsures<CoinType>{preburn: global<Preburn<CoinType>>(preburn_address)};
    }

    /// Permanently removes the coins held in the `Preburn` resource (in to_burn field)
    /// stored at `preburn_address` and updates the market cap accordingly.
    /// This function can only be called by the holder of a `BurnCapability<CoinType>`.
    /// Calls to this function will fail if the there is no `Preburn<CoinType>`
    /// resource under `preburn_address`, or, if the preburn to_burn area for
    /// `CoinType` is empty (error code 7).
    public fun burn_with_resource_cap<CoinType>(
        preburn: &mut Preburn<CoinType>,
        preburn_address: address,
        _capability: &BurnCapability<CoinType>
    ) acquires CurrencyInfo {
        let currency_code = currency_code<CoinType>();
        // Abort if no coin present in preburn area
        assert(preburn.to_burn.value > 0, Errors::invalid_state(EPREBURN_EMPTY));
        // destroy the coin in Preburn area
        let Libra { value } = withdraw_all<CoinType>(&mut preburn.to_burn);
        // update the market cap
        assert_is_currency<CoinType>();
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        assert(info.total_value >= (value as u128), Errors::limit_exceeded(ECURRENCY_INFO));
        info.total_value = info.total_value - (value as u128);
        assert(info.preburn_value >= value, Errors::limit_exceeded(EPREBURN));
        info.preburn_value = info.preburn_value - value;
        // don't emit burn events for synthetic currencies
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
        include BurnAbortsIf<CoinType>;
        include BurnEnsures<CoinType>;
    }

    spec schema BurnAbortsIf<CoinType> {
        preburn: Preburn<CoinType>;
        include AbortsIfNoCurrency<CoinType>;
        let to_burn = preburn.to_burn.value;
        let info = spec_currency_info<CoinType>();
        aborts_if to_burn == 0 with Errors::INVALID_STATE;
        aborts_if info.total_value < to_burn with Errors::LIMIT_EXCEEDED;
        aborts_if info.preburn_value < to_burn with Errors::LIMIT_EXCEEDED;
    }

    spec schema BurnEnsures<CoinType> {
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
    ): Libra<CoinType> acquires CurrencyInfo, Preburn {
        // destroy the coin in the preburn area
        let preburn = borrow_global_mut<Preburn<CoinType>>(preburn_address);
        let coin = withdraw_all<CoinType>(&mut preburn.to_burn);
        // update the market cap
        let currency_code = currency_code<CoinType>();
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        let amount = value(&coin);
        assert(info.preburn_value >= amount, Errors::limit_exceeded(EPREBURN));
        info.preburn_value = info.preburn_value - amount;
        // Don't emit cancel burn events for synthetic currencies. cancel burn shouldn't be be used
        // for synthetics in the first place
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

    /// A shortcut for immediately burning a coin. This calls preburn followed by a subsequent burn, and is
    /// used for administrative burns, like unpacking an LBR coin or charging fees.
    /// > TODO(wrwg): consider removing complexity here by removing the need for a preburn resource. The preburn
    /// > resource is required to have 0 value on entry and will have so also after this call, so it is redundant.
    public fun burn_now<CoinType>(
        coin: Libra<CoinType>,
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
        coin: Libra<CoinType>;
        preburn: Preburn<CoinType>;
        aborts_if coin.value == 0 with Errors::INVALID_ARGUMENT;
        include PreburnWithResourceAbortsIf<CoinType>;
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
        assert(exists<BurnCapability<CoinType>>(addr), Errors::requires_privilege(EBURN_CAPABILITY));
        move_from<BurnCapability<CoinType>>(addr)
    }
    spec fun remove_burn_capability {
        include AbortsIfNoBurnCapability<CoinType>;
    }
    spec schema AbortsIfNoBurnCapability<CoinType> {
        account: signer;
        aborts_if !exists<BurnCapability<CoinType>>(Signer::spec_address_of(account)) with Errors::REQUIRES_PRIVILEGE;
    }

    /// Returns the total value of `Libra<CoinType>` that is waiting to be
    /// burned throughout the system (i.e. the sum of all outstanding
    /// preburn requests across all preburn resources for the `CoinType`
    /// currency).
    public fun preburn_value<CoinType>(): u64 acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).preburn_value
    }

    /// Create a new `Libra<CoinType>` with a value of `0`. Anyone can call
    /// this and it will be successful as long as `CoinType` is a registered currency.
    public fun zero<CoinType>(): Libra<CoinType> {
        assert_is_currency<CoinType>();
        Libra<CoinType> { value: 0 }
    }

    /// Returns the `value` of the passed in `coin`. The value is
    /// represented in the base units for the currency represented by
    /// `CoinType`.
    public fun value<CoinType>(coin: &Libra<CoinType>): u64 {
        coin.value
    }

    /// Removes `amount` of value from the passed in `coin`. Returns the
    /// remaining balance of the passed in `coin`, along with another coin
    /// with value equal to `amount`. Calls will fail if `amount > Libra::value(&coin)`.
    public fun split<CoinType>(coin: Libra<CoinType>, amount: u64): (Libra<CoinType>, Libra<CoinType>) {
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
    public fun withdraw<CoinType>(coin: &mut Libra<CoinType>, amount: u64): Libra<CoinType> {
        // Check that `amount` is less than the coin's value
        assert(coin.value >= amount, Errors::limit_exceeded(EAMOUNT_EXCEEDS_COIN_VALUE));
        coin.value = coin.value - amount;
        Libra { value: amount }
    }
    spec fun withdraw {
        pragma opaque;
        include WithdrawAbortsIf<CoinType>;
        ensures coin.value == old(coin.value) - amount;
        ensures result.value == amount;
    }
    spec schema WithdrawAbortsIf<CoinType> {
        coin: Libra<CoinType>;
        amount: u64;
        aborts_if coin.value < amount with Errors::LIMIT_EXCEEDED;
    }

    /// Return a `Libra<CoinType>` worth `coin.value` and reduces the `value` of the input `coin` to
    /// zero. Does not abort.
    public fun withdraw_all<CoinType>(coin: &mut Libra<CoinType>): Libra<CoinType> {
        let val = coin.value;
        withdraw(coin, val)
    }
    spec fun withdraw_all {
        pragma opaque;
        aborts_if false;
        ensures result.value == old(coin.value);
        ensures coin.value == 0;
    }

    /// and returns a new coin whose value is equal to the sum of the two inputs.
    public fun join<CoinType>(coin1: Libra<CoinType>, coin2: Libra<CoinType>): Libra<CoinType>  {
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
    public fun deposit<CoinType>(coin: &mut Libra<CoinType>, check: Libra<CoinType>) {
        let Libra { value } = check;
        assert(MAX_U64 - coin.value >= value, Errors::limit_exceeded(ECOIN));
        coin.value = coin.value + value;
    }
    spec fun deposit {
        pragma opaque;
        include DepositAbortsIf<CoinType>;
        ensures coin.value == old(coin.value) + check.value;
    }
    spec schema DepositAbortsIf<CoinType> {
        coin: Libra<CoinType>;
        check: Libra<CoinType>;
        aborts_if coin.value + check.value > MAX_U64 with Errors::LIMIT_EXCEEDED;
    }

    /// Destroy a zero-value coin. Calls will fail if the `value` in the passed-in `coin` is non-zero
    /// so you cannot "burn" any non-zero amount of `Libra` without having
    /// a `BurnCapability` for the specific `CoinType`.
    public fun destroy_zero<CoinType>(coin: Libra<CoinType>) {
        let Libra { value } = coin;
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
    /// registered as a currency it cannot be used as a coin/currency unit in Libra.
    /// The passed-in `lr_account` must be a specific address (`CoreAddresses::CURRENCY_INFO_ADDRESS()`) and
    /// `lr_account` must also have the correct `RegisterNewCurrency` capability.
    /// After the first registration of `CoinType` as a
    /// currency, additional attempts to register `CoinType` as a currency
    /// will abort.
    /// When the `CoinType` is registered it publishes the
    /// `CurrencyInfo<CoinType>` resource under the `CoreAddresses::CURRENCY_INFO_ADDRESS()` and
    /// adds the currency to the set of `RegisteredCurrencies`. It returns
    /// `MintCapability<CoinType>` and `BurnCapability<CoinType>` resources.
    public fun register_currency<CoinType>(
        lr_account: &signer,
        to_lbr_exchange_rate: FixedPoint32,
        is_synthetic: bool,
        scaling_factor: u64,
        fractional_part: u64,
        currency_code: vector<u8>,
    ): (MintCapability<CoinType>, BurnCapability<CoinType>)
    {
        assert(Roles::has_register_new_currency_privilege(lr_account), Errors::requires_role(EREGISTRATION_PRIVILEGE));
        // Operational constraint that it must be stored under a specific address.
        CoreAddresses::assert_currency_info(lr_account);
        assert(
            !exists<CurrencyInfo<CoinType>>(Signer::address_of(lr_account)),
            Errors::already_published(ECURRENCY_INFO)
        );
        move_to(lr_account, CurrencyInfo<CoinType> {
            total_value: 0,
            preburn_value: 0,
            to_lbr_exchange_rate,
            is_synthetic,
            scaling_factor,
            fractional_part,
            currency_code: copy currency_code,
            can_mint: true,
            mint_events: Event::new_event_handle<MintEvent>(lr_account),
            burn_events: Event::new_event_handle<BurnEvent>(lr_account),
            preburn_events: Event::new_event_handle<PreburnEvent>(lr_account),
            cancel_burn_events: Event::new_event_handle<CancelBurnEvent>(lr_account),
            exchange_rate_update_events: Event::new_event_handle<ToLBRExchangeRateUpdateEvent>(lr_account)
        });
        RegisteredCurrencies::add_currency_code(
            lr_account,
            currency_code,
        );
        (MintCapability<CoinType>{}, BurnCapability<CoinType>{})
    }
    spec fun register_currency {
        include RegisterCurrencyAbortsIf<CoinType>;
        ensures spec_is_currency<CoinType>();
        ensures spec_currency_info<CoinType>().total_value == 0;
    }

    spec schema RegisterCurrencyAbortsIf<CoinType> {
        lr_account: signer;
        currency_code: vector<u8>;
        aborts_if !Roles::spec_has_register_new_currency_privilege_addr(Signer::spec_address_of(lr_account))
            with Errors::REQUIRES_ROLE;
        include CoreAddresses::AbortsIfNotCurrencyInfo{account: lr_account};
        aborts_if exists<CurrencyInfo<CoinType>>(Signer::spec_address_of(lr_account))
            with Errors::ALREADY_PUBLISHED;
        include RegisteredCurrencies::AddCurrencyCodeAbortsIf;
    }

    /// Registers a stable currency (SCS) coin -- i.e., a non-synthetic currency.
    /// Resources are published on two distinct
    /// accounts: The CoinInfo is published on the Libra root account, and the mint and
    /// burn capabilities are published on a treasury compliance account.
    /// This code allows different currencies to have different treasury compliance
    /// accounts.
    public fun register_SCS_currency<CoinType>(
        lr_account: &signer,
        tc_account: &signer,
        to_lbr_exchange_rate: FixedPoint32,
        scaling_factor: u64,
        fractional_part: u64,
        currency_code: vector<u8>,
    ) {
        Roles::assert_treasury_compliance(tc_account);
        let (mint_cap, burn_cap) =
            register_currency<CoinType>(
                lr_account,
                to_lbr_exchange_rate,
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
        publish_burn_capability<CoinType>(tc_account, burn_cap, tc_account);
    }

    spec fun register_SCS_currency {
        aborts_if exists<MintCapability<CoinType>>(Signer::spec_address_of(tc_account));
        include RegisterCurrencyAbortsIf<CoinType>;
        include PublishBurnCapAbortsIfs<CoinType>{account: tc_account};
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

    /// Returns the value of the coin in the `FromCoinType` currency in LBR.
    /// This should only be used where a _rough_ approximation of the exchange
    /// rate is needed.
    public fun approx_lbr_for_value<FromCoinType>(from_value: u64): u64
    acquires CurrencyInfo {
        let lbr_exchange_rate = lbr_exchange_rate<FromCoinType>();
        FixedPoint32::multiply_u64(from_value, lbr_exchange_rate)
    }
    spec fun approx_lbr_for_value {
        pragma opaque;
        include ApproxLbrForValueAbortsIf<FromCoinType>;
        ensures result == spec_approx_lbr_for_value<FromCoinType>(from_value);
    }
    spec schema ApproxLbrForValueAbortsIf<CoinType> {
        from_value: num;
        include AbortsIfNoCurrency<CoinType>;
        let lbr_exchange_rate = spec_lbr_exchange_rate<CoinType>();
        include FixedPoint32::MultiplyAbortsIf{val: from_value, multiplier: lbr_exchange_rate};
    }

    /// Returns the value of the coin in the `FromCoinType` currency in LBR.
    /// This should only be used where a rough approximation of the exchange
    /// rate is needed.
    public fun approx_lbr_for_coin<FromCoinType>(coin: &Libra<FromCoinType>): u64
    acquires CurrencyInfo {
        let from_value = value(coin);
        approx_lbr_for_value<FromCoinType>(from_value)
    }

    /// Returns `true` if the type `CoinType` is a registered currency.
    /// Returns `false` otherwise.
    public fun is_currency<CoinType>(): bool {
        exists<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS())
    }

    public fun is_SCS_currency<CoinType>(): bool acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        let info = borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        !info.is_synthetic
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

    /// Updates the `to_lbr_exchange_rate` held in the `CurrencyInfo` for
    /// `FromCoinType` to the new passed-in `lbr_exchange_rate`.
    public fun update_lbr_exchange_rate<FromCoinType>(
        tc_account: &signer,
        lbr_exchange_rate: FixedPoint32
    ) acquires CurrencyInfo {
        Roles::assert_treasury_compliance(tc_account);
        assert_is_currency<FromCoinType>();
        let currency_info = borrow_global_mut<CurrencyInfo<FromCoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        currency_info.to_lbr_exchange_rate = lbr_exchange_rate;
        Event::emit_event(
            &mut currency_info.exchange_rate_update_events,
            ToLBRExchangeRateUpdateEvent {
                currency_code: *&currency_info.currency_code,
                new_to_lbr_exchange_rate: FixedPoint32::get_raw_value(*&currency_info.to_lbr_exchange_rate),
            }
        );
    }
    spec fun update_lbr_exchange_rate {
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include AbortsIfNoCurrency<FromCoinType>;
        ensures spec_currency_info<FromCoinType>().to_lbr_exchange_rate == lbr_exchange_rate;
    }


    /// Returns the (rough) exchange rate between `CoinType` and `LBR`
    public fun lbr_exchange_rate<CoinType>(): FixedPoint32
    acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        *&borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).to_lbr_exchange_rate
    }

    /// There may be situations in which we disallow the further minting of
    /// coins in the system without removing the currency. This function
    /// allows the association TC account to control whether or not further coins of
    /// `CoinType` can be minted or not. If this is called with `can_mint =
    /// true`, then minting is allowed, if `can_mint = false` then minting is
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

    fun assert_is_SCS_currency<CoinType>() acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        assert(is_SCS_currency<CoinType>(), Errors::invalid_state(ECURRENCY_INFO));
    }
    spec schema AbortsIfNoSCSCurrency<CoinType> {
        include AbortsIfNoCurrency<CoinType>;
        aborts_if !spec_is_SCS_currency<CoinType>() with Errors::INVALID_STATE;
    }


    /// **************** MODULE SPECIFICATION ****************

    /// # Module Specification

    spec module {
        /// Verify all functions in this module.
        pragma verify = true;
    }

    spec module {
        /// Checks whether currency is registered. Mirrors `Self::is_currency<CoinType>`.
        define spec_is_currency<CoinType>(): bool {
            exists<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS())
        }

        /// Returns currency information.
        define spec_currency_info<CoinType>(): CurrencyInfo<CoinType> {
            global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS())
        }

        /// Specification version of `Self::approx_lbr_for_value`.
        define spec_approx_lbr_for_value<CoinType>(value: num):  num {
            FixedPoint32::spec_multiply_u64(value, spec_lbr_exchange_rate<CoinType>())
        }

        define spec_lbr_exchange_rate<CoinType>(): FixedPoint32 {
            global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).to_lbr_exchange_rate
        }

        define spec_is_SCS_currency<CoinType>(): bool {
            spec_is_currency<CoinType>() && !spec_currency_info<CoinType>().is_synthetic
        }

        /// Checks whether the currency has a mint capability.  This is only relevant for
        /// SCS coins
        define spec_has_mint_capability<CoinType>(addr1: address): bool {
            exists<MintCapability<CoinType>>(addr1)
        }

    }

    /// ## Minting

    /// For an SCS coin, the mint capability cannot move or disappear.
    /// TODO: Specify that they're published at the one true treasurycompliance address?

    spec module {

        /*
        TODO: the below invariants cause various termination problems. Marking them as on_update only
          does not help to resolve this. Need to find a away to impose those invariants only as module
          invariants but optimized for actual relevance as global invariants are. (Making them module
          invariants also does not help termination right now.)

        /// If an address has a mint capability, it is an SCS currency.
        invariant [global]
            forall coin_type: type, addr3: address where spec_has_mint_capability<coin_type>(addr3):
                spec_is_SCS_currency<coin_type>();

        /// If there is a pending offer for a mint capability, the coin_type is an SCS currency and
        /// there are no published Mint Capabilities. (This is the state after register_SCS_currency_start)
        invariant [global]
            forall coin_type: type :
                spec_is_SCS_currency<coin_type>()
                && (forall addr3: address : !spec_has_mint_capability<coin_type>(addr3));

        // At most one address has a mint capability for SCS CoinType
        invariant [global]
            forall coin_type: type where spec_is_SCS_currency<coin_type>():
                forall addr1: address, addr2: address
                     where exists<MintCapability<coin_type>>(addr1) && exists<MintCapability<coin_type>>(addr2):
                          addr1 == addr2;

        // Once a MintCapability appears at an address, it stays there.
        invariant update [global]
            forall coin_type: type:
                forall addr1: address where old(exists<MintCapability<coin_type>>(addr1)):
                    exists<MintCapability<coin_type>>(addr1);

        // If address has a mint capability, it has the treasury compliance role
        invariant [global]
            forall coin_type: type:
                forall addr1: address where exists<MintCapability<coin_type>>(addr1):
                     Roles::spec_has_treasury_compliance_role_addr(addr1);

        */

    }


    /// ## Conservation of currency

    /// TODO (dd): Unfortunately, improvements to the memory model have made it
    /// difficult to compute sum_of_coin_values correctly. We will need
    /// another approach for expressing this invariant.

    // spec module {
    //     /// Maintain a spec variable representing the sum of
    //     /// all coins of a currency type.
    //     global sum_of_coin_values<CoinType>: num;
    // }

    // /// Account for updating `sum_of_coin_values` when a `Libra` is packed or unpacked.
    // spec struct Libra {
    //     invariant pack sum_of_coin_values<CoinType> = sum_of_coin_values<CoinType> + value;
    //     invariant unpack sum_of_coin_values<CoinType> = sum_of_coin_values<CoinType> - value;
    // }

    // spec schema SumOfCoinValuesInvariant<CoinType> {
    //     /// The sum of value of coins is consistent with
    //     /// the total_value CurrencyInfo keeps track of.
    //     invariant module !spec_is_currency<CoinType>() ==> sum_of_coin_values<CoinType> == 0;
    //     invariant module spec_is_currency<CoinType>()
    //                 ==> sum_of_coin_values<CoinType>
    //                     == global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).total_value;
    // }

    // spec module {
    //     apply SumOfCoinValuesInvariant<CoinType> to *<CoinType>;
    // }

    /// TODO (dd): It would be great if we could prove that there is never a coin or a set of coins whose
    /// aggregate value exceeds the CoinInfo.total_value.  However, that property involves summations over
    /// all resources and is beyond the capabilities of the specification logic or the prover, currently.

    // TODO: This schema is better to be in Roles, but put here to avoid an unused schema warning from Roles.
    spec schema AbortsIfNotDesignatedDealer {
        account: signer;
        aborts_if !Roles::spec_has_designated_dealer_role_addr(Signer::spec_address_of(account));
    }

    spec module {
        /// The permission "MintCurrency(type)" is granted to TreasuryCompliance [B12].
        apply Roles::AbortsIfNotTreasuryCompliance{account: tc_account} to register_SCS_currency<CoinType>;

        /// The permission "BurnCurrency(type)" is granted to TreasuryCompliance [B13].
        apply Roles::AbortsIfNotTreasuryCompliance{account: tc_account} to register_SCS_currency<CoinType>;

        /// The permission "PreburnCurrency(type)" is granted to DesignatedDealer [B14].
        apply AbortsIfNotDesignatedDealer to publish_preburn_to_account<CoinType>;

        /// The permission "UpdateExchangeRate(type)" is granted to TreasuryCompliance [B15].
        apply Roles::AbortsIfNotTreasuryCompliance{account: tc_account} to update_lbr_exchange_rate<FromCoinType>;
    }

    spec schema TotalValueRemainsSame<CoinType> {
        /// The total amount of currency stays constant. The antecedant excludes register_currency
        /// and register_SCS_currency, because they make start the currency with total_value = 0.
        ensures old(spec_is_currency<CoinType>())
            ==> spec_currency_info<CoinType>().total_value == old(spec_currency_info<CoinType>().total_value);
    }
    spec module {
        /// Only mint and burn functions can change the total amount of currency.
        apply TotalValueRemainsSame<CoinType> to *<CoinType>
            except mint<CoinType>, mint_with_capability<CoinType>,
            burn<CoinType>, burn_with_capability<CoinType>, burn_with_resource_cap<CoinType>,
            burn_now<CoinType>;
    }

}
}
