// dep: ../../move-stdlib/modules/Signer.move
// dep: ../../move-stdlib/modules/Vector.move

// Regression test for a bug in handling generic mutual borrow, as well as parameter types of native functions.

address 0x1 {

module Diem {
    use Std::Signer;
    use Std::Vector;

    // A resource representing a fungible token
    struct T<phantom Token> has key, store {
        // The value of the token. May be zero
        value: u64,
    }

    struct Info<phantom Token> has key {
        total_value: u128,
        preburn_value: u64,
    }

    struct Preburn<phantom Token> has key {
        requests: vector<T<Token>>,
        is_approved: bool,
    }
    spec Preburn {
	    invariant !is_approved ==> len(requests) == 0;
    }

    public fun preburn<Token: store>(
        preburn_ref: &mut Preburn<Token>,
        coin: T<Token>
    ) acquires Info {
        let coin_value = value(&coin);
        Vector::push_back(
            &mut preburn_ref.requests,
            coin
        );
        let market_cap = borrow_global_mut<Info<Token>>(@0xA550C18);
        market_cap.preburn_value = market_cap.preburn_value + coin_value
    }

    public fun preburn_to<Token: store>(account: &signer, coin: T<Token>) acquires Info, Preburn {
        preburn(borrow_global_mut<Preburn<Token>>(Signer::address_of(account)), coin)
    }

    public fun market_cap<Token: store>(): u128 acquires Info {
        borrow_global<Info<Token>>(@0xA550C18).total_value
    }

    public fun preburn_value<Token: store>(): u64 acquires Info {
        borrow_global<Info<Token>>(@0xA550C18).preburn_value
    }

    public fun value<Token: store>(coin_ref: &T<Token>): u64 {
        coin_ref.value
    }

}

}
