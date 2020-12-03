address 0x1 {

/// The `Diem` module describes the concept of a coin in the Diem framework. It introduces the
/// resource `Diem::Diem<CoinType>`, representing a coin of given coin type.
/// The module defines functions operating on coins as well as functionality like
/// minting and burning of coins.
module DiemTest {
    use 0x1::CoreAddresses;
    use 0x1::Event::{Self, EventHandle};
    use 0x1::FixedPoint32::{Self, FixedPoint32};
    use 0x1::RegisteredCurrencies;
    use 0x1::Signer;
    use 0x1::Roles;
    use 0x1::DiemTimestamp;

    resource struct RegisterNewCurrency {}

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
    /// of `CoinType` currency to be burned by the holder of the
    /// and the `0x1::XDX` module (and `CoreAddresses::DIEM_ROOT_ADDRESS()` in testnet).
    resource struct BurnCapability<CoinType> { }

    /// The `CurrencyRegistrationCapability` is a singleton resource
    /// published under the `CoreAddresses::DIEM_ROOT_ADDRESS()` and grants
    /// the capability to the `0x1::Diem` module to add currencies to the
    /// `0x1::RegisteredCurrencies` on-chain config.

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

    /// A `BurnEvent` is emitted every time a non-synthetic[1] Diem coin is
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
    /// the time of registration the `MintCapability<CoinType>` and
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
        /// The scaling factor for the coin (i.e. the amount to multiply by
        /// to get to the human-readable representation for this currency).
        /// e.g. 10^6 for `XUS`
        ///
        /// > TODO(wrwg): should the above be "to divide by"?
        scaling_factor: u64,
        /// The smallest fractional part (number of decimal places) to be
        /// used in the human-readable representation for the currency (e.g.
        /// 10^2 for `XUS` cents)
        fractional_part: u64,
        /// The code symbol for this `CoinType`. ASCII encoded.
        /// e.g. for "XDX" this is x"4C4252". No character limit.
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
        exchange_rate_update_events: EventHandle<ToXDXExchangeRateUpdateEvent>,
    }

    // TODO (dd): It would be great to be able to prove this, but requires more work.
    // This may help prove other useful properties.
    // spec struct CurrencyInfo {
    //     invariant preburn_value <= total_value;
    // }

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
        to_burn: Diem<CoinType>,
    }

    const ENOT_GENESIS: u64 = 0;
    const EINVALID_SINGLETON_ADDRESS: u64 = 1;
    const ENOT_TREASURY_COMPLIANCE: u64 = 2;
    const EMINTING_NOT_ALLOWED: u64 = 3;
    const EIS_SYNTHETIC_CURRENCY: u64 = 4;
    const EAMOUNT_EXCEEDS_COIN_VALUE: u64 = 5;
    const EDESTRUCTION_OF_NONZERO_COIN: u64 = 6;
    const ENOT_A_REGISTERED_CURRENCY: u64 = 7;
    const ENOT_AN_SCS_CURRENCY: u64 = 8;
    const EDOES_NOT_HAVE_DIEM_ROOT_ROLE: u64 = 9;
    const EDOES_NOT_HAVE_TREASURY_COMPLIANCE_ROLE: u64 = 10;

    ///////////////////////////////////////////////////////////////////////////
    // Initialization and granting of privileges
    ///////////////////////////////////////////////////////////////////////////

    /// Grants the `RegisterNewCurrency` privilege to
    /// the calling account as long as it has the correct role (TC).
    /// Aborts if `account` does not have a `RoleId` that corresponds with
    /// the treacury compliance role.
    // public fun grant_privileges(account: &signer) {
    // }

    /// Initialization of the `Diem` module; initializes the set of
    /// registered currencies in the `0x1::RegisteredCurrencies` on-chain
    /// config, and publishes the `CurrencyRegistrationCapability` under the
    /// `CoreAddresses::DIEM_ROOT_ADDRESS()`. This can only be called from genesis.
    public fun initialize(
        config_account: &signer,
    ) {
        assert(DiemTimestamp::is_genesis(), ENOT_GENESIS);
        // Operational constraint
        assert(
            Signer::address_of(config_account) == CoreAddresses::DIEM_ROOT_ADDRESS(),
            EINVALID_SINGLETON_ADDRESS
        );
        RegisteredCurrencies::initialize(config_account);
    }

    /// Publishes the `BurnCapability` `cap` for the `CoinType` currency under `account`. `CoinType`
    /// must be a registered currency type.
    /// The caller must pass a `TreasuryComplianceRole` capability.
    /// TODO (dd): I think there is a multiple signer problem here.
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
    public fun mint<CoinType>(account: &signer, value: u64): Diem<CoinType>
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
    spec fun burn {
        // TODO: There was a timeout (> 40s) for this function in CI. Verification turned off.
        pragma verify=false;
        aborts_if !exists<BurnCapability<CoinType>>(Signer::spec_address_of(account));
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
        cancel_burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<CoinType>>(Signer::address_of(account))
        )
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

        Diem<CoinType> { value }
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
        result: Diem<CoinType>;
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
        coin: Diem<CoinType>,
        preburn: &mut Preburn<CoinType>,
        preburn_address: address,
    ) acquires CurrencyInfo {
        let coin_value = value(&coin);
        // Throw if already occupied
        assert(value(&preburn.to_burn) == 0, 6);
        deposit(&mut preburn.to_burn, coin);
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
        // TODO (dd): Could not figure this out. Maybe there is an overflow in deposit?
        pragma aborts_if_is_partial = true;
        include PreburnAbortsIf<CoinType>;
        aborts_if preburn.to_burn.value != 0;
        include PreburnEnsures<CoinType>;
    }
    spec schema PreburnAbortsIf<CoinType> {
        coin: Diem<CoinType>;
        aborts_if !spec_is_currency<CoinType>();
        aborts_if spec_currency_info<CoinType>().preburn_value + coin.value > max_u64();
    }
    // TODO change - Move prover
    spec schema PreburnEnsures<CoinType> {
        coin: Diem<CoinType>;
        preburn: Preburn<CoinType>;
        // ensures Vector::eq_push_back(preburn.requests, old(preburn.requests), coin);
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
        // TODO (dd): consider adding an assertion here that to_burn <= info.total_value
        // I don't think it can happen, but that may be difficult to prove.
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
        assert(!is_synthetic_currency<CoinType>(), EIS_SYNTHETIC_CURRENCY);
        move_to(account, create_preburn<CoinType>(tc_account))
    }

    ///////////////////////////////////////////////////////////////////////////

    /// Sends `coin` to the preburn queue for `account`, where it will wait to either be burned
    /// or returned to the balance of `account`.
    /// Calls to this function will fail if `account` does not have a
    /// `Preburn<CoinType>` resource published under it.
    public fun preburn_to<CoinType>(
        account: &signer, coin: Diem<CoinType>) acquires CurrencyInfo, Preburn {
        let sender = Signer::address_of(account);
        preburn_with_resource(coin, borrow_global_mut<Preburn<CoinType>>(sender), sender);
    }
    spec fun preburn_to {
        // TODO: Missing aborts_if in preburn_with_resource
        pragma aborts_if_is_partial = true;
        aborts_if !exists<Preburn<CoinType>>(Signer::spec_address_of(account));
        include PreburnAbortsIf<CoinType>;
        include PreburnEnsures<CoinType>{preburn: global<Preburn<CoinType>>(Signer::spec_address_of(account))};
    }

    /// Permanently removes the coins held in the `Preburn` resource (in to_burn field)
    /// stored at `preburn_address` and updates the market cap accordingly.
    /// This function can only be called by the holder of a `BurnCapability<CoinType>`.
    /// Calls to this function will fail if the there is no `Preburn<CoinType>`
    /// resource under `preburn_address`, or, if the preburn to_burn area for
    /// `CoinType` is empty (error code 7).
    public fun burn_with_capability<CoinType>(
        preburn_address: address,
        capability: &BurnCapability<CoinType>
    ) acquires CurrencyInfo, Preburn {
        // destroy the coin in the preburn to_burn area
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
        assert(preburn.to_burn.value > 0, 7);
        // destroy the coin in Preburn area
        let Diem { value } = withdraw_all<CoinType>(&mut preburn.to_burn);
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
        aborts_if preburn.to_burn.value == 0;
        aborts_if spec_currency_info<CoinType>().preburn_value - preburn.to_burn.value < 0;
        aborts_if spec_currency_info<CoinType>().total_value - preburn.to_burn.value < 0;
    }

    spec schema BurnEnsures<CoinType> {
        preburn: Preburn<CoinType>;
        // TODO(moezinia) made changes to preburn area which affect these abort conditions
        // ensures Vector::eq_pop_front(preburn.requests, old(preburn.requests));
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
        let preburn = borrow_global_mut<Preburn<CoinType>>(preburn_address);
        let coin = withdraw_all<CoinType>(&mut preburn.to_burn);
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

    /// Removes and returns the `BurnCapability<CoinType>` from `account`.
    /// Calls to this function will fail if `account` does  not have a
    /// published `BurnCapability<CoinType>` resource at the top-level.
    public fun remove_burn_capability<CoinType>(account: &signer): BurnCapability<CoinType>
    acquires BurnCapability {
        move_from<BurnCapability<CoinType>>(Signer::address_of(account))
    }

    /// Returns the total value of `Diem<CoinType>` that is waiting to be
    /// burned throughout the system (i.e. the sum of all outstanding
    /// preburn requests across all preburn resources for the `CoinType`
    /// currency).
    public fun preburn_value<CoinType>(): u64 acquires CurrencyInfo {
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
        aborts_if coin.value < amount;
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
        assert(coin.value >= amount, EAMOUNT_EXCEEDS_COIN_VALUE);
        coin.value = coin.value - amount;
        Diem { value: amount }
    }
    spec fun withdraw {
        aborts_if coin.value < amount;
        ensures coin.value == old(coin.value) - amount;
        ensures result.value == amount;
    }

    /// Return a `Diem<CoinType>` worth `coin.value` and reduces the `value` of the input `coin` to
    /// zero. Does not abort.
    public fun withdraw_all<CoinType>(coin: &mut Diem<CoinType>): Diem<CoinType> {
        let val = coin.value;
        withdraw(coin, val)
    }
    spec fun withdraw_all {
        aborts_if false;
        ensures result.value == old(coin.value);
        ensures coin.value == 0;
    }

    /// and returns a new coin whose value is equal to the sum of the two inputs.
    public fun join<CoinType>(xus: Diem<CoinType>, coin2: Diem<CoinType>): Diem<CoinType>  {
        deposit(&mut xus, coin2);
        xus
    }
    spec fun join {
        aborts_if xus.value + coin2.value > max_u64();
        ensures result.value == xus.value + coin2.value;
    }


    /// "Merges" the two coins.
    /// The coin passed in by reference will have a value equal to the sum of the two coins
    /// The `check` coin is consumed in the process
    public fun deposit<CoinType>(coin: &mut Diem<CoinType>, check: Diem<CoinType>) {
        let Diem { value } = check;
        coin.value = coin.value + value;
    }

    /// Destroy a zero-value coin. Calls will fail if the `value` in the passed-in `coin` is non-zero
    /// so you cannot "burn" any non-zero amount of `Diem` without having
    /// a `BurnCapability` for the specific `CoinType`.
    public fun destroy_zero<CoinType>(coin: Diem<CoinType>) {
        let Diem { value } = coin;
        assert(value == 0, EDESTRUCTION_OF_NONZERO_COIN)
    }
    spec fun destroy_zero {
        aborts_if coin.value > 0;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Definition of Currencies
    ///////////////////////////////////////////////////////////////////////////

    /// Register the type `CoinType` as a currency. Until the type is
    /// registered as a currency it cannot be used as a coin/currency unit in Diem.
    /// The passed-in `dr_account` must be a specific address (`CoreAddresses::CURRENCY_INFO_ADDRESS()`) and
    /// `dr_account` must also have the correct `RegisterNewCurrency` capability.
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
        assert(
            Signer::address_of(dr_account) == CoreAddresses::CURRENCY_INFO_ADDRESS(),
            EINVALID_SINGLETON_ADDRESS
        );

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
        aborts_if !Roles::spec_has_diem_root_role_addr(Signer::spec_address_of(dr_account));
        aborts_if Signer::spec_address_of(dr_account) != CoreAddresses::CURRENCY_INFO_ADDRESS();
        aborts_if exists<CurrencyInfo<CoinType>>(Signer::spec_address_of(dr_account));
        aborts_if spec_is_currency<CoinType>();
        include RegisteredCurrencies::AddCurrencyCodeAbortsIf;
    }

    /// Registers a stable currency (SCS) coin -- i.e., a non-synthetic currency.
    /// Resources are published on two distinct
    /// accounts: The CoinInfo is published on the Diem root account, and the mint and
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
        // DD: converted to move_to because of problems proving invariant.
        // publish_mint_capability<CoinType>(tc_account, mint_cap, tc_account);
        move_to(tc_account, mint_cap);
        publish_burn_capability<CoinType>(tc_account, burn_cap, tc_account);
    }

    spec fun register_SCS_currency {
        // TODO (dd): I could not figure out what the problem was here.
        pragma aborts_if_is_partial = true;
        ensures spec_has_mint_capability<CoinType>(Signer::spec_address_of(tc_account));
    }

    /// Returns the total amount of currency minted of type `CoinType`.
    public fun market_cap<CoinType>(): u128
    acquires CurrencyInfo {
        borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).total_value
    }

    /// Returns the value of the coin in the `FromCoinType` currency in XDX.
    /// This should only be used where a _rough_ approximation of the exchange
    /// rate is needed.
    public fun approx_xdx_for_value<FromCoinType>(from_value: u64): u64
    acquires CurrencyInfo {
        let xdx_exchange_rate = xdx_exchange_rate<FromCoinType>();
        FixedPoint32::multiply_u64(from_value, xdx_exchange_rate)
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

    /// Updates the `to_xdx_exchange_rate` held in the `CurrencyInfo` for
    /// `FromCoinType` to the new passed-in `xdx_exchange_rate`.
    public fun update_xdx_exchange_rate<FromCoinType>(
        tr_account: &signer,
        xdx_exchange_rate: FixedPoint32
    ) acquires CurrencyInfo {
        assert(Roles::has_treasury_compliance_role(tr_account), ENOT_TREASURY_COMPLIANCE);
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

    /// Returns the (rough) exchange rate between `CoinType` and `XDX`
    public fun xdx_exchange_rate<CoinType>(): FixedPoint32
    acquires CurrencyInfo {
        *&borrow_global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).to_xdx_exchange_rate
    }

    /// There may be situations in which we disallow the further minting of
    /// coins in the system without removing the currency. This function
    /// allows the association TC account to control whether or not further coins of
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

    fun assert_is_SCS_currency<CoinType>() acquires CurrencyInfo {
        assert(is_SCS_currency<CoinType>(), ENOT_AN_SCS_CURRENCY);
    }


    /// **************** MODULE SPECIFICATION ****************
    spec module {} // switch documentation context back to module level

    /// # Module Specification

    spec module {
        /// Verify all functions in this module.
        /// > TODO(wrwg): temporarily deactivated as a recent PR destroyed assumptions
        /// > about coin balance.
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

        /// Specification version of `Self::approx_xdx_for_value`.
        define spec_approx_xdx_for_value<CoinType>(value: num):  num {
            FixedPoint32::spec_multiply_u64(
                value,
                global<CurrencyInfo<CoinType>>(CoreAddresses::CURRENCY_INFO_ADDRESS()).to_xdx_exchange_rate
            )
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
    spec schema MintCapabilitySpecs {
        /// If an address has a mint capability, it is an SCS currency.
        invariant module forall coin_type: type
                             where (exists addr3: address : spec_has_mint_capability<coin_type>(addr3)) :
                                  spec_is_SCS_currency<coin_type>();

        /// If there is a pending offer for a mint capability, the coin_type is an SCS currency and
        /// there are no published Mint Capabilities. (This is the state after register_SCS_currency_start)
        invariant module forall coin_type: type :
                                  spec_is_SCS_currency<coin_type>()
                                  && (forall addr3: address : !spec_has_mint_capability<coin_type>(addr3));

        // At most one address has a mint capability for SCS CoinType
        invariant module forall coin_type: type where spec_is_SCS_currency<coin_type>():
            forall addr1: address, addr2: address
                 where exists<MintCapability<coin_type>>(addr1) && exists<MintCapability<coin_type>>(addr2):
                      addr1 == addr2;

        // Once a MintCapability appears at an address, it stays there.
        ensures forall coin_type: type:
            forall addr1: address where old(exists<MintCapability<coin_type>>(addr1)):
                exists<MintCapability<coin_type>>(addr1);

        // TODO: Only an address with a MintCapability may increase the amount of currency
        // TODO: Only the account managing the currency may mint.  (add manager field to CurrencyInfo?)

        // If address has a mint capability, it has the treasury compliance role
        // TODO: change has_..._role functions to take addresses, not signers.
//        ensures forall addr1: address where exists<MintCapability<CoinType>>(addr1):
//                                        Roles::spec_has_treasury_compliance_role(addr1);
    }

    spec module {
        apply MintCapabilitySpecs to *<T>, *;
    }


    /// ## Conservation of currency

    spec module {
        /// Maintain a spec variable representing the sum of
        /// all coins of a currency type.
        global sum_of_coin_values<CoinType>: num;
    }

    /// Account for updating `sum_of_coin_values` when a `Diem` is packed or unpacked.
    spec struct Diem {
        invariant pack sum_of_coin_values<CoinType> = sum_of_coin_values<CoinType> + value;
        invariant unpack sum_of_coin_values<CoinType> = sum_of_coin_values<CoinType> - value;
    }

    // spec schema TotalValueRemainsSame<CoinType> {
    //     /// The total amount of currency stays constant.
    //     ensures sum_of_coin_values<CoinType> == old(sum_of_coin_values<CoinType>);
    // }

    // spec module {
    //     /// Only mint and burn functions can change the total amount of currency.
    //     apply TotalValueRemainsSame<CoinType> to *<CoinType>
    //         except mint<CoinType>, mint_with_capability<CoinType>,
    //         burn<CoinType>, burn_with_capability<CoinType>, burn_with_resource_cap<CoinType>;
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
