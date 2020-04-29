// dep: ../tests/sources/stdlib/modules/transaction.move
// dep: ../tests/sources/stdlib/modules/vector.move

// Regression test for a bug in handling generic mutual borrow, as well as parameter types of native functions.

address 0x0 {

module Libra {
    use 0x0::Transaction;
    use 0x0::Vector;

    // A resource representing a fungible token
    resource struct T<Token> {
        // The value of the token. May be zero
        value: u64,
    }

    resource struct Info<Token> {
        total_value: u128,
        preburn_value: u64,
    }

    resource struct Preburn<Token> {
        requests: vector<T<Token>>,
        is_approved: bool,
    }
    spec struct Preburn {
	    invariant !is_approved ==> len(requests) == 0;
    }

    public fun preburn<Token>(
        preburn_ref: &mut Preburn<Token>,
        coin: T<Token>
    ) acquires Info {
        let coin_value = value(&coin);
        Vector::push_back(
            &mut preburn_ref.requests,
            coin
        );
        let market_cap = borrow_global_mut<Info<Token>>(0xA550C18);
        market_cap.preburn_value = market_cap.preburn_value + coin_value
    }

    public fun preburn_to_sender<Token>(coin: T<Token>) acquires Info, Preburn {
        preburn(borrow_global_mut<Preburn<Token>>(Transaction::sender()), coin)
    }

    public fun market_cap<Token>(): u128 acquires Info {
        borrow_global<Info<Token>>(0xA550C18).total_value
    }

    public fun preburn_value<Token>(): u64 acquires Info {
        borrow_global<Info<Token>>(0xA550C18).preburn_value
    }

    public fun value<Token>(coin_ref: &T<Token>): u64 {
        coin_ref.value
    }

}

}
