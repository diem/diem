// A minimized version of the MarketCap verification problem.
address 0x0 {

module TestMarketCapWithSchemas {

    spec module {
        pragma verify = true;
    }


    spec module {
        global sum_of_coins<X>: num;
    }

    spec schema SumOfCoinsModuleInvariant<X> {
        invariant module global<MarketCap>(0xA550C18).total_value == sum_of_coins<X>;
    }

    // A resource representing the Libra coin
    resource struct T<X> {
        // The value of the coin. May be zero
        value: u64,
    }
    spec struct T {
        include UpdateSumOfCoins<X>;
    }
    spec schema UpdateSumOfCoins<X> {
        value: num;
        invariant pack sum_of_coins<X> = sum_of_coins<X> + value;
        invariant unpack sum_of_coins<X> = sum_of_coins<X> - value;
        invariant update sum_of_coins<X> = sum_of_coins<X> - old(value) + value;
    }


    resource struct MarketCap<X> {
        // The sum of the values of all LibraCoin::T resources in the system
        total_value: u128,
    }

    public fun deposit<X>(coin_ref: &mut T<X>, check: T<X>) {
        let T { value } = check;
        coin_ref.value = coin_ref.value + value;
    }
    spec schema DepositContract<X> {
        coin_ref: &mut T<X>;
        check: T<X>;
        aborts_if coin_ref.value + check.value > max_u64();
        ensures coin_ref.value == old(coin_ref.value) + check.value;
    }
    spec fun deposit {
        // module invariant
        include SumOfCoinsModuleInvariant<X>;

        // function contract
        include DepositContract<X>;
    }

     // Deposit a check which violates the MarketCap module invariant.
     public fun deposit_invalid<X>(coin_ref: &mut T<X>, check: T<X>) {
         let T { value } = check;
         coin_ref.value = coin_ref.value + value / 2;
     }
     spec fun deposit_invalid {
         // module invariant
         include SumOfCoinsModuleInvariant<X>;

         // function contract
         aborts_if coin_ref.value + check.value / 2 > max_u64();
         ensures coin_ref.value == old(coin_ref.value) + check.value / 2;
     }
}

}
