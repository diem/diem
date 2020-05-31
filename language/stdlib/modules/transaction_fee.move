address 0x0 {

module TransactionFee {
    use 0x0::Signer;
    use 0x0::Transaction;
    use 0x0::Coin1::T as Coin1;
    use 0x0::Coin2::T as Coin2;
    use 0x0::LBR::{Self, T as LBR};
    use 0x0::Libra::{Self, T as Libra, Preburn, BurnCapability};
    use 0x0::LibraAccount;

    /// The `TransactionFeeCollection` resource holds the
    /// `LibraAccount::withdraw_with_capability` for the `0xFEE` account.
    /// This is used for the collection of the transaction fees since it
    /// must be sent from the account at the `0xB1E55ED` address.
    resource struct TransactionFeeCollection {
        cap: LibraAccount::WithdrawalCapability,
    }

    /// The `TransactionFeePreburn` holds a preburn resource for each
    /// fiat `CoinType` that can be collected as a transaction fee.
    resource struct TransactionFeePreburn<CoinType> {
        preburn: Preburn<CoinType>
    }

    /// We need to be able to determine if `CoinType` is LBR or not in
    /// order to unpack it properly before burning it. This resource is
    /// instantiated with `LBR` and published in `TransactionFee::initialize`.
    /// We then use this to determine if the / `CoinType` is LBR in `TransactionFee::is_lbr`.
    resource struct LBRIdent<CoinType> { }

    /// Called in genesis. Sets up the needed resources to collect
    /// transaction fees by the `0xB1E55ED` account.
    public fun initialize(blessed_account: &signer, fee_account: &signer) {
        Transaction::assert(Signer::address_of(blessed_account) == 0xB1E55ED, 0);
        let cap = LibraAccount::extract_sender_withdrawal_capability(fee_account);
        move_to(blessed_account, TransactionFeeCollection { cap });
        move_to(blessed_account, LBRIdent<LBR>{})
    }

    /// Sets ups the needed transaction fee state for a given `CoinType`
    /// currency.
    public fun add_txn_fee_currency<CoinType>(fee_account: &signer, burn_cap: &BurnCapability<CoinType>) {
        Transaction::assert(Signer::address_of(fee_account) == 0xFEE, 0);
        LibraAccount::add_currency<CoinType>(fee_account);
        move_to(fee_account, TransactionFeePreburn<CoinType>{
            preburn: Libra::new_preburn_with_capability(burn_cap)
        })
    }

    /// Returns whether `CoinType` is LBR or not. This is needed since we
    /// will need to unpack LBR before burning it when collecting the
    /// transaction fees.
    public fun is_lbr<CoinType>(): bool {
        exists<LBRIdent<CoinType>>(0xB1E55ED)
    }

    /// Preburns the transaction fees collected in the `CoinType` currency.
    /// If the `CoinType` is LBR, it unpacks the coin and preburns the
    /// underlying fiat.
    public fun preburn_fees<CoinType>(blessed_sender: &signer)
    acquires TransactionFeeCollection, TransactionFeePreburn {
        if (is_lbr<CoinType>()) {
            let amount = LibraAccount::balance<LBR>(0xFEE);
            let coins = LibraAccount::withdraw_with_capability<LBR>(
                &borrow_global<TransactionFeeCollection>(Signer::address_of(blessed_sender)).cap,
                amount
            );
            let (coin1, coin2) = LBR::unpack(coins);
            preburn_coin<Coin1>(coin1);
            preburn_coin<Coin2>(coin2)
        } else {
            let amount = LibraAccount::balance<CoinType>(0xFEE);
            let coins = LibraAccount::withdraw_with_capability<CoinType>(
                &borrow_global<TransactionFeeCollection>(Signer::address_of(blessed_sender)).cap,
                amount
            );
            preburn_coin(coins)
        }
    }

    /// Burns the already preburned fees from a previous call to `preburn_fees`.
    public fun burn_fees<CoinType>(blessed_account: &signer, burn_cap: &BurnCapability<CoinType>)
    acquires TransactionFeePreburn {
        Transaction::assert(Signer::address_of(blessed_account) == 0xB1E55ED, 0);
        let preburn = &mut borrow_global_mut<TransactionFeePreburn<CoinType>>(0xFEE).preburn;
        Libra::burn_with_resource_cap(
            preburn,
            0xFEE,
            burn_cap
        )
    }

    fun preburn_coin<CoinType>(coin: Libra<CoinType>)
    acquires TransactionFeePreburn {
        let preburn = &mut borrow_global_mut<TransactionFeePreburn<CoinType>>(0xFEE).preburn;
        Libra::preburn_with_resource(
            coin,
            preburn,
            0xFEE
        );
    }
}
}
