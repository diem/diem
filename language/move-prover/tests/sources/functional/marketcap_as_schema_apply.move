// A minimized version of the MarketCap verification problem.
address 0x0 {

module TestMarketCapWithSchemas {

    spec module {
        global sum_of_coins<X>: num;
    }

    spec schema SumOfCoinsModuleInvariant<X> {
        invariant module global<MarketCap>(0xA550C18).total_value == sum_of_coins<X>;
    }

    spec module {
        // This is the statement we are actually testing here. Expectation is that the invariant above
        // is woven into any public function with one type parameter except it's name starts with
        // `excepted_`.
        apply SumOfCoinsModuleInvariant<X1> to public *deposit*<X1> except excepted_*;
    }

    // A resource representing the Libra coin
    resource struct T<X> {
        // The value of the coin. May be zero
        value: u64,
    }
    spec struct T {
        invariant pack sum_of_coins<X> = sum_of_coins<X> + value;
        invariant unpack sum_of_coins<X> = sum_of_coins<X> - value;
    }

    resource struct MarketCap<X> {
        // The sum of the values of all LibraCoin::T resources in the system
        total_value: u128,
    }

    // A schema for deposit functions which match the module invariant.
    spec schema DepositCorrect<X> {
        coin_ref: &mut T<X>;
        check: T<X>;
        aborts_if coin_ref.value + check.value > max_u64();
        ensures coin_ref.value == old(coin_ref.value) + check.value;
    }

    // A schema for deposit functions which do NOT match the module invariant.
    spec schema DepositIncorrect<X> {
        coin_ref: &mut T<X>;
        check: T<X>;
        aborts_if coin_ref.value + check.value / 2 > max_u64();
        ensures coin_ref.value == old(coin_ref.value) + check.value / 2;
    }

    // A function which matches the schema apply and which satisfies the invariant.
    public fun deposit<Token>(coin_ref: &mut T<Token>, check: T<Token>) {
        let T { value } = check;
        coin_ref.value = coin_ref.value + value;
    }
    spec fun deposit {
        include DepositCorrect<Token>;
    }

    // A function which matches the schema apply and which does NOT satisfies the invariant.
    public fun deposit_incorrect<Token>(coin_ref: &mut T<Token>, check: T<Token>) {
        let T { value } = check;
        coin_ref.value = coin_ref.value + value / 2;
    }
    spec fun deposit_incorrect {
        include DepositIncorrect<Token>;
    }

    // A function which does NOT match the schema apply and which does NOT satisfies the invariant.
    // It does not match because it is not public.
    fun deposit_not_public<Token>(coin_ref: &mut T<Token>, check: T<Token>) {
        let T { value } = check;
        coin_ref.value = coin_ref.value + value / 2;
    }
    spec fun deposit_not_public {
        include DepositIncorrect<Token>;
    }

    // A function which does NOT match the schema apply and which does NOT satisfies the invariant.
    // It does not match because it's name starts with `excepted_`.
    public fun excepted_deposit<Token>(coin_ref: &mut T<Token>, check: T<Token>) {
        let T { value } = check;
        coin_ref.value = coin_ref.value + value / 2;
    }
    spec fun excepted_deposit {
        include DepositIncorrect<Token>;
    }

    // A function which does NOT match the schema apply and which does NOT satisfies the invariant.
    // It does not match because it has a different type parameter arity.
    public fun deposit_different_type_params<Token, Other>(coin_ref: &mut T<Token>, check: T<Token>) {
        let T { value } = check;
        coin_ref.value = coin_ref.value + value / 2;
    }
    spec fun deposit_different_type_params {
        include DepositIncorrect<Token>;
    }
}

}
