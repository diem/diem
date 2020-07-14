address 0x1 {

module AccountLimits {
    use 0x1::LibraTimestamp;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::LBR::LBR;
    use 0x1::Roles;
    use 0x1::CoreAddresses;
    use 0x1::Signer;

    spec module {
        pragma verify = true;
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

    const ENOT_GENESIS: u64 = 0;
    const EINVALID_INITIALIZATION_ADDRESS: u64 = 1;
    const ENOT_LIBRA_ROOT: u64 = 2;
    const ENOT_TREASURY_COMPLIANCE: u64 = 3;
    const ENO_LIMITS_DEFINITION_EXISTS: u64 = 4;
    const ELIMITS_DEFINITION_ALREADY_EXISTS: u64 = 5;


    /// 24 hours in microseconds
    const ONE_DAY: u64 = 86400000000;
    const U64_MAX: u64 = 18446744073709551615u64;

    /// Grant a capability to call this module. This does not necessarily
    /// need to be a unique capability.
    public fun grant_mutation_capability(lr_account: &signer): AccountLimitMutationCapability {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(Roles::has_libra_root_role(lr_account), ENOT_LIBRA_ROOT);
        AccountLimitMutationCapability{}
    }

    /// Initializes the account limits for accounts.
    public fun initialize(lr_account: &signer) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_INITIALIZATION_ADDRESS);
        publish_unrestricted_limits<LBR>(lr_account);
        publish_unrestricted_limits<Coin1>(lr_account);
        publish_unrestricted_limits<Coin2>(lr_account);
    }

    /// Determines if depositing `amount` of `CoinType` coins into the
    /// account at `addr` is amenable with their account limits.
    /// Returns false if this deposit violates the account limits. Effectful.
    public fun update_deposit_limits<CoinType>(
        amount: u64,
        addr: address,
        _cap: &AccountLimitMutationCapability,
    ): bool acquires LimitsDefinition, Window {
        can_receive<CoinType>(
            amount,
            borrow_global_mut<Window<CoinType>>(addr),
        )
    }
    spec fun update_deposit_limits {
        /// > TODO(wrwg): this is currently abstracted as uninterpreted function
        /// > because of termination issue. Need to investigate why.
        pragma verify = false;
        pragma opaque = true;
        ensures result == spec_update_deposit_limits<CoinType>(amount, addr);
    }
    spec module {
        define spec_update_deposit_limits<CoinType>(amount: u64, addr: address): bool;
    }

    /// Determine if withdrawing `amount` of `CoinType` coins from
    /// the account at `addr` would violate the account limits for that account.
    /// Returns `false` if this withdrawal violates account limits. Effectful.
    public fun update_withdrawal_limits<CoinType>(
        amount: u64,
        addr: address,
        _cap: &AccountLimitMutationCapability,
    ): bool acquires LimitsDefinition, Window {
        can_withdraw<CoinType>(
            amount,
            borrow_global_mut<Window<CoinType>>(addr),
        )
    }

    spec fun update_withdrawal_limits {
        /// > TODO(emmazzz): turn verify on and opaque off when the module
        /// > is fully specified.
        pragma verify = false;
        pragma opaque = true;
        ensures result == spec_update_withdrawal_limits<CoinType>(amount, addr);
    }

    spec module {
        define spec_update_withdrawal_limits<CoinType>(amount: u64, addr: address): bool;
    }

    /// All accounts that could be subject to account limits will have a
    /// `Window` for each currency they can hold published at the top level.
    /// Root accounts for multi-account entities will hold this resource at
    /// their root/parent account.
    /// Aborts with ELIMITS_DEFINITION_ALREADY_EXISTS if `to_limit` already contains a
    /// Window<CoinType>
    public fun publish_window<CoinType>(
        to_limit: &signer,
        _: &AccountLimitMutationCapability,
        limit_address: address,
    ) {
        assert(
            !exists<Window<CoinType>>(Signer::address_of(to_limit)),
            ELIMITS_DEFINITION_ALREADY_EXISTS
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

    /// Unrestricted limits are represented by setting all fields in the
    /// limits definition to `U64_MAX`. Anyone can publish an unrestricted
    /// limits since no windows will point to this limits definition unless the
    /// TC account, or a caller with access to a `&AccountLimitMutationCapability` points a
    /// window to it. Additionally, the TC controls the values held within this
    /// resource once it's published.
    public fun publish_unrestricted_limits<CoinType>(publish_account: &signer) {
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

    /// Updates the `LimitsDefinition<CoinType>` resource at `limit_address`.
    /// If any of the field arguments is `0` the corresponding field is not updated.
    public fun update_limits_definition<CoinType>(
        tc_account: &signer,
        limit_address: address,
        new_max_inflow: u64,
        new_max_outflow: u64,
        new_max_holding_balance: u64,
        new_time_period: u64,
    ) acquires LimitsDefinition {
        assert(Roles::has_treasury_compliance_role(tc_account), ENOT_TREASURY_COMPLIANCE);
        // As we don't have Optionals for txn scripts, in update_account_limit_definition.move
        // we use 0 value to represent a None (ie no update to that variable)
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
    public fun update_window_info<CoinType>(
        tc_account: &signer,
        window_address: address,
        aggregate_balance: u64,
        new_limit_address: address,
    ) acquires Window {
        assert(Roles::has_treasury_compliance_role(tc_account), ENOT_TREASURY_COMPLIANCE);
        let window = borrow_global_mut<Window<CoinType>>(window_address);
        if (aggregate_balance != 0)  { window.tracked_balance = aggregate_balance };
        assert(exists<LimitsDefinition<CoinType>>(new_limit_address), ENO_LIMITS_DEFINITION_EXISTS);
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
    }
    spec schema ResetWindowAbortsIf<CoinType> {
        window: Window<CoinType>;
        limits_definition: LimitsDefinition<CoinType>;
        include LibraTimestamp::TimeAccessAbortsIf;
        aborts_if window.window_start + limits_definition.time_period > max_u64();
    }
    spec module {
        define spec_window_expired<CoinType>(
            window: Window<CoinType>,
            limits_definition: LimitsDefinition<CoinType>
        ): bool {
            LibraTimestamp::spec_now_microseconds() > window.window_start + limits_definition.time_period
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
        pragma verify = true;
        include CanReceiveAbortsIf<CoinType>;
        include CanReceiveEnsures<CoinType>;
    }
    spec schema CanReceiveAbortsIf<CoinType> {
        amount: num;
        receiving: Window<CoinType>;
        aborts_if !exists<LimitsDefinition<CoinType>>(receiving.limit_address);
        include !spec_receiving_is_unrestricted<CoinType>(receiving)
                    ==> CanReceiveRestrictedAbortsIf<CoinType>;
    }
    spec schema CanReceiveRestrictedAbortsIf<CoinType> {
        amount: num;
        receiving: Window<CoinType>;
        aborts_if !spec_receiving_expired(receiving) && receiving.window_inflow + amount > max_u64();
        aborts_if receiving.tracked_balance + amount > max_u64();
        include ResetWindowAbortsIf<CoinType>{
            window: receiving,
            limits_definition: spec_limits_definition<CoinType>(receiving.limit_address)
        };
    }
    spec schema CanReceiveEnsures<CoinType> {
        amount: num;
        receiving: Window<CoinType>;
    }
    spec module {
        define spec_limits_definition<CoinType>(addr: address): LimitsDefinition<CoinType> {
           global<LimitsDefinition<CoinType>>(addr)
        }
        define spec_receiving_is_unrestricted<CoinType>(receiving: Window<CoinType>): bool {
            spec_is_unrestricted(spec_limits_definition<CoinType>(receiving.limit_address))
        }
        define spec_receiving_expired<CoinType>(receiving: Window<CoinType>): bool {
            spec_window_expired(receiving, spec_limits_definition<CoinType>(receiving.limit_address))
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

    /// Return whether the `LimitsDefinition` resoure is unrestricted or not.
    fun is_unrestricted<CoinType>(limits_def: &LimitsDefinition<CoinType>): bool {
        limits_def.max_inflow == U64_MAX &&
        limits_def.max_outflow == U64_MAX &&
        limits_def.max_holding == U64_MAX &&
        limits_def.time_period == ONE_DAY
    }
    spec fun is_unrestricted {
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

    public fun is_unlimited_account<CoinType>(addr: address): bool acquires LimitsDefinition {
        is_unrestricted(borrow_global<LimitsDefinition<CoinType>>(addr))
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
