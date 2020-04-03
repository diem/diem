// dep: tests/sources/stdlib/modules/transaction.move
address 0x0:

// global spec ideas
// SPEC TODO: total value global spec
// SPEC TODO: Number of coins global spec.
// SPEC TODO: who/what functions can mint.

module LibraCoin {
    use 0x0::Transaction;

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
    }

    // Return a reference to the MintCapability published under the sender's account. Fails if the
    // sender does not have a MintCapability.
    // Since only the Association account has a mint capability, this will only succeed if it is
    // invoked by  a transaction sent by that account.
    public fun mint_with_default_capability(amount: u64): T acquires MintCapability, MarketCap {
        mint(amount, borrow_global<MintCapability>(Transaction::sender()))
    }

    spec fun mint_with_default_capability {
	aborts_if !exists<MintCapability>(sender());
        aborts_if !exists<MarketCap>(0xA550C18);
        // Over mint-able amount
        aborts_if amount > 1000000000000000;
        // MarketCap overflow
        aborts_if amount + global<MarketCap>(0xA550C18).total_value > max_u128();
        ensures result.value == amount;
    }

    // Mint a new LibraCoin::T worth `value`. The caller must have a reference to a MintCapability.
    // Only the Association account can acquire such a reference, and it can do so only via
    // `borrow_sender_mint_capability`
    public fun mint(value: u64, capability: &MintCapability): T acquires MarketCap {
        // TODO: temporary measure for testnet only: limit minting to 1B Libra at a time.
        // this is to prevent the market cap's total value from hitting u64_max due to excessive
        // minting. This will not be a problem in the production Libra system because coins will
        // be backed with real-world assets, and thus minting will be correspondingly rarer.
        Transaction::assert(value <= 1000000000 * 1000000, 11); // * 1000000 because the unit is microlibra
        // update market cap resource to reflect minting
        let market_cap = borrow_global_mut<MarketCap>(0xA550C18);
        market_cap.total_value = market_cap.total_value + (value as u128);

        T{ value: value }
    }
    spec fun mint {
        aborts_if !exists<MarketCap>(0xA550C18);
        // Over mint-able amount
        aborts_if value > 1000000000000000;
        // MarketCap overflow
        aborts_if value + global<MarketCap>(0xA550C18).total_value > max_u128();
        ensures global<MarketCap>(0xA550C18).total_value == old(global<MarketCap>(0xA550C18).total_value) + value;
        ensures result.value == value;
    }

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize() {
        // Only callable by the Association address
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        move_to_sender<MintCapability>(MintCapability{ });
        move_to_sender<MarketCap>(MarketCap { total_value: 0u128 });
    }
    spec fun initialize {
        aborts_if sender() != 0xA550C18;
        aborts_if exists<MintCapability>(sender());
        aborts_if exists<MarketCap>(sender());
        ensures exists<MintCapability>(sender());
        ensures exists<MarketCap>(sender());
        ensures global<MarketCap>(sender()).total_value == 0;
        // I'm not going to specify _dummy == true because I don't think we care.
    }

    // Return the total value of all Libra in the system
    public fun market_cap(): u128 acquires MarketCap {
        borrow_global<MarketCap>(0xA550C18).total_value
    }
    spec fun market_cap {
        aborts_if !exists<MarketCap>(0xA550C18);
        ensures result == global<MarketCap>(0xA550C18).total_value;
    }

    // Create a new LibraCoin::T with a value of 0
    public fun zero(): T {
        T{value: 0}
    }
    spec fun zero {
        ensures result.value == 0;
    }

    // Public accessor for the value of a coin
    public fun value(coin_ref: &T): u64 {
        coin_ref.value
    }
    spec fun value {
        ensures result == coin_ref.value;
    }

    // Splits the given coin into two and returns them both
    // It leverages `withdraw` for any verifications of the values
    public fun split(coin: T, amount: u64): (T, T) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }
    spec fun split {
        aborts_if coin.value < amount;
        ensures result_2.value == amount && result_1.value == old(coin.value) - amount;
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
    spec fun withdraw {
        aborts_if coin_ref.value < amount;
        ensures coin_ref.value == old(coin_ref.value) - amount;
        ensures result.value == amount;
    }

    // Merges two coins and returns a new coin whose value is equal to the sum of the two inputs
    public fun join(coin1: T, coin2: T): T  {
        deposit(&mut coin1, coin2);
        coin1
    }
    spec fun join {
        aborts_if coin1.value + coin2.value > max_u64();
        ensures result.value == old(coin1.value) + old(coin2.value);
    }

    // "Merges" the two coins
    // The coin passed in by reference will have a value equal to the sum of the two coins
    // The `check` coin is consumed in the process
    public fun deposit(coin_ref: &mut T, check: T) {
        let T { value } = check;
        coin_ref.value= coin_ref.value + value;
    }
    spec fun deposit {
        aborts_if coin_ref.value + check.value > max_u64();
        ensures coin_ref.value == old(coin_ref.value) + old(check.value);
    }

    // Destroy a coin
    // Fails if the value is non-zero
    // The amount of LibraCoin::T in the system is a tightly controlled property,
    // so you cannot "burn" any non-zero amount of LibraCoin::T
    // SPEC TODO: Number of coins reduced by 1.
    public fun destroy_zero(coin: T) {
        let T { value } = coin;
        Transaction::assert(value == 0, 11);
    }
    spec fun destroy_zero {
        aborts_if coin.value != 0;
    }
}
