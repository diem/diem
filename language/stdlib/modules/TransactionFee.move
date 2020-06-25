address 0x1 {

module TransactionFee {
    use 0x1::CoreAddresses;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::LBR::{Self, LBR};
    use 0x1::Libra::{Self, Libra, Preburn, BurnCapability};
    use 0x1::Signer;
    use 0x1::Roles::{Capability, TreasuryComplianceRole};

    /// The `TransactionFee` resource holds a preburn resource for each
    /// fiat `CoinType` that can be collected as a transaction fee.
    resource struct TransactionFee<CoinType> {
        balance: Libra<CoinType>,
        preburn: Preburn<CoinType>,
    }

    /// Called in genesis. Sets up the needed resources to collect transaction fees from the
    /// `TransactionFee` resource with the TreasuryCompliance account.
    public fun initialize(
        assoc_account: &signer,
        tc_capability: &Capability<TreasuryComplianceRole>,
    ) {
        assert(
            Signer::address_of(assoc_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(),
            0
        );

        // accept fees in all the currencies
        add_txn_fee_currency<Coin1>(assoc_account, tc_capability);
        add_txn_fee_currency<Coin2>(assoc_account, tc_capability);
        add_txn_fee_currency<LBR>(assoc_account, tc_capability);
    }

    /// Sets ups the needed transaction fee state for a given `CoinType` currency by
    /// (1) configuring `fee_account` to accept `CoinType`
    /// (2) publishing a wrapper of the `Preburn<CoinType>` resource under `fee_account`
    fun add_txn_fee_currency<CoinType>(
        assoc_account: &signer,
        tc_capability: &Capability<TreasuryComplianceRole>,
    ) {
        move_to(
            assoc_account,
            TransactionFee<CoinType> {
                balance: Libra::zero(),
                preburn: Libra::create_preburn(tc_capability)
            }
        )
    }

    /// Deposit `coin` into the transaction fees bucket
    public fun pay_fee<CoinType>(coin: Libra<CoinType>) acquires TransactionFee {
        let fees = borrow_global_mut<TransactionFee<CoinType>>(
            CoreAddresses::LIBRA_ROOT_ADDRESS()
        );
        Libra::deposit(&mut fees.balance, coin)
    }

    /// Preburns the transaction fees collected in the `CoinType` currency.
    /// If the `CoinType` is LBR, it unpacks the coin and preburns the
    /// underlying fiat.
    public fun burn_fees<CoinType>(
        tc_account: &signer,
        tc_capability: &Capability<TreasuryComplianceRole>,
    ) acquires TransactionFee {
        let fee_address =  CoreAddresses::LIBRA_ROOT_ADDRESS();
        if (LBR::is_lbr<CoinType>()) {
            // extract fees
            let fees = borrow_global_mut<TransactionFee<LBR>>(fee_address);
            let coins = Libra::withdraw_all<LBR>(&mut fees.balance);
            let (coin1, coin2) = LBR::unpack(tc_account, coins);
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
            Libra::publish_burn_capability(tc_account, coin1_burn_cap, tc_capability);
            Libra::publish_burn_capability(tc_account, coin2_burn_cap, tc_capability);
        } else {
            // extract fees
            let fees = borrow_global_mut<TransactionFee<CoinType>>(fee_address);
            let coin = Libra::withdraw_all(&mut fees.balance);
            // burn
            let burn_cap = Libra::remove_burn_capability<CoinType>(tc_account);
            preburn_burn_fees(&burn_cap, fees, coin);
            Libra::publish_burn_capability(tc_account, burn_cap, tc_capability);
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

}
}
