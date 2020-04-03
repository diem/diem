// dep: tests/sources/stdlib/modules/libra_account.move
// dep: tests/sources/stdlib/modules/hash.move
// dep: tests/sources/stdlib/modules/lbr.move
// dep: tests/sources/stdlib/modules/lcs.move
// dep: tests/sources/stdlib/modules/libra.move
// dep: tests/sources/stdlib/modules/libra_transaction_timeout.move
// dep: tests/sources/stdlib/modules/transaction.move
// dep: tests/sources/stdlib/modules/vector.move
// dep: tests/sources/stdlib/modules/libra_time.move
// dep: tests/sources/stdlib/modules/libra_system.move
// dep: tests/sources/stdlib/modules/validator_config.move

address 0x0:

module TransactionFee {
    use 0x0::LibraAccount;
    use 0x0::LibraSystem;
    use 0x0::Transaction;

    ///////////////////////////////////////////////////////////////////////////
    // Transaction Fee Distribution
    ///////////////////////////////////////////////////////////////////////////
    // Implements a basic transaction fee distribution logic.
    //
    // We have made a couple design decisions here that are worth noting:
    //  * We pay out once per-block for now.
    //    TODO: Once we have a better on-chain representation of
    //          epochs this should be changed over to be once per-epoch.
    //  * Sometimes the number of validators does not evenly divide the transaction fees to be
    //    distributed. In such cases the remainder ("dust") is left in the transaction fees pot and
    //    these remaining fees will be included in the calculations for the transaction fee
    //    distribution in the next epoch. This distribution strategy is meant to in part minimize the
    //    benefit of being the first validator in the validator set.

    resource struct TransactionFees {
        fee_withdrawal_capability: LibraAccount::WithdrawalCapability,
    }

    // Initialize the transaction fee distribution module. We keep track of the last paid block
    // height in order to ensure that we don't try to pay more than once per-block. We also
    // encapsulate the withdrawal capability to the transaction fee account so that we can withdraw
    // the fees from this account from block metadata transactions.
    fun initialize_transaction_fees() {
        Transaction::assert(Transaction::sender() == 0xFEE, 0);
        move_to_sender<TransactionFees>(TransactionFees {
            fee_withdrawal_capability: LibraAccount::extract_sender_withdrawal_capability(),
        });
    }

    public fun distribute_transaction_fees<Token>() acquires TransactionFees {
      // Can only be invoked by LibraVM privilege.
      Transaction::assert(Transaction::sender() == 0x0, 33);

      let num_validators = LibraSystem::validator_set_size();
      let amount_collected = LibraAccount::balance<Token>(0xFEE);

      // If amount_collected == 0, this will also return early
      if (amount_collected < num_validators) return ();

      // Calculate the amount of money to be dispursed, along with the remainder.
      let amount_to_distribute_per_validator = per_validator_distribution_amount(
          amount_collected,
          num_validators
      );

      // Iterate through the validators distributing fees equally
      distribute_transaction_fees_internal<Token>(
          amount_to_distribute_per_validator,
          num_validators,
      );
    }

    // After the book keeping has been performed, this then distributes the
    // transaction fees equally to all validators with the exception that
    // any remainder (in the case that the number of validators does not
    // evenly divide the transaction fee pot) is distributed to the first
    // validator.
    fun distribute_transaction_fees_internal<Token>(
        amount_to_distribute_per_validator: u64,
        num_validators: u64
    ) acquires TransactionFees {
        let distribution_resource = borrow_global<TransactionFees>(0xFEE);
        let index = 0;

        while (index < num_validators) {

            let addr = LibraSystem::get_ith_validator_address(index);
            // Increment the index into the validator set.
            index = index + 1;

            LibraAccount::pay_from_capability<Token>(
                addr,
                x"",
                &distribution_resource.fee_withdrawal_capability,
                amount_to_distribute_per_validator,
                x"",
            );
           }
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
}
