// A minimized version of the MarketCap verification problem.
address 0x1 {

module TestMarketCapWithSchemas {

    /*
    TODO(refactoring): this test is deactivated until we have ported this (or a similar) feature, or decided to
      drop it in which case the test should be removed.

    spec module {
        pragma verify = true;
    }


    spec module {
        global sum_of_coins<X>: num;
    }

    spec schema SumOfCoinsModuleInvariant<X> {
        invariant module global<MarketCap>(0xA550C18).total_value == sum_of_coins<X>;
    }

    // A resource representing the Diem coin
    resource struct T<X> {
        // The value of the coin. May be zero
        value: u64,
    }
    spec T {
        include UpdateSumOfCoins<X>;
    }
    spec schema UpdateSumOfCoins<X> {
        value: num;
        invariant pack sum_of_coins<X> = sum_of_coins<X> + value;
        invariant unpack sum_of_coins<X> = sum_of_coins<X> - value;
    }


    resource struct MarketCap<X> {
        // The sum of the values of all DiemCoin::T resources in the system
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
    spec deposit {
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
     spec deposit_invalid {
         // module invariant
         include SumOfCoinsModuleInvariant<X>;

         // function contract
         aborts_if coin_ref.value + check.value / 2 > max_u64();
         ensures coin_ref.value == old(coin_ref.value) + check.value / 2;
     }

     */
}

}
