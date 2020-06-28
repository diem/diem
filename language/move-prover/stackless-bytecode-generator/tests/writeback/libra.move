// dep: ../../stdlib/modules/Signer.move
// dep: ../../stdlib/modules/Vector.move

address 0x1 {

module Libra {
    use 0x1::Signer;
    use 0x1::Vector;

    // A resource representing a fungible token
    resource struct T<Token> {
        // The value of the token. May be zero
        value: u64,
    }

    // A singleton resource that grants access to `Libra::mint`. Only the Association has one.
    resource struct MintCapability<Token> { }

    resource struct Info<Token> {
        // The sum of the values of all Libra::T resources in the system
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
    // burn requests from the same account. However, burn requests from the same account must be
    // resolved in FIFO order.
    resource struct Preburn<Token> {
        // Queue of pending burn requests
        requests: vector<T<Token>>,
        // Boolean that is true if the holder of the MintCapability has approved this account as a
        // preburner
        is_approved: bool,
    }

    public fun register<Token>(association: &signer) {
        // Only callable by the Association address
        assert(Signer::address_of(association) == 0xA550C18, 1);
        move_to(association, MintCapability<Token>{ });
        move_to(association, Info<Token> { total_value: 0u128, preburn_value: 0 });
    }

    fun assert_is_registered<Token>() {
        assert(exists<Info<Token>>(0xA550C18), 12);
    }

    // Return `amount` coins.
    // Fails if the sender does not have a published MintCapability.
    public fun mint<Token>(account: &signer, amount: u64): T<Token> acquires Info, MintCapability {
        mint_with_capability(amount, borrow_global<MintCapability<Token>>(Signer::address_of(account)))
    }

    // Burn the coins currently held in the preburn holding area under `preburn_address`.
    // Fails if the sender does not have a published MintCapability.
    public fun burn<Token>(
        account: &signer,
        preburn_address: address
    ) acquires Info, MintCapability, Preburn {
        burn_with_capability(
            preburn_address,
            borrow_global<MintCapability<Token>>(Signer::address_of(account))
        )
    }

    // Cancel the oldest burn request from `preburn_address`
    // Fails if the sender does not have a published MintCapability.
    public fun cancel_burn<Token>(
        account: &signer,
        preburn_address: address
    ): T<Token> acquires Info, MintCapability, Preburn {
        cancel_burn_with_capability(
            preburn_address,
            borrow_global<MintCapability<Token>>(Signer::address_of(account))
        )
    }

    // Create a new Preburn resource
    public fun new_preburn<Token>(): Preburn<Token> {
        assert_is_registered<Token>();
        Preburn<Token> { requests: Vector::empty(), is_approved: false, }
    }

    // Mint a new Libra::T worth `value`. The caller must have a reference to a MintCapability.
    // Only the Association account can acquire such a reference, and it can do so only via
    // `borrow_sender_mint_capability`
    public fun mint_with_capability<Token>(
        value: u64,
        _capability: &MintCapability<Token>
    ): T<Token> acquires Info {
        assert_is_registered<Token>();
        // TODO: temporary measure for testnet only: limit minting to 1B Libra at a time.
        // this is to prevent the market cap's total value from hitting u64_max due to excessive
        // minting. This will not be a problem in the production Libra system because coins will
        // be backed with real-world assets, and thus minting will be correspondingly rarer.
        // * 1000000 here because the unit is microlibra
        assert(value <= 1000000000 * 1000000, 11);
        // update market cap resource to reflect minting
        let market_cap = borrow_global_mut<Info<Token>>(0xA550C18);
        market_cap.total_value = market_cap.total_value + (value as u128);

        T<Token> { value }
    }

    // Send coin to the preburn holding area `preburn_ref`, where it will wait to be burned.
    public fun preburn<Token>(
        preburn_ref: &mut Preburn<Token>,
        coin: T<Token>
    ) acquires Info {
        // TODO: bring this back once we can automate approvals in testnet
        // assert(preburn_ref.is_approved, 13);
        let coin_value = value(&coin);
        Vector::push_back(
            &mut preburn_ref.requests,
            coin
        );
        let market_cap = borrow_global_mut<Info<Token>>(0xA550C18);
        market_cap.preburn_value = market_cap.preburn_value + coin_value
    }

    // Send coin to the preburn holding area, where it will wait to be burned.
    // Fails if the sender does not have a published Preburn resource
    public fun preburn_to<Token>(account: &signer, coin: T<Token>) acquires Info, Preburn {
        let sender = Signer::address_of(account);
        preburn(borrow_global_mut<Preburn<Token>>(sender), coin)
    }

    // Permanently remove the coins held in the `Preburn` resource stored at `preburn_address` and
    // update the market cap accordingly. If there are multiple preburn requests in progress, this
    // will remove the oldest one.
    // Can only be invoked by the holder of the MintCapability. Fails if the there is no `Preburn`
    // resource under `preburn_address` or has one with no pending burn requests.
    public fun burn_with_capability<Token>(
        preburn_address: address,
        _capability: &MintCapability<Token>
    ) acquires Info, Preburn {
        // destroy the coin at the head of the preburn queue
        let preburn = borrow_global_mut<Preburn<Token>>(preburn_address);
        let T { value } = Vector::remove(&mut preburn.requests, 0);
        // update the market cap
        let market_cap = borrow_global_mut<Info<Token>>(0xA550C18);
        market_cap.total_value = market_cap.total_value - (value as u128);
        market_cap.preburn_value = market_cap.preburn_value - value
    }

    // Cancel the burn request in the `Preburn` resource stored at `preburn_address` and
    // return the coins to the caller.
    // If there are multiple preburn requests in progress, this will cancel the oldest one.
    // Can only be invoked by the holder of the MintCapability. Fails if the transaction sender
    // does not have a published Preburn resource or has one with no pending burn requests.
    public fun cancel_burn_with_capability<Token>(
        preburn_address: address,
        _capability: &MintCapability<Token>
    ): T<Token> acquires Info, Preburn {
        // destroy the coin at the head of the preburn queue
        let preburn = borrow_global_mut<Preburn<Token>>(preburn_address);
        let coin = Vector::remove(&mut preburn.requests, 0);
        // update the market cap
        let market_cap = borrow_global_mut<Info<Token>>(0xA550C18);
        market_cap.preburn_value = market_cap.preburn_value - value(&coin);

        coin
    }

    // Publish `preburn` under the sender's account
    public fun publish_preburn<Token>(account: &signer, preburn: Preburn<Token>) {
        move_to(account, preburn)
    }

    // Remove and return the `Preburn` resource under the sender's account
    public fun remove_preburn<Token>(account: &signer): Preburn<Token> acquires Preburn {
        move_from<Preburn<Token>>(Signer::address_of(account))
    }

    // Destroys the given preburn resource.
    // Aborts if `requests` is non-empty
    public fun destroy_preburn<Token>(preburn: Preburn<Token>) {
        let Preburn { requests, is_approved: _ } = preburn;
        Vector::destroy_empty(requests)
    }

    // Publish `capability` under the sender's account
    public fun publish_mint_capability<Token>(account: &signer, capability: MintCapability<Token>) {
        move_to(account, capability)
    }

    // Remove and return the MintCapability from the sender's account. Fails if the sender does
    // not have a published MintCapability
    public fun remove_mint_capability<Token>(account: &signer): MintCapability<Token> acquires MintCapability {
        move_from<MintCapability<Token>>(Signer::address_of(account))
    }

    // Return the total value of all Libra in the system
    public fun market_cap<Token>(): u128 acquires Info {
        borrow_global<Info<Token>>(0xA550C18).total_value
    }

    // Return the total value of Libra to be burned
    public fun preburn_value<Token>(): u64 acquires Info {
        borrow_global<Info<Token>>(0xA550C18).preburn_value
    }

    // Create a new Libra::T with a value of 0
    public fun zero<Token>(): T<Token> {
        // prevent silly coin types (e.g., Libra<bool>) from being created
        assert_is_registered<Token>();
        T { value: 0 }
    }

    // Public accessor for the value of a coin
    public fun value<Token>(coin_ref: &T<Token>): u64 {
        coin_ref.value
    }

    // Splits the given coin into two and returns them both
    // It leverages `withdraw` for any verifications of the values
    public fun split<Token>(coin: T<Token>, amount: u64): (T<Token>, T<Token>) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }

    // "Divides" the given coin into two, where original coin is modified in place
    // The original coin will have value = original value - `value`
    // The new coin will have a value = `value`
    // Fails if the coins value is less than `value`
    public fun withdraw<Token>(coin_ref: &mut T<Token>, value: u64): T<Token> {
        // Check that `amount` is less than the coin's value
        assert(coin_ref.value >= value, 10);

        // Split the coin
        coin_ref.value = coin_ref.value - value;
        T { value }
    }

    // Merges two coins and returns a new coin whose value is equal to the sum of the two inputs
    public fun join<Token>(coin1: T<Token>, coin2: T<Token>): T<Token>  {
        deposit(&mut coin1, coin2);
        coin1
    }

    // "Merges" the two coins
    // The coin passed in by reference will have a value equal to the sum of the two coins
    // The `check` coin is consumed in the process
    public fun deposit<Token>(coin_ref: &mut T<Token>, check: T<Token>) {
        let T { value } = check;
        coin_ref.value= coin_ref.value + value;
    }

    // Destroy a coin
    // Fails if the value is non-zero
    // The amount of Libra::T in the system is a tightly controlled property,
    // so you cannot "burn" any non-zero amount of Libra::T
    public fun destroy_zero<Token>(coin: T<Token>) {
        let T<Token> { value } = coin;
        assert(value == 0, 11);
    }
}
}
