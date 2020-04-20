address 0x0 {

module Libra {
    use 0x0::Association;
    use 0x0::Event;
    use 0x0::FixedPoint32;
    use 0x0::LibraConfig;
    use 0x0::RegisteredCurrencies;
    use 0x0::Transaction;
    use 0x0::Vector;

    // The currency has a `CoinType` color that tells us what currency the
    // `value` inside represents.
    resource struct T<CoinType> { value: u64 }

    // A minting capability allows coins of type `CoinType` to be minted
    resource struct MintCapability<CoinType> { }

    // A burn capability allows coins of type `CoinType` to be burned
    resource struct BurnCapability<CoinType> { }

    // A operations capability to allow this module to register currencies
    // with the RegisteredCurrencies on-chain config.
    resource struct CurrencyRegistrationCapability {
        cap: RegisteredCurrencies::RegistrationCapability,
    }

    struct MintEvent {
        // funds added to the system
        amount: u64,
        // UTF-8 encoded symbol for the coin type (e.g., "LBR")
        currency_code: vector<u8>,
    }

    struct BurnEvent {
        // funds removed from the system
        amount: u64,
        // UTF-8 encoded symbol for the coin type (e.g., "LBR")
        currency_code: vector<u8>,
        // address with the Preburn resource that stored the now-burned funds
        preburn_address: address,
    }

    struct PreburnEvent {
        // funds waiting to be removed from the system
        amount: u64,
        // UTF-8 encoded symbol for the coin type (e.g., "LBR")
        currency_code: vector<u8>,
        // address with the Preburn resource that now holds the funds
        preburn_address: address,
    }

    struct CancelBurnEvent {
        // funds returned
        amount: u64,
        // UTF-8 encoded symbol for the coin type (e.g., "LBR")
        currency_code: vector<u8>,
        // address with the Preburn resource that holds the now-returned funds
        preburn_address: address,
    }

    // The information for every supported currency is stored in a resource
    // under the `currency_addr()` address. Unless they are specified
    // otherwise the fields in this resource are immutable.
    resource struct CurrencyInfo<CoinType> {
        // The total value for the currency represented by
        // `CoinType`. Mutable.
        total_value: u128,
        // Value of funds that are in the process of being burned
        preburn_value: u64,
        // The (rough) exchange rate from `CoinType` to LBR.
        to_lbr_exchange_rate: FixedPoint32::T,
        // Holds whether or not this currency is synthetic (contributes to the
        // off-chain reserve) or not. An example of such a currency would be
        // the LBR.
        is_synthetic: bool,
        // The scaling factor for the coin (i.e. the amount to multiply by
        // to get to the human-readable reprentation for this currency). e.g. 10^6 for Coin1
        scaling_factor: u64,
        // The smallest fractional part (number of decimal places) to be
        // used in the human-readable representation for the currency (e.g.
        // 10^2 for Coin1 cents)
        fractional_part: u64,
        // The code symbol for this `CoinType`. UTF-8 encoded.
        // e.g. for "LBR" this is x"4C4252". No character limit.
        currency_code: vector<u8>,
        // We may want to disable the ability to mint further coins of a
        // currency while that currency is still around. Mutable.
        can_mint: bool,
        // event stream for minting
        mint_events: Event::EventHandle<MintEvent>,
        // event stream for burning
        burn_events: Event::EventHandle<BurnEvent>,
        // event stream for preburn requests
        preburn_events: Event::EventHandle<PreburnEvent>,
        // event stream for cancelled preburn requests
        cancel_burn_events: Event::EventHandle<CancelBurnEvent>,
    }

    // A holding area where funds that will subsequently be burned wait while their underyling
    // assets are sold off-chain.
    // This resource can only be created by the holder of the BurnCapability. An account that
    // contains this address has the authority to initiate a burn request. A burn request can be
    // resolved by the holder of the BurnCapability by either (1) burning the funds, or (2)
    // returning the funds to the account that initiated the burn request.
    // This design supports multiple preburn requests in flight at the same time, including multiple
    // burn requests from the same account. However, burn requests from the same account must be
    // resolved in FIFO order.
    resource struct Preburn<Token> {
        // Queue of pending burn requests
        requests: vector<T<Token>>,
        // Boolean that is true if the holder of the BurnCapability has approved this account as a
        // preburner
        is_approved: bool,
    }

    // An association account holding this privilege can add/remove the
    // currencies from the system.
    struct AddCurrency { }

    ///////////////////////////////////////////////////////////////////////////
    // Initialization and granting of privileges
    ///////////////////////////////////////////////////////////////////////////

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize() {
        Association::assert_sender_is_association();
        Transaction::assert(Transaction::sender() == LibraConfig::default_config_address(), 0);
        let cap = RegisteredCurrencies::initialize();
        move_to_sender(CurrencyRegistrationCapability{ cap })
    }

    // Returns a MintCapability for the `CoinType` currency. `CoinType`
    // must be a registered currency type.
    public fun grant_mint_capability<CoinType>(): MintCapability<CoinType> {
        assert_assoc_and_currency<CoinType>();
        MintCapability<CoinType> { }
    }

    // Returns a `BurnCapability` for the `CoinType` currency. `CoinType`
    // must be a registered currency type.
    public fun grant_burn_capability<CoinType>(): BurnCapability<CoinType> {
        assert_assoc_and_currency<CoinType>();
        BurnCapability<CoinType> { }
    }

    public fun grant_burn_capability_for_sender<CoinType>() {
        Transaction::assert(Transaction::sender() == 0xD1E, 0);
        move_to_sender(grant_burn_capability<CoinType>());
    }

    // Return `amount` coins.
    // Fails if the sender does not have a published MintCapability.
    public fun mint<Token>(amount: u64): T<Token> acquires CurrencyInfo, MintCapability {
        mint_with_capability(amount, borrow_global<MintCapability<Token>>(Transaction::sender()))
    }

    // Burn the coins currently held in the preburn holding area under `preburn_address`.
    // Fails if the sender does not have a published `BurnCapability`.
    public fun burn<Token>(
        preburn_address: address
    ) acquires BurnCapability, CurrencyInfo, Preburn {
        burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<Token>>(Transaction::sender())
        )
    }

    // Cancel the oldest burn request from `preburn_address`
    // Fails if the sender does not have a published `BurnCapability`.
    public fun cancel_burn<Token>(
        preburn_address: address
    ): T<Token> acquires BurnCapability, CurrencyInfo, Preburn {
        cancel_burn_with_capability(
            preburn_address,
            borrow_global<BurnCapability<Token>>(Transaction::sender())
        )
    }

    public fun new_preburn<Token>(): Preburn<Token> {
        assert_is_coin<Token>();
        Preburn<Token> { requests: Vector::empty(), is_approved: false, }
    }

    // Mint a new Libra::T worth `value`. The caller must have a reference to a MintCapability.
    // Only the Association account can acquire such a reference, and it can do so only via
    // `borrow_sender_mint_capability`
    public fun mint_with_capability<Token>(
        value: u64,
        _capability: &MintCapability<Token>
    ): T<Token> acquires CurrencyInfo {
        assert_is_coin<Token>();
        // TODO: temporary measure for testnet only: limit minting to 1B Libra at a time.
        // this is to prevent the market cap's total value from hitting u64_max due to excessive
        // minting. This will not be a problem in the production Libra system because coins will
        // be backed with real-world assets, and thus minting will be correspondingly rarer.
        // * 1000000 here because the unit is microlibra
        Transaction::assert(value <= 1000000000 * 1000000, 11);
        let currency_code = currency_code<Token>();
        // update market cap resource to reflect minting
        let info = borrow_global_mut<CurrencyInfo<Token>>(0xA550C18);
        Transaction::assert(info.can_mint, 4);
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

        T<Token> { value }
    }

    // Create a new Preburn resource.
    // Can only be called by the holder of the BurnCapability.
    public fun new_preburn_with_capability<Token>(
        _capability: &BurnCapability<Token>
    ): Preburn<Token> {
        assert_is_coin<Token>();
        Preburn<Token> { requests: Vector::empty(), is_approved: true }
    }

    // Send a coin to the preburn holding area `preburn` that is passed in.
    public fun preburn_with_resource<Token>(
        coin: T<Token>,
        preburn: &mut Preburn<Token>,
        preburn_address: address,
    ) acquires CurrencyInfo {
        let coin_value = value(&coin);
        Vector::push_back(
            &mut preburn.requests,
            coin
        );
        let currency_code = currency_code<Token>();
        let info = borrow_global_mut<CurrencyInfo<Token>>(0xA550C18);
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

    // Send coin to the preburn holding area, where it will wait to be burned.
    // Fails if the sender does not have a published Preburn resource
    public fun preburn_to_sender<Token>(coin: T<Token>) acquires CurrencyInfo, Preburn {
        let sender = Transaction::sender();
        preburn_with_resource(coin, borrow_global_mut<Preburn<Token>>(sender), sender);
    }

    // Permanently remove the coins held in the `Preburn` resource stored at `preburn_address` and
    // update the market cap accordingly. If there are multiple preburn requests in progress, this
    // will remove the oldest one.
    // Can only be invoked by the holder of the `BurnCapability`. Fails if the there is no `Preburn`
    // resource under `preburn_address` or has one with no pending burn requests.
    public fun burn_with_capability<Token>(
        preburn_address: address,
        capability: &BurnCapability<Token>
    ) acquires CurrencyInfo, Preburn {
        // destroy the coin at the head of the preburn queue
        burn_with_resource_cap(
            borrow_global_mut<Preburn<Token>>(preburn_address),
            preburn_address,
            capability
        )
    }

    // Permanently remove the coins held in the passed-in preburn resource
    // and update the market cap accordingly. If there are multiple preburn
    // requests in progress, this will remove the oldest one.
    // Can only be invoked by the holder of the `BurnCapability`. Fails if
    // the `preburn` resource has no pending burn requests.
    public fun burn_with_resource_cap<Token>(
        preburn: &mut Preburn<Token>,
        preburn_address: address,
        _capability: &BurnCapability<Token>
    ) acquires CurrencyInfo {
        // destroy the coin at the head of the preburn queue
        let T { value } = Vector::remove(&mut preburn.requests, 0);
        // update the market cap
        let currency_code = currency_code<Token>();
        let info = borrow_global_mut<CurrencyInfo<Token>>(0xA550C18);
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

    // Cancel the burn request in the `Preburn` resource stored at `preburn_address` and
    // return the coins to the caller.
    // If there are multiple preburn requests in progress, this will cancel the oldest one.
    // Can only be invoked by the holder of the `BurnCapability`. Fails if the transaction sender
    // does not have a published Preburn resource or has one with no pending burn requests.
    public fun cancel_burn_with_capability<Token>(
        preburn_address: address,
        _capability: &BurnCapability<Token>
    ): T<Token> acquires CurrencyInfo, Preburn {
        // destroy the coin at the head of the preburn queue
        let preburn = borrow_global_mut<Preburn<Token>>(preburn_address);
        let coin = Vector::remove(&mut preburn.requests, 0);
        // update the market cap
        let currency_code = currency_code<Token>();
        let info = borrow_global_mut<CurrencyInfo<Token>>(0xA550C18);
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

    // Publish `preburn` under the sender's account
    public fun publish_preburn<Token>(preburn: Preburn<Token>) {
        move_to_sender(preburn)
    }

    // Publish `capability` under the sender's account
    public fun publish_mint_capability<Token>(capability: MintCapability<Token>) {
        move_to_sender(capability)
    }

    // Remove and return the `Preburn` resource under the sender's account
    public fun remove_preburn<Token>(): Preburn<Token> acquires Preburn {
        move_from<Preburn<Token>>(Transaction::sender())
    }

    // Destroys the given preburn resource.
    // Aborts if `requests` is non-empty
    public fun destroy_preburn<Token>(preburn: Preburn<Token>) {
        let Preburn { requests, is_approved: _ } = preburn;
        Vector::destroy_empty(requests)
    }

    // Remove and return the MintCapability from the sender's account. Fails if the sender does
    // not have a published MintCapability
    public fun remove_mint_capability<Token>(): MintCapability<Token> acquires MintCapability {
        move_from<MintCapability<Token>>(Transaction::sender())
    }

    // Remove and return the BurnCapability from the sender's account. Fails if the sender does
    // not have a published BurnCapability
    public fun remove_burn_capability<Token>(): BurnCapability<Token> acquires BurnCapability {
        move_from<BurnCapability<Token>>(Transaction::sender())
    }

    // Return the total value of Libra to be burned
    public fun preburn_value<Token>(): u64 acquires CurrencyInfo {
        borrow_global<CurrencyInfo<Token>>(0xA550C18).preburn_value
    }

    // Create a new Libra::T<CoinType> with a value of 0
    public fun zero<CoinType>(): T<CoinType> {
        assert_is_coin<CoinType>();
        T<CoinType> { value: 0 }
    }

    // Public accessor for the value of a coin
    public fun value<CoinType>(coin: &T<CoinType>): u64 {
        coin.value
    }

    // Splits the given coin into two and returns them both
    // It leverages `Self::withdraw` for any verifications of the values
    public fun split<CoinType>(coin: T<CoinType>, amount: u64): (T<CoinType>, T<CoinType>) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }

    // "Divides" the given coin into two, where the original coin is modified in place
    // The original coin will have value = original value - `amount`
    // The new coin will have a value = `amount`
    // Fails if the coins value is less than `amount`
    public fun withdraw<CoinType>(coin: &mut T<CoinType>, amount: u64): T<CoinType> {
        // Check that `amount` is less than the coin's value
        Transaction::assert(coin.value >= amount, 10);
        coin.value = coin.value - amount;
        T { value: amount }
    }

    // Merges two coins of the same currency and returns a new coin whose
    // value is equal to the sum of the two inputs
    public fun join<CoinType>(coin1: T<CoinType>, coin2: T<CoinType>): T<CoinType>  {
        deposit(&mut coin1, coin2);
        coin1
    }

    // "Merges" the two coins
    // The coin passed in by reference will have a value equal to the sum of the two coins
    // The `check` coin is consumed in the process
    public fun deposit<CoinType>(coin: &mut T<CoinType>, check: T<CoinType>) {
        let T { value } = check;
        coin.value = coin.value + value;
    }

    // Destroy a coin
    // Fails if the value is non-zero
    // The amount of LibraCoin.T in the system is a tightly controlled property,
    // so you cannot "burn" any non-zero amount of LibraCoin.T
    public fun destroy_zero<CoinType>(coin: T<CoinType>) {
        let T { value } = coin;
        Transaction::assert(value == 0, 5)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Definition of Currencies
    ///////////////////////////////////////////////////////////////////////////

    // Register the type `CoinType` as a currency. Without this, a type
    // cannot be used as a coin/currency unit n Libra.
    public fun register_currency<CoinType>(
        to_lbr_exchange_rate: FixedPoint32::T,
        is_synthetic: bool,
        scaling_factor: u64,
        fractional_part: u64,
        currency_code: vector<u8>,
    ) acquires CurrencyRegistrationCapability {
        // And only callable by the designated currency address.
        Transaction::assert(Association::has_privilege<AddCurrency>(Transaction::sender()), 8);

        move_to_sender(MintCapability<CoinType>{});
        move_to_sender(BurnCapability<CoinType>{});
        move_to_sender(CurrencyInfo<CoinType> {
            total_value: 0,
            preburn_value: 0,
            to_lbr_exchange_rate,
            is_synthetic,
            scaling_factor,
            fractional_part,
            currency_code: copy currency_code,
            can_mint: true,
            mint_events: Event::new_event_handle<MintEvent>(),
            burn_events: Event::new_event_handle<BurnEvent>(),
            preburn_events: Event::new_event_handle<PreburnEvent>(),
            cancel_burn_events: Event::new_event_handle<CancelBurnEvent>()
        });
        RegisteredCurrencies::add_currency_code(
            currency_code,
            &borrow_global<CurrencyRegistrationCapability>(LibraConfig::default_config_address()).cap
        )
    }

    // Return the total amount of currency minted of type `CoinType`
    public fun market_cap<CoinType>(): u128
    acquires CurrencyInfo {
        borrow_global<CurrencyInfo<CoinType>>(currency_addr()).total_value
    }

    // Returns the value of the coin in the `FromCoinType` currency in LBR.
    // This should only be used where a _rough_ approximation of the exchange
    // rate is needed.
    public fun approx_lbr_for_value<FromCoinType>(from_value: u64): u64
    acquires CurrencyInfo {
        let lbr_exchange_rate = lbr_exchange_rate<FromCoinType>();
        FixedPoint32::multiply_u64(from_value, lbr_exchange_rate)
    }

    // Returns the value of the coin in the `FromCoinType` currency in LBR.
    // This should only be used where a rough approximation of the exchange
    // rate is needed.
    public fun approx_lbr_for_coin<FromCoinType>(coin: &T<FromCoinType>): u64
    acquires CurrencyInfo {
        let from_value = value(coin);
        approx_lbr_for_value<FromCoinType>(from_value)
    }

    // Return true if the type `CoinType` is a registered currency.
    public fun is_currency<CoinType>(): bool {
        exists<CurrencyInfo<CoinType>>(currency_addr())
    }

    // Predicate on whether `CoinType` is a synthetic currency.
    public fun is_synthetic_currency<CoinType>(): bool
    acquires CurrencyInfo {
        let addr = currency_addr();
        exists<CurrencyInfo<CoinType>>(addr) &&
            borrow_global<CurrencyInfo<CoinType>>(addr).is_synthetic
    }

    // Returns the scaling factor for the `CoinType` currency.
    public fun scaling_factor<CoinType>(): u64
    acquires CurrencyInfo {
        borrow_global<CurrencyInfo<CoinType>>(currency_addr()).scaling_factor
    }

    // Returns the representable fractional part for the `CoinType` currency.
    public fun fractional_part<CoinType>(): u64
    acquires CurrencyInfo {
        borrow_global<CurrencyInfo<CoinType>>(currency_addr()).fractional_part
    }

    // Return the currency code for the registered currency.
    public fun currency_code<CoinType>(): vector<u8>
    acquires CurrencyInfo {
        *&borrow_global<CurrencyInfo<CoinType>>(currency_addr()).currency_code
    }

    // Updates the exchange rate for `FromCoinType` to LBR exchange rate held on chain.
    public fun update_lbr_exchange_rate<FromCoinType>(lbr_exchange_rate: FixedPoint32::T)
    acquires CurrencyInfo {
        assert_assoc_and_currency<FromCoinType>();
        let currency_info = borrow_global_mut<CurrencyInfo<FromCoinType>>(currency_addr());
        currency_info.to_lbr_exchange_rate = lbr_exchange_rate;
    }

    // Return the (rough) exchange rate between `CoinType` and LBR
    public fun lbr_exchange_rate<CoinType>(): FixedPoint32::T
    acquires CurrencyInfo {
        *&borrow_global<CurrencyInfo<CoinType>>(currency_addr()).to_lbr_exchange_rate
    }

    // There may be situations in which we disallow the further minting of
    // coins in the system without removing the currency. This function
    // allows the association to control whether or not further coins of
    // `CoinType` can be minted or not.
    public fun update_minting_ability<CoinType>(can_mint: bool)
    acquires CurrencyInfo {
        assert_assoc_and_currency<CoinType>();
        let currency_info = borrow_global_mut<CurrencyInfo<CoinType>>(Transaction::sender());
        currency_info.can_mint = can_mint;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Helper functions
    ///////////////////////////////////////////////////////////////////////////

    // The (singleton) address under which the currency registration
    // information is published.
    fun currency_addr(): address {
        0xA550C18
    }

    // Assert that the sender is an association account, and that
    // `CoinType` is a regstered currency type.
    fun assert_assoc_and_currency<CoinType>() {
        Association::assert_sender_is_association();
        assert_is_coin<CoinType>();
    }

    // Assert that `CoinType` is a registered currency
    fun assert_is_coin<CoinType>() {
        Transaction::assert(is_currency<CoinType>(), 1);
    }
}

}
