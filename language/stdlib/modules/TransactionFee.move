address 0x1 {

module TransactionFee {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::LBR::{Self, LBR};
    use 0x1::Libra::{Self, Libra, Preburn};
    use 0x1::Roles;
    use 0x1::LibraTimestamp;

    /// The `TransactionFee` resource holds a preburn resource for each
    /// fiat `CoinType` that can be collected as a transaction fee.
    resource struct TransactionFee<CoinType> {
        balance: Libra<CoinType>,
        preburn: Preburn<CoinType>,
    }

    spec module {
        invariant [global] LibraTimestamp::is_operating() ==> is_initialized();
    }

    const ETRANSACTION_FEE: u64 = 1;

    /// Called in genesis. Sets up the needed resources to collect transaction fees from the
    /// `TransactionFee` resource with the TreasuryCompliance account.
    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        CoreAddresses::assert_libra_root(lr_account);
        Roles::assert_treasury_compliance(tc_account);
        // accept fees in all the currencies
        add_txn_fee_currency<Coin1>(lr_account, tc_account);
        add_txn_fee_currency<Coin2>(lr_account, tc_account);
        add_txn_fee_currency<LBR>(lr_account, tc_account);
    }
    spec fun initialize {
        include LibraTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot{account: lr_account};
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include AddTxnFeeCurrencyAbortsIf<Coin1>;
        include AddTxnFeeCurrencyAbortsIf<Coin2>;
        include AddTxnFeeCurrencyAbortsIf<LBR>;
        ensures is_initialized();
        ensures transaction_fee<LBR>().balance.value == 0;
        ensures transaction_fee<Coin1>().balance.value == 0;
        ensures transaction_fee<Coin2>().balance.value == 0;
    }
    spec schema AddTxnFeeCurrencyAbortsIf<CoinType> {
        include Libra::AbortsIfNoCurrency<CoinType>;
        aborts_if exists<TransactionFee<CoinType>>(CoreAddresses::LIBRA_ROOT_ADDRESS()) with Errors::ALREADY_PUBLISHED;
    }

    fun is_coin_initialized<CoinType>(): bool {
        exists<TransactionFee<CoinType>>(CoreAddresses::LIBRA_ROOT_ADDRESS())
    }

    fun is_initialized(): bool {
        is_coin_initialized<LBR>() && is_coin_initialized<Coin1>() && is_coin_initialized<Coin2>()
    }

    spec define transaction_fee<CoinType>(): TransactionFee<CoinType> {
        borrow_global<TransactionFee<CoinType>>(CoreAddresses::LIBRA_ROOT_ADDRESS())
    }

    /// Sets ups the needed transaction fee state for a given `CoinType` currency by
    /// (1) configuring `lr_account` to accept `CoinType`
    /// (2) publishing a wrapper of the `Preburn<CoinType>` resource under `lr_account`
    fun add_txn_fee_currency<CoinType>(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        Libra::assert_is_currency<CoinType>();
        assert(
            !exists<TransactionFee<CoinType>>(CoreAddresses::LIBRA_ROOT_ADDRESS()),
            Errors::already_published(ETRANSACTION_FEE)
        );
        move_to(
            lr_account,
            TransactionFee<CoinType> {
                balance: Libra::zero(),
                preburn: Libra::create_preburn(tc_account)
            }
        )
    }

    /// Deposit `coin` into the transaction fees bucket
    public fun pay_fee<CoinType>(coin: Libra<CoinType>) acquires TransactionFee {
        LibraTimestamp::assert_operating();
        assert(is_coin_initialized<CoinType>(), Errors::not_published(ETRANSACTION_FEE));
        let fees = borrow_global_mut<TransactionFee<CoinType>>(
            CoreAddresses::LIBRA_ROOT_ADDRESS()
        );
        Libra::deposit(&mut fees.balance, coin)
    }

    spec fun pay_fee {
        include LibraTimestamp::AbortsIfNotOperating;
        aborts_if !is_coin_initialized<CoinType>() with Errors::NOT_PUBLISHED;
        let fees = transaction_fee<CoinType>().balance;
        include Libra::DepositAbortsIf<CoinType>{coin: fees, check: coin};
        ensures fees.value == old(fees.value) + coin.value;
    }

    /// Preburns the transaction fees collected in the `CoinType` currency.
    /// If the `CoinType` is LBR, it unpacks the coin and preburns the
    /// underlying fiat.
    public fun burn_fees<CoinType>(
        tc_account: &signer,
    ) acquires TransactionFee {
        LibraTimestamp::assert_operating();
        Roles::assert_treasury_compliance(tc_account);
        assert(is_coin_initialized<CoinType>(), Errors::not_published(ETRANSACTION_FEE));
        let fee_address =  CoreAddresses::LIBRA_ROOT_ADDRESS();
        let tc_address = CoreAddresses::TREASURY_COMPLIANCE_ADDRESS();
        if (LBR::is_lbr<CoinType>()) {
            // extract fees
            let fees = borrow_global_mut<TransactionFee<LBR>>(fee_address);
            let coins = Libra::withdraw_all<LBR>(&mut fees.balance);
            let (coin1, coin2) = LBR::unpack(coins);
            // burn
            let coin1_burn_cap = Libra::remove_burn_capability<Coin1>(tc_account);
            let coin2_burn_cap = Libra::remove_burn_capability<Coin2>(tc_account);
            Libra::burn_now(
                coin1,
                &mut borrow_global_mut<TransactionFee<Coin1>>(fee_address).preburn,
                tc_address,
                &coin1_burn_cap
            );
            Libra::burn_now(
                coin2,
                &mut borrow_global_mut<TransactionFee<Coin2>>(fee_address).preburn,
                tc_address,
                &coin2_burn_cap
            );
            Libra::publish_burn_capability(tc_account, coin1_burn_cap, tc_account);
            Libra::publish_burn_capability(tc_account, coin2_burn_cap, tc_account);
        } else {
            // extract fees
            let fees = borrow_global_mut<TransactionFee<CoinType>>(fee_address);
            let coin = Libra::withdraw_all(&mut fees.balance);
            let burn_cap = Libra::remove_burn_capability<CoinType>(tc_account);
            // burn
            Libra::burn_now(
                coin,
                &mut fees.preburn,
                tc_address,
                &burn_cap
            );
            Libra::publish_burn_capability(tc_account, burn_cap, tc_account);
        }
    }

    spec fun burn_fees {
        /// > TODO: this times out and likely is also not fully correct yet.
        pragma verify = false;

        include LibraTimestamp::AbortsIfNotOperating;
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        aborts_if !is_coin_initialized<CoinType>() with Errors::NOT_PUBLISHED;
        include if (LBR::spec_is_lbr<CoinType>()) BurnFeesLBR else BurnFeesNotLBR<CoinType>;

        /// The correct amount of fees is burnt and subtracted from market cap.
        ensures Libra::spec_market_cap<CoinType>()
            == old(Libra::spec_market_cap<CoinType>()) - old(transaction_fee<CoinType>().balance.value);
        /// All the fees is burnt so the balance becomes 0.
        ensures transaction_fee<CoinType>().balance.value == 0;
    }
    /// Specification of the case where burn type is LBR.
    spec schema BurnFeesLBR {
        tc_account: signer;
        include Libra::AbortsIfNoBurnCapability<Coin1>{account: tc_account};
        include Libra::AbortsIfNoBurnCapability<Coin2>{account: tc_account};
        let lbr_fees = transaction_fee<LBR>();
        include LBR::UnpackAbortsIf{coin: lbr_fees.balance};
        let coin1_fees = transaction_fee<Coin1>();
        let coin1 = Libra<Coin1>{value: LBR::spec_unpack_coin1(lbr_fees.balance)};
        include Libra::BurnNowAbortsIf<Coin1>{coin: coin1, preburn: coin1_fees.preburn};
        let coin2_fees = transaction_fee<Coin2>();
        let coin2 = Libra<Coin2>{value: LBR::spec_unpack_coin2(lbr_fees.balance)};
        include Libra::BurnNowAbortsIf<Coin2>{coin: coin2, preburn: coin2_fees.preburn};
    }
    /// Specification of the case where burn type is not LBR.
    spec schema BurnFeesNotLBR<CoinType> {
        tc_account: signer;
        include Libra::AbortsIfNoBurnCapability<CoinType>{account: tc_account};
        let fees = transaction_fee<CoinType>();
        include Libra::BurnNowAbortsIf<CoinType>{coin: fees.balance, preburn: fees.preburn};
    }

    spec module {
        pragma verify;
    }
}
}
