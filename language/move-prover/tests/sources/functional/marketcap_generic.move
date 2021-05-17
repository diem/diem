// A minimized version of the MarketCap verification problem.
address 0x1 {

module TestMarketCapGeneric {

    /*
    TODO(refactoring): this test is deactivated until we have ported this (or a similar) feature, or decided to
      drop it in which case the test should be removed.

    spec module {
        pragma verify = true;
    }


    spec module {
        // SPEC: sum of values of all coins.
        global sum_of_coins<X>: num;

        fun internal_sum_of_coins_invariant<X>(): bool {
            global<MarketCap>(0xA550C18).total_value == sum_of_coins<X>
        }

        // Make an indirect call here to test whether spec var usage is lifted
        // correctly up the call chain.
        fun sum_of_coins_invariant<X>(): bool {
            internal_sum_of_coins_invariant<X>()
        }
    }

    // A resource representing the Diem coin
    resource struct T<X> {
        // The value of the coin. May be zero
        value: u64,
    }
    spec T {
        // maintain true sum_of_coins
        invariant pack sum_of_coins<X> = sum_of_coins<X> + value;
        invariant unpack sum_of_coins<X> = sum_of_coins<X> - value;
    }

    resource struct MarketCap<X> {
        // The sum of the values of all DiemCoin::T resources in the system
        total_value: u128,
    }

    // Deposit a check.
    // The coin passed in by reference will have a value equal to the sum of the two coins
    // The `check` coin is consumed in the process
    public fun deposit<X>(coin_ref: &mut T<X>, check: T<X>) {
        let T { value } = check;
        coin_ref.value = coin_ref.value + value;
    }
    spec deposit {
        // module invariant
        requires sum_of_coins_invariant<X>();
        ensures sum_of_coins_invariant<X>();

        // function invariant
        aborts_if coin_ref.value + check.value > max_u64();
        ensures coin_ref.value == old(coin_ref.value) + check.value;

    }

     // Deposit a check which violates the MarketCap module invariant.
     public fun deposit_invalid<X>(coin_ref: &mut T<X>, check: T<X>) {
         let T { value } = check;
         coin_ref.value = coin_ref.value + value / 2;
     }
     spec deposit_invalid {
         // module invariant
         requires sum_of_coins_invariant<X>();
         ensures sum_of_coins_invariant<X>();

         // function invariant
         aborts_if coin_ref.value + check.value / 2 > max_u64();
         ensures coin_ref.value == old(coin_ref.value) + check.value / 2;
     }

     */
}

}
