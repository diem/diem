address 0x0:

module LibraCoin {
    use 0x0::Transaction;
    use 0x0::Vector;

    // A resource representing the Libra coin
    resource struct T {
        // The value of the coin. May be zero
        value: u64,
    }

    // A singleton resource that grants access to `LibraCoin::mint`. Only the Association has one.
    resource struct MintCapability { }

    resource struct MarketCap {
        // The sum of the values of all LibraCoin::T resources in the system
        total_value: u128,
        // Value of funds that are in the process of being burned
        preburn_value: u64,
    }

    // A holding area where funds that will subsequently be burned wait while their underyling
    // assets are sold off-chain.
    // This resource can only be created by the holder of the MintCapability. An account that
    // contains this address has the authority to initiate a burn request. A burn request can be
    // resolved by the holder of the MintCapability by either (1) burning the funds, or (2)
    // returning the funds to the account that initiated the burn request.
    // This design supports multiple preburn requests in flight at the same time, including multiple
    // burn requests from the same account. However, burn requests initiaing from the same account
    // must be resolved in FIFO order.
    resource struct Preburn {
        // Queue of pending burn requests
        requests: vector<T>,
    }

    // Return `amount` coins.
    // Fails if the sender does not have a published MintCapability.
    public fun mint(amount: u64): T acquires MarketCap, MintCapability {
        mint_with_capability(amount, borrow_global<MintCapability>(Transaction::sender()))
    }

    // Burn the coins currently held in the preburn holding area under `preburn_address`.
    // Fails if the sender does not have a published MintCapability.
    public fun burn(
        preburn_address: address
    ) acquires MarketCap, MintCapability, Preburn {
        burn_with_capability(preburn_address, borrow_global<MintCapability>(Transaction::sender()))
    }

    // Cancel the oldest burn request from `preburn_address`
    // Fails if the sender does not have a published MintCapability.
    public fun cancel_burn(
        preburn_address: address
    ): T acquires MarketCap, MintCapability, Preburn {
        cancel_burn_with_capability(
            preburn_address,
            borrow_global<MintCapability>(Transaction::sender())
        )
    }

    // Create a new Preburn resource.
    // Fails if the sender does not have a published MintCapability.
    public fun new_preburn(): Preburn acquires MintCapability {
        new_preburn_with_capability(borrow_global<MintCapability>(Transaction::sender()))
    }

    // Mint a new LibraCoin::T worth `value`. The caller must have a reference to a MintCapability.
    // Only the Association account can acquire such a reference, and it can do so only via
    // `borrow_sender_mint_capability`
    public fun mint_with_capability(
        value: u64,
        _capability: &MintCapability
    ): T acquires MarketCap {
        // TODO: temporary measure for testnet only: limit minting to 1B Libra at a time.
        // this is to prevent the market cap's total value from hitting u64_max due to excessive
        // minting. This will not be a problem in the production Libra system because coins will
        // be backed with real-world assets, and thus minting will be correspondingly rarer.
        // * 1000000 here because the unit is microlibra
        Transaction::assert(value <= 1000000000 * 1000000, 11);
        // update market cap resource to reflect minting
        let market_cap = borrow_global_mut<MarketCap>(0xA550C18);
        market_cap.total_value = market_cap.total_value + (value as u128);

        T { value }
    }

    // Create a new Preburn resource.
    // Can only be called by the holder of the MintCapability.
    public fun new_preburn_with_capability(_capability: &MintCapability): Preburn {
        Preburn { requests: Vector::empty() }
    }

    // Send coin to the preburn holding area, where it will wait to be burned.
    // Fails if the sender does not have a published Preburn resource
    public fun preburn(coin: T) acquires MarketCap, Preburn {
        let coin_value = value(&coin);
        Vector::push_back(
            &mut borrow_global_mut<Preburn>(Transaction::sender()).requests,
            coin
        );
        let market_cap = borrow_global_mut<MarketCap>(0xA550C18);
        market_cap.preburn_value = market_cap.preburn_value + coin_value
    }

    // Permanently remove the coins held in the `Preburn` resource stored at `preburn_address` and
    // update the market cap accordingly. If there are multiple preburn requests in progress, this
    // will remove the oldest one.
    // Can only be invoked by the holder of the MintCapability. Fails if the there is no `Preburn`
    // resource under `preburn_address` or has one with no pending burn requests.
    public fun burn_with_capability(
        preburn_address: address,
        _capability: &MintCapability
    ) acquires MarketCap, Preburn {
        // destroy the coin at the head of the preburn queue
        let preburn = borrow_global_mut<Preburn>(preburn_address);
        let T { value } = Vector::remove(&mut preburn.requests, 0);
        // update the market cap
        let market_cap = borrow_global_mut<MarketCap>(0xA550C18);
        market_cap.total_value = market_cap.total_value - (value as u128);
        market_cap.preburn_value = market_cap.preburn_value - value
    }

    // Cancel the burn request in the `Preburn` resource stored at `preburn_address` and
    // return the coins to the caller.
    // If there are multiple preburn requests in progress, this will cancel the oldest one.
    // Can only be invoked by the holder of the MintCapability. Fails if the transaction sender
    // does not have a published Preburn resource or has one with no pending burn requests.
    public fun cancel_burn_with_capability(
        preburn_address: address,
        _capability: &MintCapability
    ): T acquires MarketCap, Preburn {
        // destroy the coin at the head of the preburn queue
        let preburn = borrow_global_mut<Preburn>(preburn_address);
        let coin = Vector::remove(&mut preburn.requests, 0);
        // update the market cap
        let market_cap = borrow_global_mut<MarketCap>(0xA550C18);
        market_cap.preburn_value = market_cap.preburn_value - value(&coin);

        coin
    }

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize() {
        // Only callable by the Association address
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        move_to_sender<MintCapability>(MintCapability{ });
        move_to_sender<MarketCap>(MarketCap { total_value: 0u128, preburn_value: 0 });
    }

    // Publish `preburn` under the sender's account
    public fun publish_preburn(preburn: Preburn) {
        move_to_sender(preburn)
    }

    // Publish `capability` under the sender's account
    public fun publish_mint_capability(capability: MintCapability) {
        move_to_sender(capability)
    }

    // Remove and return the MintCapability from the sender's account. Fails if the sender does
    // not have a published MintCapability
    public fun remove_mint_capability(): MintCapability acquires MintCapability {
        move_from<MintCapability>(Transaction::sender())
    }

    // Return the total value of all Libra in the system
    public fun market_cap(): u128 acquires MarketCap {
        borrow_global<MarketCap>(0xA550C18).total_value
    }

    // Return the total value of Libra to be burned
    public fun preburn_value(): u64 acquires MarketCap {
        borrow_global<MarketCap>(0xA550C18).preburn_value
    }

    // Create a new LibraCoin::T with a value of 0
    public fun zero(): T {
        T{value: 0}
    }

    // Public accessor for the value of a coin
    public fun value(coin_ref: &T): u64 {
        coin_ref.value
    }

    // Splits the given coin into two and returns them both
    // It leverages `withdraw` for any verifications of the values
    public fun split(coin: T, amount: u64): (T, T) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }

    // "Divides" the given coin into two, where original coin is modified in place
    // The original coin will have value = original value - `amount`
    // The new coin will have a value = `amount`
    // Fails if the coins value is less than `amount`
    public fun withdraw(coin_ref: &mut T, amount: u64): T {
        // Check that `amount` is less than the coin's value
        Transaction::assert(coin_ref.value >= amount, 10);

        // Split the coin
        coin_ref.value = coin_ref.value - amount;
        T{ value: amount }
    }

    // Merges two coins and returns a new coin whose value is equal to the sum of the two inputs
    public fun join(coin1: T, coin2: T): T  {
        deposit(&mut coin1, coin2);
        coin1
    }

    // "Merges" the two coins
    // The coin passed in by reference will have a value equal to the sum of the two coins
    // The `check` coin is consumed in the process
    public fun deposit(coin_ref: &mut T, check: T) {
        let T { value } = check;
        coin_ref.value= coin_ref.value + value;
    }

    // Destroy a coin
    // Fails if the value is non-zero
    // The amount of LibraCoin::T in the system is a tightly controlled property,
    // so you cannot "burn" any non-zero amount of LibraCoin::T
    public fun destroy_zero(coin: T) {
        let T { value } = coin;
        Transaction::assert(value == 0, 11);
    }
}
