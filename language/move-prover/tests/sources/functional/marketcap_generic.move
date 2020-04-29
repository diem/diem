// A minimized version of the MarketCap verification problem.
address 0x0 {

module TestMarketCapGeneric {

    spec module {
        // SPEC: sum of values of all coins.
        global sum_of_coins<X>: num;

        define internal_sum_of_coins_invariant<X>(): bool {
            global<MarketCap>(0xA550C18).total_value == sum_of_coins<X>
        }

        // Make an indirect call here to test whether spec var usage is lifted
        // correctly up the call chain.
        define sum_of_coins_invariant<X>(): bool {
            internal_sum_of_coins_invariant<X>()
        }
    }

    // A resource representing the Libra coin
    resource struct T<X> {
        // The value of the coin. May be zero
        value: u64,
    }
    spec struct T {
        // maintain true sum_of_coins
        invariant pack sum_of_coins<X> = sum_of_coins<X> + value;
        invariant unpack sum_of_coins<X> = sum_of_coins<X> - value;
        invariant update sum_of_coins<X> = sum_of_coins<X> - old(value) + value;
    }

    resource struct MarketCap<X> {
        // The sum of the values of all LibraCoin::T resources in the system
        total_value: u128,
    }

    // Deposit a check.
    // The coin passed in by reference will have a value equal to the sum of the two coins
    // The `check` coin is consumed in the process
    public fun deposit<X>(coin_ref: &mut T<X>, check: T<X>) {
        let T { value } = check;
        coin_ref.value = coin_ref.value + value;
    }
    spec fun deposit {
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
     spec fun deposit_invalid {
         // module invariant
         requires sum_of_coins_invariant<X>();
         ensures sum_of_coins_invariant<X>();

         // function invariant
         aborts_if coin_ref.value + check.value / 2 > max_u64();
         ensures coin_ref.value == old(coin_ref.value) + check.value / 2;
     }
}

}
