// A minimized version of the MarketCap verification problem.
address 0x1 {

module TestMarketCap {

    /*
    TODO(refactoring): this test is deactivated until we have ported this (or a similar) feature, or decided to
      drop it in which case the test should be removed.

    spec module {
        pragma verify = true;
    }


    spec module {
        // SPEC: sum of values of all coins.
        global sum_of_coins: num;

        // GLOBAL SPEC: MarketCap has correct value
        invariant global<MarketCap>(0xA550C18).total_value == sum_of_coins;
    }

    // A resource representing the Diem coin
    resource struct T {
        // The value of the coin. May be zero
        value: u64,
    }
    spec struct T {
        // maintain true sum_of_coins
        invariant pack sum_of_coins = sum_of_coins + value;
        invariant unpack sum_of_coins = sum_of_coins - value;
    }

    resource struct MarketCap {
        // The sum of the values of all DiemCoin::T resources in the system
        total_value: u128,
    }

    // Deposit a check.
    // The coin passed in by reference will have a value equal to the sum of the two coins
    // The `check` coin is consumed in the process
    public fun deposit(coin_ref: &mut T, check: T) {
        let T { value } = check;
        coin_ref.value = coin_ref.value + value;
    }
    spec fun deposit {
        aborts_if coin_ref.value + check.value > max_u64();
        ensures coin_ref.value == old(coin_ref.value) + check.value;
    }

     // Deposit a check which violates the MarketCap module invariant.
     public fun deposit_invalid(coin_ref: &mut T, check: T) {
         let T { value } = check;
         coin_ref.value = coin_ref.value + value / 2;
     }
     spec fun deposit_invalid {
         aborts_if coin_ref.value + check.value / 2 > max_u64();
         ensures coin_ref.value == old(coin_ref.value) + check.value / 2;
     }

     */
}

}
