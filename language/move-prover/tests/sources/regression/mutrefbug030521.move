module 0x42::Bug {

    struct Diem<phantom CoinType> {
        /// The value of this coin in the base units for `CoinType`
        value: u64
    }


    public fun withdraw<CoinType>(coin: &mut Diem<CoinType>, amount: u64): Diem<CoinType> {
        coin.value = coin.value - amount;
        Diem { value: amount }
    }
    spec withdraw {
        pragma opaque;
        include WithdrawAbortsIf<CoinType>;
        ensures coin.value == old(coin.value) - amount;
        ensures result.value == amount;
    }
    spec schema WithdrawAbortsIf<CoinType> {
        coin: Diem<CoinType>;
        amount: u64;
        aborts_if coin.value < amount;
    }

    public fun split<CoinType>(coin: Diem<CoinType>, amount: u64): (Diem<CoinType>, Diem<CoinType>) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }
    spec split {
        aborts_if coin.value < amount;
        ensures result_1.value == coin.value - amount;
        ensures result_2.value == amount;
    }

}
