address 0x1 {

/// Functions to initialize, accumulated, and burn transaction fees.

module TransactionFee {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::Coin1::Coin1;
    use 0x1::LBR;
    use 0x1::Libra::{Self, Libra, Preburn};
    use 0x1::Roles;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;

    /// The `TransactionFee` resource holds a preburn resource for each
    /// fiat `CoinType` that can be collected as a transaction fee.
    resource struct TransactionFee<CoinType> {
        balance: Libra<CoinType>,
        preburn: Preburn<CoinType>,
    }

    /// A `TransactionFee` resource is not in the required state
    const ETRANSACTION_FEE: u64 = 0;

    /// Called in genesis. Sets up the needed resources to collect transaction fees from the
    /// `TransactionFee` resource with the TreasuryCompliance account.
    public fun initialize(
        tc_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        Roles::assert_treasury_compliance(tc_account);
        // accept fees in all the currencies
        add_txn_fee_currency<Coin1>(tc_account);
    }
    spec fun initialize {
        include LibraTimestamp::AbortsIfNotGenesis;
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include AddTxnFeeCurrencyAbortsIf<Coin1>;
        ensures is_initialized();
        ensures spec_transaction_fee<Coin1>().balance.value == 0;
    }
    spec schema AddTxnFeeCurrencyAbortsIf<CoinType> {
        include Libra::AbortsIfNoCurrency<CoinType>;
        aborts_if exists<TransactionFee<CoinType>>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS())
            with Errors::ALREADY_PUBLISHED;
    }

    fun is_coin_initialized<CoinType>(): bool {
        exists<TransactionFee<CoinType>>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS())
    }

    fun is_initialized(): bool {
        is_coin_initialized<Coin1>()
    }

    /// Sets ups the needed transaction fee state for a given `CoinType` currency by
    /// (1) configuring `tc_account` to accept `CoinType`
    /// (2) publishing a wrapper of the `Preburn<CoinType>` resource under `tc_account`
    public fun add_txn_fee_currency<CoinType>(tc_account: &signer) {
        Libra::assert_is_currency<CoinType>();
        assert(
            !is_coin_initialized<CoinType>(),
            Errors::already_published(ETRANSACTION_FEE)
        );
        move_to(
            tc_account,
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
            CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(),
        );
        Libra::deposit(&mut fees.balance, coin)
    }

    spec fun pay_fee {
        include LibraTimestamp::AbortsIfNotOperating;
        aborts_if !is_coin_initialized<CoinType>() with Errors::NOT_PUBLISHED;
        let fees = spec_transaction_fee<CoinType>().balance;
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
        let tc_address = CoreAddresses::TREASURY_COMPLIANCE_ADDRESS();
        if (LBR::is_lbr<CoinType>()) {
            // TODO: Once the composition of LBR is determined fill this in to
            // unpack and burn the backing coins of the LBR coin.
            abort Errors::invalid_state(ETRANSACTION_FEE)
        } else {
            // extract fees
            let fees = borrow_global_mut<TransactionFee<CoinType>>(tc_address);
            let coin = Libra::withdraw_all(&mut fees.balance);
            let burn_cap = Libra::remove_burn_capability<CoinType>(tc_account);
            // burn
            Libra::burn_now(
                coin,
                &mut fees.preburn,
                tc_address,
                &burn_cap
            );
            Libra::publish_burn_capability(tc_account, burn_cap);
        }
    }

    spec fun burn_fees {
        /// Must abort if the account does not have the TreasuryCompliance role [[H3]][PERMISSION].
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};

        include LibraTimestamp::AbortsIfNotOperating;
        aborts_if !is_coin_initialized<CoinType>() with Errors::NOT_PUBLISHED;
        include if (LBR::spec_is_lbr<CoinType>()) BurnFeesLBR else BurnFeesNotLBR<CoinType>;

        /// The correct amount of fees is burnt and subtracted from market cap.
        ensures Libra::spec_market_cap<CoinType>()
            == old(Libra::spec_market_cap<CoinType>()) - old(spec_transaction_fee<CoinType>().balance.value);
        /// All the fees is burnt so the balance becomes 0.
        ensures spec_transaction_fee<CoinType>().balance.value == 0;
    }
    /// STUB: To be filled in at a later date once the makeup of the LBR has been determined.
    ///
    /// # Specification of the case where burn type is LBR.
    spec schema BurnFeesLBR {
        tc_account: signer;
        aborts_if true with Errors::INVALID_STATE;
    }
    /// # Specification of the case where burn type is not LBR.
    spec schema BurnFeesNotLBR<CoinType> {
        tc_account: signer;
        /// Must abort if the account does not have BurnCapability [[H3]][PERMISSION].
        include Libra::AbortsIfNoBurnCapability<CoinType>{account: tc_account};

        let fees = spec_transaction_fee<CoinType>();
        include Libra::BurnNowAbortsIf<CoinType>{coin: fees.balance, preburn: fees.preburn};

        /// tc_account retrieves BurnCapability [[H3]][PERMISSION].
        /// BurnCapability is not transferrable [[J3]][PERMISSION].
        ensures exists<Libra::BurnCapability<CoinType>>(Signer::spec_address_of(tc_account));
    }

    spec module {} // Switch documentation context to module level.

    /// # Initialization

    spec module {
        /// If time has started ticking, then `TransactionFee` resources have been initialized.
        invariant [global] LibraTimestamp::is_operating() ==> is_initialized();
    }

    /// # Helper Function

    spec define spec_transaction_fee<CoinType>(): TransactionFee<CoinType> {
        borrow_global<TransactionFee<CoinType>>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS())
    }
}
}
