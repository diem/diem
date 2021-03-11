address 0x1 {
/// This module holds scripts relating to treasury and compliance-related
/// activities in the Diem Framework.
///
/// Only accounts with a role of `Roles::TREASURY_COMPLIANCE` and
/// `Roles::DESIGNATED_DEALER` can (successfully) use the scripts in this
/// module. The exact role required for a transaction is determined on a
/// per-transaction basis.
module TreasuryComplianceScripts {
    use 0x1::DiemAccount;
    use 0x1::Diem;
    use 0x1::SlidingNonce;

    /// # Summary
    /// Cancels and returns the coins held in the preburn area under
    /// `preburn_address`, which are equal to the `amount` specified in the transaction. Finds the first preburn
    /// resource with the matching amount and returns the funds to the `preburn_address`'s balance.
    /// Can only be successfully sent by an account with Treasury Compliance role.
    ///
    /// # Technical Description
    /// Cancels and returns all coins held in the `Diem::Preburn<Token>` resource under the `preburn_address` and
    /// return the funds to the `preburn_address` account's `DiemAccount::Balance<Token>`.
    /// The transaction must be sent by an `account` with a `Diem::BurnCapability<Token>`
    /// resource published under it. The account at `preburn_address` must have a
    /// `Diem::Preburn<Token>` resource published under it, and its value must be nonzero. The transaction removes
    /// the entire balance held in the `Diem::Preburn<Token>` resource, and returns it back to the account's
    /// `DiemAccount::Balance<Token>` under `preburn_address`. Due to this, the account at
    /// `preburn_address` must already have a balance in the `Token` currency published
    /// before this script is called otherwise the transaction will fail.
    ///
    /// ## Events
    /// The successful execution of this transaction will emit:
    /// * A `Diem::CancelBurnEvent` on the event handle held in the `Diem::CurrencyInfo<Token>`
    /// resource's `burn_events` published under `0xA550C18`.
    /// * A `DiemAccount::ReceivedPaymentEvent` on the `preburn_address`'s
    /// `DiemAccount::DiemAccount` `received_events` event handle with both the `payer` and `payee`
    /// being `preburn_address`.
    ///
    /// # Parameters
    /// | Name              | Type      | Description                                                                                                                          |
    /// | ------            | ------    | -------------                                                                                                                        |
    /// | `Token`           | Type      | The Move type for the `Token` currenty that burning is being cancelled for. `Token` must be an already-registered currency on-chain. |
    /// | `account`         | `&signer` | The signer reference of the sending account of this transaction, must have a burn capability for `Token` published under it.         |
    /// | `preburn_address` | `address` | The address where the coins to-be-burned are currently held.                                                                         |
    /// | `amount`          | `u64`     | The amount to be cancelled.                                                                                                          |
    ///
    /// # Common Abort Conditions
    /// | Error Category                | Error Reason                                     | Description                                                                                                                         |
    /// | ----------------              | --------------                                   | -------------                                                                                                                       |
    /// | `Errors::REQUIRES_CAPABILITY` | `Diem::EBURN_CAPABILITY`                         | The sending `account` does not have a `Diem::BurnCapability<Token>` published under it.                                             |
    /// | `Errors::INVALID_STATE`       | `Diem::EPREBURN_NOT_FOUND`                       | The `Diem::PreburnQueue<Token>` resource under `preburn_address` does not contain a preburn request with a value matching `amount`. |
    /// | `Errors::NOT_PUBLISHED`       | `Diem::EPREBURN_QUEUE`                           | The account at `preburn_address` does not have a `Diem::PreburnQueue<Token>` resource published under it.                           |
    /// | `Errors::NOT_PUBLISHED`       | `Diem::ECURRENCY_INFO`                           | The specified `Token` is not a registered currency on-chain.                                                                        |
    /// | `Errors::INVALID_ARGUMENT`    | `DiemAccount::EPAYEE_CANT_ACCEPT_CURRENCY_TYPE`  | The account at `preburn_address` doesn't have a balance resource for `Token`.                                                       |
    /// | `Errors::LIMIT_EXCEEDED`      | `DiemAccount::EDEPOSIT_EXCEEDS_LIMITS`           | The depositing of the funds held in the prebun area would exceed the `account`'s account limits.                                    |
    /// | `Errors::INVALID_STATE`       | `DualAttestation::EPAYEE_COMPLIANCE_KEY_NOT_SET` | The `account` does not have a compliance key set on it but dual attestion checking was performed.                                   |
    ///
    /// # Related Scripts
    /// * `Script::burn_txn_fees`
    /// * `Script::burn`
    /// * `Script::preburn`

    public(script) fun cancel_burn_with_amount<Token: store>(account: &signer, preburn_address: address, amount: u64) {
        DiemAccount::cancel_burn<Token>(account, preburn_address, amount)
    }

    spec fun cancel_burn_with_amount {
        use 0x1::CoreAddresses;
        use 0x1::Errors;
        use 0x1::Diem;

        include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
        include DiemAccount::CancelBurnAbortsIf<Token>;
        include Diem::CancelBurnWithCapEnsures<Token>;
        include DiemAccount::DepositEnsures<Token>{payee: preburn_address};

        let total_preburn_value = global<Diem::CurrencyInfo<Token>>(
            CoreAddresses::CURRENCY_INFO_ADDRESS()
        ).preburn_value;
        let balance_at_addr = DiemAccount::balance<Token>(preburn_address);

        /// The total value of preburn for `Token` should decrease by the preburned amount.
        ensures total_preburn_value == old(total_preburn_value) - amount;

        /// The balance of `Token` at `preburn_address` should increase by the preburned amount.
        ensures balance_at_addr == old(balance_at_addr) + amount;

        include Diem::CancelBurnWithCapEmits<Token>;
        include DiemAccount::DepositEmits<Token>{
            payer: preburn_address,
            payee: preburn_address,
            amount: amount,
            metadata: x""
        };

        aborts_with [check]
            Errors::REQUIRES_CAPABILITY,
            Errors::NOT_PUBLISHED,
            Errors::INVALID_ARGUMENT,
            Errors::LIMIT_EXCEEDED,
            Errors::INVALID_STATE;

        /// **Access Control:**
        /// Only the account with the burn capability can cancel burning [[H3]][PERMISSION].
        include Diem::AbortsIfNoBurnCapability<Token>{account: account};
    }

    /// # Summary
    /// Burns the coins held in a preburn resource in the preburn queue at the
    /// specified preburn address, which are equal to the `amount` specified in the
    /// transaction. Finds the first relevant outstanding preburn request with
    /// matching amount and removes the contained coins from the system. The sending
    /// account must be the Treasury Compliance account.
    /// The account that holds the preburn queue resource will normally be a Designated
    /// Dealer, but there are no enforced requirements that it be one.
    ///
    /// # Technical Description
    /// This transaction permanently destroys all the coins of `Token` type
    /// stored in the `Diem::Preburn<Token>` resource published under the
    /// `preburn_address` account address.
    ///
    /// This transaction will only succeed if the sending `account` has a
    /// `Diem::BurnCapability<Token>`, and a `Diem::Preburn<Token>` resource
    /// exists under `preburn_address`, with a non-zero `to_burn` field. After the successful execution
    /// of this transaction the `total_value` field in the
    /// `Diem::CurrencyInfo<Token>` resource published under `0xA550C18` will be
    /// decremented by the value of the `to_burn` field of the preburn resource
    /// under `preburn_address` immediately before this transaction, and the
    /// `to_burn` field of the preburn resource will have a zero value.
    ///
    /// ## Events
    /// The successful execution of this transaction will emit a `Diem::BurnEvent` on the event handle
    /// held in the `Diem::CurrencyInfo<Token>` resource's `burn_events` published under
    /// `0xA550C18`.
    ///
    /// # Parameters
    /// | Name              | Type      | Description                                                                                                                  |
    /// | ------            | ------    | -------------                                                                                                                |
    /// | `Token`           | Type      | The Move type for the `Token` currency being burned. `Token` must be an already-registered currency on-chain.                |
    /// | `tc_account`      | `&signer` | The signer reference of the sending account of this transaction, must have a burn capability for `Token` published under it. |
    /// | `sliding_nonce`   | `u64`     | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                                                   |
    /// | `preburn_address` | `address` | The address where the coins to-be-burned are currently held.                                                                 |
    /// | `amount`          | `u64`     | The amount to be burned.                                                                                                     |
    ///
    /// # Common Abort Conditions
    /// | Error Category                | Error Reason                            | Description                                                                                                                         |
    /// | ----------------              | --------------                          | -------------                                                                                                                       |
    /// | `Errors::NOT_PUBLISHED`       | `SlidingNonce::ESLIDING_NONCE`          | A `SlidingNonce` resource is not published under `account`.                                                                         |
    /// | `Errors::INVALID_ARGUMENT`    | `SlidingNonce::ENONCE_TOO_OLD`          | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not.                                          |
    /// | `Errors::INVALID_ARGUMENT`    | `SlidingNonce::ENONCE_TOO_NEW`          | The `sliding_nonce` is too far in the future.                                                                                       |
    /// | `Errors::INVALID_ARGUMENT`    | `SlidingNonce::ENONCE_ALREADY_RECORDED` | The `sliding_nonce` has been previously recorded.                                                                                   |
    /// | `Errors::REQUIRES_CAPABILITY` | `Diem::EBURN_CAPABILITY`                | The sending `account` does not have a `Diem::BurnCapability<Token>` published under it.                                             |
    /// | `Errors::INVALID_STATE`       | `Diem::EPREBURN_NOT_FOUND`              | The `Diem::PreburnQueue<Token>` resource under `preburn_address` does not contain a preburn request with a value matching `amount`. |
    /// | `Errors::NOT_PUBLISHED`       | `Diem::EPREBURN_QUEUE`                  | The account at `preburn_address` does not have a `Diem::PreburnQueue<Token>` resource published under it.                           |
    /// | `Errors::NOT_PUBLISHED`       | `Diem::ECURRENCY_INFO`                  | The specified `Token` is not a registered currency on-chain.                                                                        |
    ///
    /// # Related Scripts
    /// * `Script::burn_txn_fees`
    /// * `Script::cancel_burn`
    /// * `Script::preburn`

    public(script) fun burn_with_amount<Token: store>(account: &signer, sliding_nonce: u64, preburn_address: address, amount: u64) {
        SlidingNonce::record_nonce_or_abort(account, sliding_nonce);
        Diem::burn<Token>(account, preburn_address, amount)
    }
    spec fun burn_with_amount {
        use 0x1::Errors;
        use 0x1::DiemAccount;

        // TODO: burn functionality has specification issues. Fix and reactivate.
        pragma verify = false;

        include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
        include SlidingNonce::RecordNonceAbortsIf{ seq_nonce: sliding_nonce };
        include Diem::BurnAbortsIf<Token>;
        include Diem::BurnEnsures<Token>;

        aborts_with [check]
            Errors::INVALID_ARGUMENT,
            Errors::REQUIRES_CAPABILITY,
            Errors::NOT_PUBLISHED,
            Errors::INVALID_STATE,
            Errors::LIMIT_EXCEEDED;

        include Diem::BurnWithResourceCapEmits<Token>{preburn: global<Diem::Preburn<Token>>(preburn_address)};

        /// **Access Control:**
        /// Only the account with the burn capability can burn coins [[H3]][PERMISSION].
        include Diem::AbortsIfNoBurnCapability<Token>{account: account};
    }
}
}
