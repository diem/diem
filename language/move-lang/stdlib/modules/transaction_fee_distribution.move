address 0x0:

// This module implements a basic transaction fee distribution logic.
//
// We have made a couple design decisions here that are worth noting:
//  * We pay out per-epoch. However this functionality is only in spirit at the moment
//    since we have neither block metadata transactions nor epochs (at the Move level).
//    TODO: Once we have block metadata transactions and epochs, this code needs to be updated to
//    distribute fees every epoch and be called from the metadata transaction.
//  * In the handling of dust, we simply leave it to the next distribution.
//    This is meant to in part minimize the benefit of being the first validator in
//    the validator set.
module TransactionFeeDistribution {
    use 0x0::LibraCoin;
    use 0x0::ValidatorSet;
    use 0x0::LibraAccount;
    use 0x0::Transaction;
    // import 0x0::Epoch;

    resource struct T {
        last_epoch_paid: u64,
        fee_withdrawal_capability: LibraAccount::WithdrawalCapability,
    }

    // Initialize the transaction fee distribution module. We keep track of the last paid block
    // height in order to ensure that we don't try to pay more than once per-block. We also
    // encapsulate the withdrawal capability to the transaction fee account so that we can withdraw
    // the fees from this account from block metadata transactions.
    public fun initialize() {
        Transaction::assert(Transaction::sender() == 0xFEE, 0);
        move_to_sender(T {
            // last_epoch_paid: Epoch::get_current_epoch(),
            last_epoch_paid: 0,
            fee_withdrawal_capability: LibraAccount::extract_sender_withdrawal_capability(),
        })
    }

    // TODO: This is public for now for testing purposes. However, in the future block metadata
    // transactions will call this and it will be marked as a private function.
    public fun distribute_transaction_fees() acquires T {
        let distribution_resource = borrow_global_mut<T>(0xFEE);
        //current_epoch = Epoch::get_current_epoch();

        // TODO: This check and update is commented out at the moment since we don't have epochs.
        //assert(*&copy(distribution_resource).last_epoch_paid < copy(current_epoch), 0);
        // Note: We don't enforce sequentiality for transaction fee distribution.
        //*(&mut copy(distribution_resource).last_epoch_paid) = move(current_epoch);

        let amount_collected = LibraAccount::balance(0xFEE);

        let lib_coin_to_distribute =
        LibraAccount::withdraw_with_capability(
            &distribution_resource.fee_withdrawal_capability,
            amount_collected,
        );

        let num_validators = ValidatorSet::size();
        // Calculate the amount of money to be dispursed, along with the remainder.
        let amount_to_distribute_per_validator = Self::per_validator_distribution_amount(
            amount_collected,
            num_validators,
        );

        // Iterate through the validators distributing fees equally
        Self::distribute_transaction_fees_internal(
            lib_coin_to_distribute,
            amount_to_distribute_per_validator,
            num_validators,
        );
    }

    // This calculates the amount to be distributed to each validator equally. We do this by calculating
    // the integer division of the transaction fees collected by the number of validators. In
    // particular, this means that if the number of validators does not evenly divide the
    // transaction fees collected, then there will be a remainder that is left in the transaction
    // fees pot to be distributed later.
    fun per_validator_distribution_amount(amount_collected: u64, num_validators: u64): u64 {
        Transaction::assert(num_validators != 0, 0);
        let validator_payout = amount_collected / num_validators;
        Transaction::assert(validator_payout * num_validators <= amount_collected, 1);
        validator_payout
    }

    // After the book keeping has been performed, this then distributes the
    // transaction fees equally to all validators with the exception that
    // any remainder (in the case that the number of validators does not
    // evenly divide the transaction fee pot) is distributed to the first
    // validator.
    fun distribute_transaction_fees_internal(
        collected_fees: LibraCoin::T,
        amount_to_distribute_per_validator: u64,
        num_validators: u64
    ) {
        let index = 0;
        while (index < num_validators) {
            let addr = ValidatorSet::get_ith_validator_address(index);
            // Increment the index into the validator set.
            index = index + 1;
            let payment;
            (collected_fees, payment) = LibraCoin::split(
                collected_fees,
                amount_to_distribute_per_validator,
            );
            LibraAccount::deposit(addr, payment);
        };

        // Now pay back any remainder to the transaction fees. Deposits with value zero
        // are not allowed so if the remainder is zero we have to destroy the coin.
        if (LibraCoin::value(&collected_fees) == 0) LibraCoin::destroy_zero(collected_fees)
        else LibraAccount::deposit(0xFEE, collected_fees)
    }

}
