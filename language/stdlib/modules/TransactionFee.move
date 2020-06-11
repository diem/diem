address 0x1 {

module TransactionFee {
    use 0x1::CoreAddresses;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::LBR::{Self, LBR};
    use 0x1::Libra::{Self, Libra, Preburn, BurnCapability};
    use 0x1::LibraAccount;
    use 0x1::Signer;

    /// The `TransactionFeeCollection` resource holds the
    /// `LibraAccount::WithdrawCapability` for the `CoreAddresses::TRANSACTION_FEE_ADDRESS()` account.
    /// This is used for the collection of the transaction fees since it
    /// must be sent from the account at the `CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()` address.
    resource struct TransactionFeeCollection {
        cap: LibraAccount::WithdrawCapability,
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
    /// transaction fees from the `0xFEE` account with the `0xB1E55ED` account.
    public fun initialize(association: &signer, fee_account: &signer, auth_key_prefix: vector<u8>) {
        assert(
            Signer::address_of(association) == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(),
            0
        );
        assert(
            Signer::address_of(fee_account) == CoreAddresses::TRANSACTION_FEE_ADDRESS(),
            0
        );

        LibraAccount::create_testnet_account<LBR>(
            association, Signer::address_of(fee_account), auth_key_prefix
        );
        // accept fees in all the currencies. No need to do this for LBR
        add_txn_fee_currency<Coin1>(association, fee_account);
        add_txn_fee_currency<Coin2>(association, fee_account);

        let cap = LibraAccount::extract_withdraw_capability(fee_account);
        move_to(fee_account, TransactionFeeCollection { cap });
        move_to(fee_account, LBRIdent<LBR>{});
    }

    /// Sets ups the needed transaction fee state for a given `CoinType` currency by
    /// (1) configuring `fee_account` to accept `CoinType`
    /// (2) publishing a wrapper of the `Preburn<CoinType>` resource under `fee_account`
    fun add_txn_fee_currency<CoinType>(association: &signer, fee_account: &signer) {
        LibraAccount::add_currency<CoinType>(fee_account);
        move_to(fee_account, TransactionFeePreburn<CoinType> {
            preburn: Libra::create_preburn(association)
        })
    }

    /// Returns whether `CoinType` is LBR or not. This is needed since we
    /// will need to unpack LBR before burning it when collecting the
    /// transaction fees.
    public fun is_lbr<CoinType>(): bool {
        exists<LBRIdent<CoinType>>(CoreAddresses::TRANSACTION_FEE_ADDRESS())
    }

    /// Preburns the transaction fees collected in the `CoinType` currency.
    /// If the `CoinType` is LBR, it unpacks the coin and preburns the
    /// underlying fiat.
    public fun preburn_fees<CoinType>(blessed_sender: &signer)
    acquires TransactionFeeCollection, TransactionFeePreburn {
        assert(
            Signer::address_of(blessed_sender) == CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(),
            0
        );
        if (is_lbr<CoinType>()) {
            let amount = LibraAccount::balance<LBR>(CoreAddresses::TRANSACTION_FEE_ADDRESS());
            let coins = LibraAccount::withdraw_from<LBR>(
                &borrow_global<TransactionFeeCollection>(0xFEE).cap,
                amount
            );
            let (coin1, coin2) = LBR::unpack(blessed_sender, coins);
            preburn_coin<Coin1>(coin1);
            preburn_coin<Coin2>(coin2)
        } else {
            let amount = LibraAccount::balance<CoinType>(CoreAddresses::TRANSACTION_FEE_ADDRESS());
            let coins = LibraAccount::withdraw_from<CoinType>(
                &borrow_global<TransactionFeeCollection>(0xFEE).cap,
                amount
            );
            preburn_coin(coins)
        }
    }

    /// Burns the already preburned fees from a previous call to `preburn_fees`.
    public fun burn_fees<CoinType>(burn_cap: &BurnCapability<CoinType>)
    acquires TransactionFeePreburn {
        let preburn = &mut borrow_global_mut<TransactionFeePreburn<CoinType>>(
            CoreAddresses::TRANSACTION_FEE_ADDRESS()
        ).preburn;
        Libra::burn_with_resource_cap(
            preburn,
            CoreAddresses::TRANSACTION_FEE_ADDRESS(),
            burn_cap
        )
    }

    fun preburn_coin<CoinType>(coin: Libra<CoinType>)
    acquires TransactionFeePreburn {
        let preburn = &mut borrow_global_mut<TransactionFeePreburn<CoinType>>(
            CoreAddresses::TRANSACTION_FEE_ADDRESS()
        ).preburn;
        Libra::preburn_with_resource(
            coin,
            preburn,
            CoreAddresses::TRANSACTION_FEE_ADDRESS()
        );
    }
}
}
