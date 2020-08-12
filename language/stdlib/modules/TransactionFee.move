address 0x1 {

module TransactionFee {
    use 0x1::CoreAddresses;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::LBR::{Self, LBR};
    use 0x1::Libra::{Self, Libra, Preburn, BurnCapability};
    use 0x1::Signer;
    use 0x1::Roles;
    use 0x1::LibraTimestamp;

    /// The `TransactionFee` resource holds a preburn resource for each
    /// fiat `CoinType` that can be collected as a transaction fee.
    resource struct TransactionFee<CoinType> {
        balance: Libra<CoinType>,
        preburn: Preburn<CoinType>,
    }

    const ENOT_GENESIS: u64 = 0;
    const ENOT_TREASURY_COMPLIANCE: u64 = 1;
    const EINVALID_SINGLETON_ADDRESS: u64 = 2;

    /// Called in genesis. Sets up the needed resources to collect transaction fees from the
    /// `TransactionFee` resource with the TreasuryCompliance account.
    public fun initialize(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(
            Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(),
            EINVALID_SINGLETON_ADDRESS
        );
        assert(Roles::has_treasury_compliance_role(tc_account), ENOT_TREASURY_COMPLIANCE);
        // accept fees in all the currencies
        add_txn_fee_currency<Coin1>(lr_account, tc_account);
        add_txn_fee_currency<Coin2>(lr_account, tc_account);
        add_txn_fee_currency<LBR>(lr_account, tc_account);
    }

    spec fun initialize {
        aborts_if !LibraTimestamp::spec_is_genesis();
        aborts_if Signer::spec_address_of(lr_account) != CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS();
        aborts_if !Roles::spec_has_treasury_compliance_role_addr(Signer::spec_address_of(tc_account));
        ensures spec_is_initialized<LBR>();
        ensures spec_is_initialized<Coin1>();
        ensures spec_is_initialized<Coin2>();
        ensures spec_txn_fee_balance<LBR>() == 0;
        ensures spec_txn_fee_balance<Coin1>() == 0;
        ensures spec_txn_fee_balance<Coin2>() == 0;
    }

    spec module {
        /// Returns true if the TransactionFee resource for CoinType has been
        /// initialized.
        define spec_is_initialized<CoinType>(): bool {
            exists<TransactionFee<CoinType>>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS())
        }

        /// Returns the transaction fee balance for CoinType.
        define spec_txn_fee_balance<CoinType>(): u64 {
            global<TransactionFee<CoinType>>(
                CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS()
            ).balance.value
        }
    }
    /// Sets ups the needed transaction fee state for a given `CoinType` currency by
    /// (1) configuring `lr_account` to accept `CoinType`
    /// (2) publishing a wrapper of the `Preburn<CoinType>` resource under `lr_account`
    fun add_txn_fee_currency<CoinType>(
        lr_account: &signer,
        tc_account: &signer,
    ) {
        move_to(
            lr_account,
            TransactionFee<CoinType> {
                balance: Libra::zero(),
                preburn: Libra::create_preburn(tc_account)
            }
        )
    }

    spec fun add_txn_fee_currency {
        pragma assume_no_abort_from_here = true;
        aborts_if exists<TransactionFee<CoinType>>(Signer::spec_address_of(lr_account));
        aborts_if !Roles::spec_has_treasury_compliance_role_addr(Signer::spec_address_of(tc_account));
        aborts_if !Libra::spec_is_currency<CoinType>();
        ensures exists<TransactionFee<CoinType>>(Signer::spec_address_of(lr_account));
        ensures global<TransactionFee<CoinType>>(
            Signer::spec_address_of(lr_account)).balance.value == 0;
    }

    /// Deposit `coin` into the transaction fees bucket
    public fun pay_fee<CoinType>(coin: Libra<CoinType>) acquires TransactionFee {
        let fees = borrow_global_mut<TransactionFee<CoinType>>(
            CoreAddresses::LIBRA_ROOT_ADDRESS()
        );
        Libra::deposit(&mut fees.balance, coin)
    }

    spec fun pay_fee {
        aborts_if !spec_is_initialized<CoinType>();
        aborts_if spec_txn_fee_balance<CoinType>() + coin.value > max_u64();
        ensures spec_txn_fee_balance<CoinType>() == old(spec_txn_fee_balance<CoinType>()) + coin.value;

    }

    /// Preburns the transaction fees collected in the `CoinType` currency.
    /// If the `CoinType` is LBR, it unpacks the coin and preburns the
    /// underlying fiat.
    public fun burn_fees<CoinType>(
        tc_account: &signer,
    ) acquires TransactionFee {
        let fee_address =  CoreAddresses::LIBRA_ROOT_ADDRESS();
        if (LBR::is_lbr<CoinType>()) {
            // extract fees
            let fees = borrow_global_mut<TransactionFee<LBR>>(fee_address);
            let coins = Libra::withdraw_all<LBR>(&mut fees.balance);
            let (coin1, coin2) = LBR::unpack(coins);
            // burn
            let coin1_burn_cap = Libra::remove_burn_capability<Coin1>(tc_account);
            let coin2_burn_cap = Libra::remove_burn_capability<Coin2>(tc_account);
            preburn_burn_fees(
                &coin1_burn_cap,
                borrow_global_mut<TransactionFee<Coin1>>(fee_address),
                coin1
            );
            preburn_burn_fees(
                &coin2_burn_cap,
                borrow_global_mut<TransactionFee<Coin2>>(fee_address),
                coin2
            );
            Libra::publish_burn_capability(tc_account, coin1_burn_cap, tc_account);
            Libra::publish_burn_capability(tc_account, coin2_burn_cap, tc_account);
        } else {
            // extract fees
            let fees = borrow_global_mut<TransactionFee<CoinType>>(fee_address);
            let coin = Libra::withdraw_all(&mut fees.balance);
            // burn
            let burn_cap = Libra::remove_burn_capability<CoinType>(tc_account);
            preburn_burn_fees(&burn_cap, fees, coin);
            Libra::publish_burn_capability(tc_account, burn_cap, tc_account);
        }
    }

    spec fun burn_fees {
        // TODO: reactivate after aborts_if soundness fix.
        pragma verify = false;
        /// > TODO(emmazzz): We are not able to specify formally the conditions
        /// involving FixedPoint32. Here are some informal specifications:
        /// (1) aborts if CoinType is LBR and the reserve does not have enough
        ///     backing for Coin1 and Coin2 to unpack the LBR coin.
        /// (2) ensures if CoinType is LBR, then the correct amount of Coin1
        ///     and Coin2 is burnt.
        aborts_if !spec_is_initialized<CoinType>();
        aborts_if LBR::spec_is_lbr<CoinType>()
                && (!spec_is_valid_txn_fee_currency<Coin1>(tc_account)
                 || !spec_is_valid_txn_fee_currency<Coin2>(tc_account));
        aborts_if !LBR::spec_is_lbr<CoinType>()
            && !spec_is_valid_txn_fee_currency<CoinType>(tc_account);
        aborts_if !Roles::spec_has_treasury_compliance_role_addr(Signer::spec_address_of(tc_account));
        /// The correct amount of fees is burnt and subtracted from market cap.
        ensures Libra::spec_market_cap<CoinType>()
            == old(Libra::spec_market_cap<CoinType>()) - old(spec_txn_fee_balance<CoinType>());
        /// All the fees is burnt so the balance becomes 0.
        ensures spec_txn_fee_balance<CoinType>() == 0;
    }

    spec module {
        define spec_is_valid_txn_fee_currency<CoinType>(tc_account: signer): bool {
            Libra::spec_is_currency<CoinType>()
            && spec_is_initialized<CoinType>()
            && Libra::spec_has_burn_cap<CoinType>(Signer::spec_address_of(tc_account))
        }
    }

    /// Preburn `coin` to the `Preburn` inside `fees`, then immediately burn them using `burn_cap`.
    fun preburn_burn_fees<CoinType>(
        burn_cap: &BurnCapability<CoinType>,
        fees: &mut TransactionFee<CoinType>,
        coin: Libra<CoinType>
    ) {
        let tc_address = CoreAddresses::TREASURY_COMPLIANCE_ADDRESS();
        let preburn = &mut fees.preburn;
        Libra::preburn_with_resource(coin, preburn, tc_address);
        Libra::burn_with_resource_cap(preburn, tc_address, burn_cap)
    }

    spec fun preburn_burn_fees {
        pragma assume_no_abort_from_here = true, opaque = true;
        aborts_if !Libra::spec_is_currency<CoinType>();
        aborts_if Libra::spec_currency_info<CoinType>().preburn_value + coin.value > max_u64();
        aborts_if fees.preburn.to_burn.value > 0;
        aborts_if coin.value == 0;
        aborts_if Libra::spec_market_cap<CoinType>() < coin.value;
        ensures Libra::spec_market_cap<CoinType>()
            == old(Libra::spec_market_cap<CoinType>()) - coin.value;
    }

    spec module {
        pragma verify = true;
    }
}
}
