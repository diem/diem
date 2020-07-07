address 0x1 {

/// The `Libra` module describes the concept of a coin in the Libra framework. It introduces the
/// resource `Libra::Libra<CoinType>`, representing a coin of given coin type.
/// The module defines functions operating on coins as well as functionality like
/// minting and burning of coins.
module Libra {
    use 0x1::CoreAddresses;
    use 0x1::Event::{Self, EventHandle};
    use 0x1::FixedPoint32::{Self, FixedPoint32};
    use 0x1::RegisteredCurrencies::{Self, RegistrationCapability};
    use 0x1::Signer;
    use 0x1::Vector;
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
    /// This capability is held only either by the `CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()` account or the
    /// `0x1::LBR` module (and `CoreAddresses::LIBRA_ROOT_ADDRESS()` in testnet).
    ///
    /// > TODO(wrwg): what does it mean that a capability is held by a module? Consider to remove?
    resource struct MintCapability<CoinType> { }

    /// The `BurnCapability` resource defines a capability to allow coins
    /// of `CoinType` currency to be burned by the holder of the
    /// and the `0x1::LBR` module (and `CoreAddresses::LIBRA_ROOT_ADDRESS()` in testnet).
    resource struct BurnCapability<CoinType> { }

    /// The `CurrencyRegistrationCapability` is a singleton resource
    /// published under the `CoreAddresses::LIBRA_ROOT_ADDRESS()` and grants
    /// the capability to the `0x1::Libra` module to add currencies to the
    /// `0x1::RegisteredCurrencies` on-chain config.
    resource struct CurrencyRegistrationCapability {
        /// A capability to allow updating the set of registered currencies on-chain.
        cap: RegistrationCapability,
    }

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
    /// including multiple burn requests from the same account. However, burn requests
    /// (and cancellations) from the same account must be resolved in FIFO order.
    resource struct Preburn<CoinType> {
        /// The queue of pending burn requests
        requests: vector<Libra<CoinType>>,
    }

    const ENOT_GENESIS: u64 = 0;
    const EINVALID_SINGLETON_ADDRESS: u64 = 1;
    const ENOT_TREASURY_COMPLIANCE: u64 = 2;
    const EMINTING_NOT_ALLOWED: u64 = 3;
    const EIS_SYNTHETIC_CURRENCY: u64 = 4;
    const EAMOUNT_EXCEEDS_COIN_VALUE: u64 = 5;
    const EDESTRUCTION_OF_NONZERO_COIN: u64 = 6;
    const EDOES_NOT_HAVE_REGISTRATION_PRIVILEGE: u64 = 7;
    const ENOT_A_REGISTERED_CURRENCY: u64 = 8;

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
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        // Operational constraint
        assert(
            Signer::address_of(config_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(),
            EINVALID_SINGLETON_ADDRESS
        );
        let cap = RegisteredCurrencies::initialize(config_account);
        move_to(config_account, CurrencyRegistrationCapability{ cap })
    }

    /// Publishes the `MintCapability` `cap` for the `CoinType` currency
    /// under `account`. `CoinType`  must be a registered currency type.
    /// The caller must pass a `TreasuryComplianceRole` capability.
    public fun publish_mint_capability<CoinType>(
        publish_account: &signer,
        cap: MintCapability<CoinType>,
        tc_account: &signer,
    ) {
        assert(Roles::has_treasury_compliance_role(tc_account), ENOT_TREASURY_COMPLIANCE);
        assert_is_currency<CoinType>();
        move_to(publish_account, cap)
    }

    /// Publishes the `BurnCapability` `cap` for the `CoinType` currency under `account`. `CoinType`
    /// must be a registered currency type.
    /// The caller must pass a `TreasuryComplianceRole` capability.
    public fun publish_burn_capability<CoinType>(
        account: &signer,
        cap: BurnCapability<CoinType>,
        tc_account: &signer,
    ) {
        assert(Roles::has_treasury_compliance_role(tc_account), ENOT_TREASURY_COMPLIANCE);
        assert_is_currency<CoinType>();
        move_to(account, cap)
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
        burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<CoinType>>(Signer::address_of(account))
        )
    }

    /// Cancels the oldest burn request in the `Preburn` resource held
    /// under the `preburn_address`, and returns the coins.
    /// Calls to this will fail if the sender does not have a published
    /// `BurnCapability<CoinType>`, or if there are no preburn requests
    /// outstanding in the `Preburn` resource under `preburn_address`.
    public fun cancel_burn<CoinType>(
        account: &signer,
        preburn_address: address
    ): Libra<CoinType> acquires BurnCapability, CurrencyInfo, Preburn {
        cancel_burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<CoinType>>(Signer::address_of(account))
        )
    }

    /// Create a new `Preburn` resource, and return it back to the sender.
    /// The `CoinType` must be a registered currency on-chain.
    public fun new_preburn<CoinType>(): Preburn<CoinType> {
        assert_is_currency<CoinType>();
        Preburn<CoinType> { requests: Vector::empty() }
    }

    /// Mint a new `Libra` coin of `CoinType` currency worth `value`. The
    /// caller must have a reference to a `MintCapability<CoinType>`. Only
    /// the Association account or the `0x1::LBR` module can acquire such a
    /// reference.
    public fun mint_with_capability<CoinType>(
        value: u64,
        _capability: &MintCapability<CoinType>
    ): Libra<CoinType> acquires CurrencyInfo {
        assert_is_currency<CoinType>();
        let currency_code = currency_code<CoinType>();
        // update market cap resource to reflect minting
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        assert(info.can_mint, EMINTING_NOT_ALLOWED);
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
        aborts_if !spec_is_currency<CoinType>();
        aborts_if !spec_currency_info<CoinType>().can_mint;
        aborts_if spec_currency_info<CoinType>().total_value + value > max_u128();
    }
    spec schema MintEnsures<CoinType> {
        value: u64;
        result: Libra<CoinType>;
        ensures spec_currency_info<CoinType>().total_value
                    == old(spec_currency_info<CoinType>().total_value) + value;
        ensures result.value == value;
    }

    /// Add the `coin` to the `preburn` queue in the `Preburn` resource
    /// held at the address `preburn_address`. Emits a `PreburnEvent` to
    /// the `preburn_events` event stream in the `CurrencyInfo` for the
    /// `CoinType` passed in. However, if the currency being preburned is
    /// `synthetic` then no `PreburnEvent` event will be emitted.
    public fun preburn_with_resource<CoinType>(
        coin: Libra<CoinType>,
        preburn: &mut Preburn<CoinType>,
        preburn_address: address,
    ) acquires CurrencyInfo {
        let coin_value = value(&coin);
        Vector::push_back(
            &mut preburn.requests,
            coin
        );
        let currency_code = currency_code<CoinType>();
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
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
        include PreburnAbortsIf<CoinType>;
        include PreburnEnsures<CoinType>;
    }
    spec schema PreburnAbortsIf<CoinType> {
        coin: Libra<CoinType>;
        aborts_if !spec_is_currency<CoinType>();
        aborts_if spec_currency_info<CoinType>().preburn_value + coin.value > max_u64();
    }
    spec schema PreburnEnsures<CoinType> {
        coin: Libra<CoinType>;
        preburn: Preburn<CoinType>;
        ensures Vector::eq_push_back(preburn.requests, old(preburn.requests), coin);
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
        assert(Roles::has_treasury_compliance_role(tc_account), ENOT_TREASURY_COMPLIANCE);
        assert_is_currency<CoinType>();
        Preburn<CoinType> { requests: Vector::empty() }
    }

    /// Publishes a `Preburn` resource under `account`. This function is
    /// used for bootstrapping the designated dealer at account-creation
    /// time, and the association TC account `creator` (at `CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()`) is creating
    /// this resource for the designated dealer.
    public fun publish_preburn_to_account<CoinType>(
        account: &signer,
        tc_account: &signer
    ) acquires CurrencyInfo {
        assert(!is_synthetic_currency<CoinType>(), EIS_SYNTHETIC_CURRENCY);
        move_to(account, create_preburn<CoinType>(tc_account))
    }

    ///////////////////////////////////////////////////////////////////////////

    /// Sends `coin` to the preburn queue for `account`, where it will wait to either be burned
    /// or returned to the balance of `account`.
    /// Calls to this function will fail if `account` does not have a
    /// `Preburn<CoinType>` resource published under it.
    public fun preburn_to<CoinType>(
        account: &signer, coin: Libra<CoinType>) acquires CurrencyInfo, Preburn {
        let sender = Signer::address_of(account);
        preburn_with_resource(coin, borrow_global_mut<Preburn<CoinType>>(sender), sender);
    }
    spec fun preburn_to {
        aborts_if !exists<Preburn<CoinType>>(Signer::spec_address_of(account));
        include PreburnAbortsIf<CoinType>;
        include PreburnEnsures<CoinType>{preburn: global<Preburn<CoinType>>(Signer::spec_address_of(account))};
    }

    /// Permanently removes the coins held in the `Preburn` resource stored at `preburn_address` and
    /// updates the market cap accordingly. If there are multiple preburn
    /// requests in progress (i.e. in the preburn queue), this will remove the oldest one.
    /// This function can only be called by the holder of a `BurnCapability<CoinType>`.
    /// Calls to this function will fail if the there is no `Preburn<CoinType>`
    /// resource under `preburn_address`, or, if the preburn queue for
    /// `CoinType` has no pending burn requests.
    public fun burn_with_capability<CoinType>(
        preburn_address: address,
        capability: &BurnCapability<CoinType>
    ) acquires CurrencyInfo, Preburn {
        // destroy the coin at the head of the preburn queue
        burn_with_resource_cap(
            borrow_global_mut<Preburn<CoinType>>(preburn_address),
            preburn_address,
            capability
        )
    }
    spec fun burn_with_capability {
        aborts_if !exists<Preburn<CoinType>>(preburn_address);
        include BurnAbortsIf<CoinType>{preburn: global<Preburn<CoinType>>(preburn_address)};
        include BurnEnsures<CoinType>{preburn: global<Preburn<CoinType>>(preburn_address)};
    }

    /// Permanently removes the coins held in the `Preburn` resource `preburn` stored at `preburn_address` and
    /// updates the market cap accordingly. If there are multiple preburn
    /// requests in progress (i.e. in the preburn queue), this will remove the oldest one.
    /// This function can only be called by the holder of a `BurnCapability<CoinType>`.
    /// Calls to this function will fail if the there is no `Preburn<CoinType>`
    /// resource under `preburn_address`, or, if the preburn queue for
    /// `CoinType` has no pending burn requests.
    public fun burn_with_resource_cap<CoinType>(
        preburn: &mut Preburn<CoinType>,
        preburn_address: address,
        _capability: &BurnCapability<CoinType>
    ) acquires CurrencyInfo {
        let currency_code = currency_code<CoinType>();
        // destroy the coin at the head of the preburn queue
        let Libra { value } = Vector::remove(&mut preburn.requests, 0);
        // update the market cap
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        info.total_value = info.total_value - (value as u128);
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
        aborts_if !spec_is_currency<CoinType>();
        aborts_if len(preburn.requests) == 0;
        aborts_if {
            let i = spec_currency_info<CoinType>();
            i.total_value < preburn.requests[0].value || i.preburn_value < preburn.requests[0].value
        };
    }
    spec schema BurnEnsures<CoinType> {
        preburn: Preburn<CoinType>;
        ensures Vector::eq_pop_front(preburn.requests, old(preburn.requests));
        ensures spec_currency_info<CoinType>().total_value
                == old(spec_currency_info<CoinType>().total_value) - old(preburn.requests[0].value);
        ensures spec_currency_info<CoinType>().preburn_value
                == old(spec_currency_info<CoinType>().preburn_value) - old(preburn.requests[0].value);
    }

    /// Cancels the burn request in the `Preburn` resource stored at `preburn_address` and
    /// return the coins to the caller.
    /// If there are multiple preburn requests in progress for `CoinType` (i.e. in the
    /// preburn queue), this will cancel the oldest one.
    /// This function can only be called by the holder of a
    /// `BurnCapability<CoinType>`, and will fail if the `Preburn<CoinType>` resource
    /// at `preburn_address` does not contain any pending burn requests.
    public fun cancel_burn_with_capability<CoinType>(
        preburn_address: address,
        _capability: &BurnCapability<CoinType>
    ): Libra<CoinType> acquires CurrencyInfo, Preburn {
        // destroy the coin at the head of the preburn queue
        let preburn = borrow_global_mut<Preburn<CoinType>>(preburn_address);
        let coin = Vector::remove(&mut preburn.requests, 0);
        // update the market cap
        let currency_code = currency_code<CoinType>();
        let info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        let amount = value(&coin);
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

    /// Removes and returns the `MintCapability<CoinType>` from `account`.
    /// Calls to this function will fail if `account` does  not have a
    /// published `MintCapability<CoinType>` resource at the top-level.
    public fun remove_mint_capability<CoinType>(account: &signer): MintCapability<CoinType>
    acquires MintCapability {
        move_from<MintCapability<CoinType>>(Signer::address_of(account))
    }

    /// Removes and returns the `BurnCapability<CoinType>` from `account`.
    /// Calls to this function will fail if `account` does  not have a
    /// published `BurnCapability<CoinType>` resource at the top-level.
    public fun remove_burn_capability<CoinType>(account: &signer): BurnCapability<CoinType>
    acquires BurnCapability {
        move_from<BurnCapability<CoinType>>(Signer::address_of(account))
    }

    /// Returns the total value of `Libra<CoinType>` that is waiting to be
    /// burned throughout the system (i.e. the sum of all outstanding
    /// preburn requests across all preburn resources for the `CoinType`
    /// currency).
    public fun preburn_value<CoinType>(): u64 acquires CurrencyInfo {
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
        aborts_if coin.value < amount;
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
        assert(coin.value >= amount, EAMOUNT_EXCEEDS_COIN_VALUE);
        coin.value = coin.value - amount;
        Libra { value: amount }
    }
    spec fun withdraw {
        aborts_if coin.value < amount;
        ensures coin.value == old(coin.value) - amount;
        ensures result.value == amount;
    }

    /// Return a `Libra<CoinType>` worth `coin.value` and reduces the `value` of the input `coin` to
    /// zero. Does not abort.
    public fun withdraw_all<CoinType>(coin: &mut Libra<CoinType>): Libra<CoinType> {
        let val = coin.value;
        withdraw(coin, val)
    }
    spec fun withdraw_all {
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
        aborts_if coin1.value + coin2.value > max_u64();
        ensures result.value == coin1.value + coin2.value;
    }


    /// "Merges" the two coins.
    /// The coin passed in by reference will have a value equal to the sum of the two coins
    /// The `check` coin is consumed in the process
    public fun deposit<CoinType>(coin: &mut Libra<CoinType>, check: Libra<CoinType>) {
        let Libra { value } = check;
        coin.value = coin.value + value;
    }

    /// Destroy a zero-value coin. Calls will fail if the `value` in the passed-in `coin` is non-zero
    /// so you cannot "burn" any non-zero amount of `Libra` without having
    /// a `BurnCapability` for the specific `CoinType`.
    public fun destroy_zero<CoinType>(coin: Libra<CoinType>) {
        let Libra { value } = coin;
        assert(value == 0, EDESTRUCTION_OF_NONZERO_COIN)
    }
    spec fun destroy_zero {
        aborts_if coin.value > 0;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Definition of Currencies
    ///////////////////////////////////////////////////////////////////////////

    /// Register the type `CoinType` as a currency. Until the type is
    /// registered as a currency it cannot be used as a coin/currency unit in Libra.
    /// The passed-in `account` must be a specific address (`CoreAddresses::CURRENCY_INFO_ADDRESS()`) and
    /// the `account` must also have the correct `RegisterNewCurrency` capability.
    /// After the first registration of `CoinType` as a
    /// currency, all subsequent tries to register `CoinType` as a currency
    /// will fail.
    /// When the `CoinType` is registered it publishes the
    /// `CurrencyInfo<CoinType>` resource under the `CoreAddresses::CURRENCY_INFO_ADDRESS()` and
    /// adds the currency to the set of `RegisteredCurrencies`. It returns
    /// `MintCapability<CoinType>` and `BurnCapability<CoinType>` resources.
    public fun register_currency<CoinType>(
        account: &signer,
        tc_account: &signer,
        to_lbr_exchange_rate: FixedPoint32,
        is_synthetic: bool,
        scaling_factor: u64,
        fractional_part: u64,
        currency_code: vector<u8>,
    ): (MintCapability<CoinType>, BurnCapability<CoinType>)
    acquires CurrencyRegistrationCapability {
        assert(Roles::has_register_new_currency_privilege(tc_account), EDOES_NOT_HAVE_REGISTRATION_PRIVILEGE);
        // Operational constraint that it must be stored under a specific address.
        assert(
            Signer::address_of(account) == CoreAddresses::CURRENCY_INFO_ADDRESS(),
            EINVALID_SINGLETON_ADDRESS
        );

        move_to(account, CurrencyInfo<CoinType> {
            total_value: 0,
            preburn_value: 0,
            to_lbr_exchange_rate,
            is_synthetic,
            scaling_factor,
            fractional_part,
            currency_code: copy currency_code,
            can_mint: true,
            mint_events: Event::new_event_handle<MintEvent>(account),
            burn_events: Event::new_event_handle<BurnEvent>(account),
            preburn_events: Event::new_event_handle<PreburnEvent>(account),
            cancel_burn_events: Event::new_event_handle<CancelBurnEvent>(account),
            exchange_rate_update_events: Event::new_event_handle<ToLBRExchangeRateUpdateEvent>(account)
        });
        RegisteredCurrencies::add_currency_code(
            currency_code,
            &borrow_global<CurrencyRegistrationCapability>(CoreAddresses::LIBRA_ROOT_ADDRESS()).cap
        );
        (MintCapability<CoinType>{}, BurnCapability<CoinType>{})
    }
    spec fun register_currency {
        aborts_if !Roles::spec_has_register_new_currency_privilege(tc_account);
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_CURRENCY_INFO_ADDRESS();
        aborts_if !exists<CurrencyRegistrationCapability>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS());
        aborts_if exists<CurrencyInfo<CoinType>>(Signer::spec_address_of(account));
        aborts_if spec_is_currency<CoinType>();
        include RegisteredCurrencies::AddCurrencyCodeAbortsIf;

        ensures spec_is_currency<CoinType>();
    }

    /// Returns the total amount of currency minted of type `CoinType`.
    public fun market_cap<CoinType>(): u128
    acquires CurrencyInfo {
        borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).total_value
    }

    /// Returns the value of the coin in the `FromCoinType` currency in LBR.
    /// This should only be used where a _rough_ approximation of the exchange
    /// rate is needed.
    public fun approx_lbr_for_value<FromCoinType>(from_value: u64): u64
    acquires CurrencyInfo {
        let lbr_exchange_rate = lbr_exchange_rate<FromCoinType>();
        FixedPoint32::multiply_u64(from_value, lbr_exchange_rate)
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
        borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).scaling_factor
    }

    /// Returns the representable (i.e. real-world) fractional part for the
    /// `CoinType` currency as defined in its `CurrencyInfo`.
    public fun fractional_part<CoinType>(): u64
    acquires CurrencyInfo {
        borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).fractional_part
    }

    /// Returns the currency code for the registered currency as defined in
    /// its `CurrencyInfo` resource.
    public fun currency_code<CoinType>(): vector<u8>
    acquires CurrencyInfo {
        *&borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).currency_code
    }

    /// Updates the `to_lbr_exchange_rate` held in the `CurrencyInfo` for
    /// `FromCoinType` to the new passed-in `lbr_exchange_rate`.
    public fun update_lbr_exchange_rate<FromCoinType>(
        tr_account: &signer,
        lbr_exchange_rate: FixedPoint32
    ) acquires CurrencyInfo {
        assert(Roles::has_treasury_compliance_role(tr_account), ENOT_TREASURY_COMPLIANCE);
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

    /// Returns the (rough) exchange rate between `CoinType` and `LBR`
    public fun lbr_exchange_rate<CoinType>(): FixedPoint32
    acquires CurrencyInfo {
        *&borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).to_lbr_exchange_rate
    }

    /// There may be situations in which we disallow the further minting of
    /// coins in the system without removing the currency. This function
    /// allows the association to control whether or not further coins of
    /// `CoinType` can be minted or not. If this is called with `can_mint =
    /// true`, then minting is allowed, if `can_mint = false` then minting is
    /// disallowed until it is turned back on via this function. All coins
    /// start out in the default state of `can_mint = true`.
    public fun update_minting_ability<CoinType>(
        tr_account: &signer,
        can_mint: bool,
        )
    acquires CurrencyInfo {
        assert(Roles::has_treasury_compliance_role(tr_account), ENOT_TREASURY_COMPLIANCE);
        assert_is_currency<CoinType>();
        let currency_info = borrow_global_mut<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS());
        currency_info.can_mint = can_mint;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Helper functions
    ///////////////////////////////////////////////////////////////////////////

    /// Asserts that `CoinType` is a registered currency.
    fun assert_is_currency<CoinType>() {
        assert(is_currency<CoinType>(), ENOT_A_REGISTERED_CURRENCY);
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
            exists<CurrencyInfo<CoinType>>(CoreAddresses::SPEC_CURRENCY_INFO_ADDRESS())
        }

        /// Returns currency information.
        define spec_currency_info<CoinType>(): CurrencyInfo<CoinType> {
            global<CurrencyInfo<CoinType>>(CoreAddresses::SPEC_CURRENCY_INFO_ADDRESS())
        }

        /// Specification version of `Self::approx_lbr_for_value`.
        define spec_approx_lbr_for_value<CoinType>(value: num):  num {
            FixedPoint32::spec_multiply_u64(
                value,
                global<CurrencyInfo<CoinType>>(CoreAddresses::SPEC_CURRENCY_INFO_ADDRESS()).to_lbr_exchange_rate
            )
        }
    }

    /// ## Conservation of currency

    spec module {
        /// Maintain a spec variable representing the sum of
        /// all coins of a currency type.
        global sum_of_coin_values<CoinType>: num;
    }

    /// Account for updating `sum_of_coin_values` when a `Libra` is packed or unpacked.
    spec struct Libra {
        invariant pack sum_of_coin_values<CoinType> = sum_of_coin_values<CoinType> + value;
        invariant unpack sum_of_coin_values<CoinType> = sum_of_coin_values<CoinType> - value;
    }

    spec schema TotalValueRemainsSame<CoinType> {
        /// The total amount of currency stays constant.
        ensures sum_of_coin_values<CoinType> == old(sum_of_coin_values<CoinType>);
    }

    spec module {
        /// Only mint and burn functions can change the total amount of currency.
        apply TotalValueRemainsSame<CoinType> to *<CoinType>
            except mint<CoinType>, mint_with_capability<CoinType>,
            burn<CoinType>, burn_with_capability<CoinType>, burn_with_resource_cap<CoinType>;
    }

    spec schema SumOfCoinValuesInvariant<CoinType> {
        /// The sum of value of coins is consistent with
        /// the total_value CurrencyInfo keeps track of.
        invariant module !spec_is_currency<CoinType>() ==> sum_of_coin_values<CoinType> == 0;
        invariant module spec_is_currency<CoinType>()
                    ==> sum_of_coin_values<CoinType>
                        == global<CurrencyInfo<CoinType>>(CoreAddresses::SPEC_CURRENCY_INFO_ADDRESS()).total_value;
    }

    spec module {
        apply SumOfCoinValuesInvariant<CoinType> to *<CoinType>;
    }

    /*
    TODO: specify the following:

          sum of coin values in preburn + sum of account balances == total value

          This is false right now because in `cancel_burn`, we return the coin
          directly instead of depositing it into an account. However, even after
          this has been fixed, we will likely need to coordinate with the account
          module and maybe add a spec variable representing the sum of account
          balances.
    */
}
}
