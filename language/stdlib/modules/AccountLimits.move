address 0x1 {

module AccountLimits {
    use 0x1::Errors;
    use 0x1::LibraTimestamp;
    use 0x1::Roles;
    use 0x1::Signer;

    spec module {
        pragma verify;
    }
    /// An operations capability that restricts callers of this module since
    /// the operations can mutate account states.
    resource struct AccountLimitMutationCapability { }

    /// A resource specifying the account limits per-currency. There is a default
    /// "unlimited" `LimitsDefinition` resource for accounts published at
    /// `CoreAddresses::LIBRA_ROOT_ADDRESS()`, but other accounts may have
    /// different account limit definitons. In such cases, they will have a
    /// `LimitsDefinition` published under their (root) account.
    resource struct LimitsDefinition<CoinType> {
        /// The maximum inflow allowed during the specified time period.
        max_inflow: u64,
        /// The maximum outflow allowed during the specified time period.
        max_outflow: u64,
        /// Time period, specified in microseconds
        time_period: u64,
        /// The maximum amount that can be held
        max_holding: u64,
    }
    spec struct LimitsDefinition {
        invariant max_inflow > 0;
        invariant max_outflow > 0;
        invariant time_period > 0;
        invariant max_holding > 0;
    }

    /// A struct holding account transaction information for the time window
    /// starting at `window_start` and lasting for the `time_period` specified
    /// in the limits definition at `limit_address`.
    resource struct Window<CoinType> {
        /// Time window start in microseconds
        window_start: u64,
        /// The inflow during this time window
        window_inflow: u64,
        /// The inflow during this time window
        window_outflow: u64,
        /// The balance that this account has held during this time period.
        tracked_balance: u64,
        /// address storing the LimitsDefinition resource that governs this window
        limit_address: address,
    }

    /// Invariant that `LimitsDefinition` exists if a `Window` exists.
    spec module {
        // TODO(wrwg): this invariant currently leads to non-termination as it produces a new address
        //   from an implication. Should try to generate a trigger for this kind of invariants, perhaps
        //   this fixes it.
        //invariant [global]
        //   forall window_addr: address, coin_type: type where exists<Window<coin_type>>(window_addr):
        //        exists<LimitsDefinition<coin_type>>(global<Window<coin_type>>(window_addr).limit_address);
    }

    const ELIMITS_DEFINITION: u64 = 0;
    const EWINDOW: u64 = 1;

    /// 24 hours in microseconds
    const ONE_DAY: u64 = 86400000000;
    const U64_MAX: u64 = 18446744073709551615u64;

    /// Grant a capability to call this module. This does not necessarily
    /// need to be a unique capability.
    public fun grant_mutation_capability(lr_account: &signer): AccountLimitMutationCapability {
        LibraTimestamp::assert_genesis();
        Roles::assert_libra_root(lr_account);
        AccountLimitMutationCapability{}
    }
    spec fun grant_mutation_capability {
        include LibraTimestamp::AbortsIfNotGenesis;
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
    }

    /// Determines if depositing `amount` of `CoinType` coins into the
    /// account at `addr` is amenable with their account limits.
    /// Returns false if this deposit violates the account limits. Effectful.
    public fun update_deposit_limits<CoinType>(
        amount: u64,
        addr: address,
        _cap: &AccountLimitMutationCapability,
    ): bool acquires LimitsDefinition, Window {
        assert(exists<Window<CoinType>>(addr), Errors::not_published(EWINDOW));
        can_receive<CoinType>(
            amount,
            borrow_global_mut<Window<CoinType>>(addr),
        )
    }
    spec fun update_deposit_limits {
        include UpdateDepositLimitsAbortsIf<CoinType>;
        ensures result == old(spec_update_deposit_limits<CoinType>(amount, addr));
    }
    spec schema UpdateDepositLimitsAbortsIf<CoinType> {
        amount: u64;
        addr: address;
        include AbortsIfNoWindow<CoinType>;
        include CanReceiveAbortsIf<CoinType>{receiving: global<Window<CoinType>>(addr)};
    }
    spec schema AbortsIfNoWindow<CoinType> {
        addr: address;
        aborts_if !exists<Window<CoinType>>(addr) with Errors::NOT_PUBLISHED;
    }
    spec define spec_update_deposit_limits<CoinType>(amount: u64, addr: address): bool {
        spec_receiving_limits_ok(global<Window<CoinType>>(addr), amount)
    }

    /// Determine if withdrawing `amount` of `CoinType` coins from
    /// the account at `addr` would violate the account limits for that account.
    /// Returns `false` if this withdrawal violates account limits. Effectful.
    public fun update_withdrawal_limits<CoinType>(
        amount: u64,
        addr: address,
        _cap: &AccountLimitMutationCapability,
    ): bool acquires LimitsDefinition, Window {
        assert(exists<Window<CoinType>>(addr), Errors::not_published(EWINDOW));
        can_withdraw<CoinType>(
            amount,
            borrow_global_mut<Window<CoinType>>(addr),
        )
    }
    spec fun update_withdrawal_limits {
        include UpdateWithdrawalLimitsAbortsIf<CoinType>;
        ensures result == old(spec_update_withdrawal_limits<CoinType>(amount, addr));
    }
    spec schema UpdateWithdrawalLimitsAbortsIf<CoinType> {
        amount: u64;
        addr: address;
        include AbortsIfNoWindow<CoinType>;
        include CanWithdrawAbortsIf<CoinType>{sending: global<Window<CoinType>>(addr)};
    }
    spec define spec_update_withdrawal_limits<CoinType>(amount: u64, addr: address): bool {
        spec_withdrawal_limits_ok(global<Window<CoinType>>(addr), amount)
    }

    /// All accounts that could be subject to account limits will have a
    /// `Window` for each currency they can hold published at the top level.
    /// Root accounts for multi-account entities will hold this resource at
    /// their root/parent account.
    public fun publish_window<CoinType>(
        to_limit: &signer,
        _: &AccountLimitMutationCapability,
        limit_address: address,
    ) {
        assert(exists<LimitsDefinition<CoinType>>(limit_address), Errors::not_published(ELIMITS_DEFINITION));
        assert(
            !exists<Window<CoinType>>(Signer::address_of(to_limit)),
            Errors::already_published(EWINDOW)
        );
        move_to(
            to_limit,
            Window<CoinType> {
                window_start: current_time(),
                window_inflow: 0,
                window_outflow: 0,
                tracked_balance: 0,
                limit_address,
            }
        )
    }
    spec fun publish_window {
        // TODO(wrwg): put these conditions into a schema.
        aborts_if !exists<LimitsDefinition<CoinType>>(limit_address) with Errors::NOT_PUBLISHED;
        aborts_if exists<Window<CoinType>>(Signer::spec_address_of(to_limit)) with Errors::ALREADY_PUBLISHED;
    }

    /// Unrestricted limits are represented by setting all fields in the
    /// limits definition to `U64_MAX`. Anyone can publish an unrestricted
    /// limits since no windows will point to this limits definition unless the
    /// TC account, or a caller with access to a `&AccountLimitMutationCapability` points a
    /// window to it. Additionally, the TC controls the values held within this
    /// resource once it's published.
    public fun publish_unrestricted_limits<CoinType>(publish_account: &signer) {
        assert(
            !exists<LimitsDefinition<CoinType>>(Signer::address_of(publish_account)),
            Errors::already_published(ELIMITS_DEFINITION)
        );
        move_to(
            publish_account,
            LimitsDefinition<CoinType> {
                max_inflow: U64_MAX,
                max_outflow: U64_MAX,
                max_holding: U64_MAX,
                time_period: ONE_DAY,
            }
        )
    }
    spec fun publish_unrestricted_limits {
        // TODO(wrwg): put these conditions into a schema.
        aborts_if exists<LimitsDefinition<CoinType>>(Signer::spec_address_of(publish_account))
            with Errors::ALREADY_PUBLISHED;
    }

    /// Updates the `LimitsDefinition<CoinType>` resource at `limit_address`.
    /// If any of the field arguments is `0` the corresponding field is not updated.
    ///
    /// TODO(wrwg): specify
    public fun update_limits_definition<CoinType>(
        tc_account: &signer,
        limit_address: address,
        new_max_inflow: u64,
        new_max_outflow: u64,
        new_max_holding_balance: u64,
        new_time_period: u64,
    ) acquires LimitsDefinition {
        Roles::assert_treasury_compliance(tc_account);
        // As we don't have Optionals for txn scripts, in update_account_limit_definition.move
        // we use 0 value to represent a None (ie no update to that variable)
        assert(exists<LimitsDefinition<CoinType>>(limit_address), Errors::not_published(ELIMITS_DEFINITION));
        let limits_def = borrow_global_mut<LimitsDefinition<CoinType>>(limit_address);
        if (new_max_inflow > 0) { limits_def.max_inflow = new_max_inflow };
        if (new_max_outflow > 0) { limits_def.max_outflow = new_max_outflow };
        if (new_max_holding_balance > 0) { limits_def.max_holding = new_max_holding_balance };
        if (new_time_period > 0) { limits_def.time_period = new_time_period };
    }

    /// Update either the `tracked_balance` or `limit_address` fields of the
    /// `Window<CoinType>` stored under `window_address`.
    /// * Since we don't track balances of accounts before they are limited, once
    ///   they do become limited the approximate balance in `CointType` held by
    ///   the entity across all of its accounts will need to be set by the association.
    ///   if `aggregate_balance` is set to zero the field is not updated.
    /// * This updates the `limit_address` in the window resource to a new limits definition at
    ///   `new_limit_address`. If the `aggregate_balance` needs to be updated
    ///   but the `limit_address` should remain the same, the current
    ///   `limit_address` needs to be passed in for `new_limit_address`.
    /// TODO(wrwg): specify
    public fun update_window_info<CoinType>(
        tc_account: &signer,
        window_address: address,
        aggregate_balance: u64,
        new_limit_address: address,
    ) acquires Window {
        Roles::assert_treasury_compliance(tc_account);
        let window = borrow_global_mut<Window<CoinType>>(window_address);
        if (aggregate_balance != 0)  { window.tracked_balance = aggregate_balance };
        assert(exists<LimitsDefinition<CoinType>>(new_limit_address), Errors::not_published(ELIMITS_DEFINITION));
        window.limit_address = new_limit_address;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Internal utiility functions
    ///////////////////////////////////////////////////////////////////////////

    /// If the time window starting at `window.window_start` and lasting for
    /// `limits_definition.time_period` has elapsed, resets the window and
    /// the inflow and outflow records.
    fun reset_window<CoinType>(window: &mut Window<CoinType>, limits_definition: &LimitsDefinition<CoinType>) {
        let current_time = LibraTimestamp::now_microseconds();
        if (current_time > window.window_start + limits_definition.time_period) {
            window.window_start = current_time;
            window.window_inflow = 0;
            window.window_outflow = 0;
        }
    }
    spec fun reset_window {
        include ResetWindowAbortsIf<CoinType>;
        include ResetWindowEnsures<CoinType>;
    }
    spec schema ResetWindowAbortsIf<CoinType> {
        window: Window<CoinType>;
        limits_definition: LimitsDefinition<CoinType>;
        include LibraTimestamp::AbortsIfNoTime;
        aborts_if window.window_start + limits_definition.time_period > max_u64();
    }
    spec schema ResetWindowEnsures<CoinType> {
        window: Window<CoinType>;
        limits_definition: LimitsDefinition<CoinType>;
        ensures window == old(spec_window_reset_with_limits(window, limits_definition));
    }
    spec module {
        define spec_window_expired<CoinType>(
            window: Window<CoinType>,
            limits_definition: LimitsDefinition<CoinType>
        ): bool {
            LibraTimestamp::spec_now_microseconds() > window.window_start + limits_definition.time_period
        }
        define spec_window_reset_with_limits<CoinType>(
            window: Window<CoinType>,
            limits_definition: LimitsDefinition<CoinType>
        ): Window<CoinType> {
            if (spec_window_expired<CoinType>(window, limits_definition)) {
                Window<CoinType>{
                    limit_address: window.limit_address,
                    tracked_balance: window.tracked_balance,
                    window_start: LibraTimestamp::spec_now_microseconds(),
                    window_inflow: 0,
                    window_outflow: 0
                }
            } else {
                window
            }
        }
    }

    /// Verify that the receiving account tracked by the `receiving` window
    /// can receive `amount` funds without violating requirements
    /// specified the `limits_definition` passed in.
    fun can_receive<CoinType>(
        amount: u64,
        receiving: &mut Window<CoinType>,
    ): bool acquires LimitsDefinition {
        let limits_definition = borrow_global_mut<LimitsDefinition<CoinType>>(receiving.limit_address);
        // If the limits are unrestricted then don't do any more work.
        if (is_unrestricted(limits_definition)) return true;

        reset_window(receiving, limits_definition);
        // Check that the inflow is OK
        // TODO(wrwg): instead of aborting if the below additions overflow, we should perhaps just have ok false.
        let inflow_ok = receiving.window_inflow + amount <= limits_definition.max_inflow;
        // Check that the holding after the deposit is OK
        let holding_ok = receiving.tracked_balance + amount <= limits_definition.max_holding;
        // The account with `receiving` window can receive the payment so record it.
        if (inflow_ok && holding_ok) {
            receiving.window_inflow = receiving.window_inflow + amount;
            receiving.tracked_balance = receiving.tracked_balance + amount;
        };
        inflow_ok && holding_ok
    }
    spec fun can_receive {
        include CanReceiveAbortsIf<CoinType>;
        ensures result == spec_receiving_limits_ok(old(receiving), amount);
        ensures
            if (result && !spec_window_unrestricted(old(receiving)))
                receiving.window_inflow == spec_window_reset(old(receiving)).window_inflow + amount &&
                receiving.tracked_balance == spec_window_reset(old(receiving)).tracked_balance + amount
            else
                receiving == spec_window_reset(old(receiving)) || receiving == old(receiving);
    }
    spec schema CanReceiveAbortsIf<CoinType> {
        amount: num;
        receiving: Window<CoinType>;
        aborts_if !exists<LimitsDefinition<CoinType>>(receiving.limit_address);
        include !spec_window_unrestricted<CoinType>(receiving) ==> CanReceiveRestrictedAbortsIf<CoinType>;
    }
    spec schema CanReceiveRestrictedAbortsIf<CoinType> {
        amount: num;
        receiving: Window<CoinType>;
        include ResetWindowAbortsIf<CoinType>{
            window: receiving,
            limits_definition: spec_window_limits<CoinType>(receiving)
        };
        aborts_if spec_window_reset(receiving).window_inflow + amount > max_u64();
        aborts_if spec_window_reset(receiving).tracked_balance + amount > max_u64();
    }
    spec module {
        define spec_window_limits<CoinType>(window: Window<CoinType>): LimitsDefinition<CoinType> {
           global<LimitsDefinition<CoinType>>(window.limit_address)
        }
        define spec_window_unrestricted<CoinType>(window: Window<CoinType>): bool {
            spec_is_unrestricted(spec_window_limits<CoinType>(window))
        }
        define spec_window_reset<CoinType>(window: Window<CoinType>): Window<CoinType> {
            spec_window_reset_with_limits(window, spec_window_limits<CoinType>(window))
        }
        define spec_receiving_limits_ok<CoinType>(receiving: Window<CoinType>, amount: u64): bool {
            spec_window_unrestricted(receiving) ||
            spec_window_reset(receiving).window_inflow + amount
                    <= spec_window_limits(receiving).max_inflow &&
            spec_window_reset(receiving).tracked_balance + amount
                    <= spec_window_limits(receiving).max_holding
        }
    }

    /// Verify that `amount` can be withdrawn from the account tracked
    /// by the `sending` window without violating any limits specified
    /// in its `limits_definition`.
    fun can_withdraw<CoinType>(
        amount: u64,
        sending: &mut Window<CoinType>,
    ): bool acquires LimitsDefinition {
        let limits_definition = borrow_global_mut<LimitsDefinition<CoinType>>(sending.limit_address);
        // If the limits are unrestricted then don't do any more work.
        if (is_unrestricted(limits_definition)) return true;

        reset_window(sending, limits_definition);
        // Check outflow is OK
        let outflow_ok = sending.window_outflow + amount <= limits_definition.max_outflow;
        // Flow is OK, so record it.
        if (outflow_ok) {
            sending.window_outflow = sending.window_outflow + amount;
            sending.tracked_balance = if (amount >= sending.tracked_balance) 0
                                       else sending.tracked_balance - amount;
        };
        outflow_ok
    }
    spec fun can_withdraw {
        include CanWithdrawAbortsIf<CoinType>;
        ensures result == spec_withdrawal_limits_ok(old(sending), amount);
        ensures
            if (result && !spec_window_unrestricted(old(sending)))
                sending.window_outflow == spec_window_reset(old(sending)).window_outflow + amount
            else
                sending == spec_window_reset(old(sending)) || sending == old(sending);
    }
    spec schema CanWithdrawAbortsIf<CoinType> {
        amount: u64;
        sending: &mut Window<CoinType>;
        aborts_if !exists<LimitsDefinition<CoinType>>(sending.limit_address);
        include !spec_window_unrestricted(sending) ==> CanWithdrawRestrictedAbortsIf<CoinType>;
    }
    spec schema CanWithdrawRestrictedAbortsIf<CoinType> {
        amount: u64;
        sending: &mut Window<CoinType>;
        include ResetWindowAbortsIf<CoinType>{
            window: sending,
            limits_definition: spec_window_limits<CoinType>(sending)
        };
        aborts_if spec_window_reset(sending).window_outflow + amount > max_u64();
    }
    spec module {
       define spec_withdrawal_limits_ok<CoinType>(sending: Window<CoinType>, amount: u64): bool {
            spec_window_unrestricted(sending) ||
            spec_window_reset(sending).window_outflow + amount <= spec_window_limits(sending).max_outflow
        }
    }

    /// Return whether the `LimitsDefinition` resoure is unrestricted or not.
    fun is_unrestricted<CoinType>(limits_def: &LimitsDefinition<CoinType>): bool {
        limits_def.max_inflow == U64_MAX &&
        limits_def.max_outflow == U64_MAX &&
        limits_def.max_holding == U64_MAX &&
        limits_def.time_period == ONE_DAY
    }
    spec fun is_unrestricted {
        pragma opaque;
        aborts_if false;
        ensures result == spec_is_unrestricted(limits_def);
    }
    spec module {
        define spec_is_unrestricted<CoinType>(limits_def: LimitsDefinition<CoinType>): bool {
            limits_def.max_inflow == max_u64() &&
            limits_def.max_outflow == max_u64() &&
            limits_def.max_holding == max_u64() &&
            limits_def.time_period == 86400000000
        }
    }

    public fun limits_definition_address<CoinType>(addr: address): address acquires Window {
        borrow_global<Window<CoinType>>(addr).limit_address
    }

    public fun has_limits_published<CoinType>(addr: address): bool {
        exists<LimitsDefinition<CoinType>>(addr)
    }

    public fun has_window_published<CoinType>(addr: address): bool {
        exists<Window<CoinType>>(addr)
    }
    spec fun has_window_published {
        ensures result == spec_has_window_published<CoinType>(addr);
    }
    spec module {
        define spec_has_window_published<CoinType>(addr: address): bool {
            exists<Window<CoinType>>(addr)
        }
    }

    fun current_time(): u64 {
        if (LibraTimestamp::is_not_initialized()) 0 else LibraTimestamp::now_microseconds()
    }

}
}
