address 0x1 {

module AccountLimits {
    use 0x1::LibraTimestamp;
    use 0x1::Coin1::Coin1;
    use 0x1::Coin2::Coin2;
    use 0x1::LBR::LBR;
    use 0x1::Roles::{has_libra_root_role, has_treasury_compliance_role};
    use 0x1::CoreAddresses;
    use 0x1::Signer;

    /// An operations capability that restricts callers of this module since
    /// the operations can mutate account states.
    resource struct CallingCapability { }

    /// A resource specifying the account limits per-currency. There is a default
    /// `LimitsDefinition` resource for unhosted accounts published at
    /// `CoreAddresses::LIBRA_ROOT_ADDRESS()`, but other not-unhosted accounts may have
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

    const NOT_GENESIS: u64 = 0;
    const INVALID_INITIALIZATION_ADDRESS: u64 = 1;
    const NOT_LIBRA_ROOT: u64 = 2;
    const NOT_TREASURY_COMPLIANCE: u64 = 3;

    /// 24 hours in microseconds
    const ONE_DAY: u64 = 86400000000;
    const U64_MAX: u64 = 18446744073709551615u64;

    /// Grant a capability to call this module. This does not necessarily
    /// need to be a unique capability.
    public fun grant_calling_capability(lr_account: &signer): CallingCapability {
        assert(LibraTimestamp::is_genesis(), NOT_GENESIS);
        assert(has_libra_root_role(lr_account), NOT_LIBRA_ROOT);
        CallingCapability{}
    }

    /// Initializes the account limits for unhosted accounts.
    public fun initialize(lr_account: &signer, calling_cap: &CallingCapability) {
        assert(LibraTimestamp::is_genesis(), NOT_GENESIS);
        assert(Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), INVALID_INITIALIZATION_ADDRESS);
        publish_unrestricted_limits<LBR>(lr_account, calling_cap);
        publish_unrestricted_limits<Coin1>(lr_account, calling_cap);
        publish_unrestricted_limits<Coin2>(lr_account, calling_cap);
    }

    /// Determines if depositing `amount` of `CoinType` coins into the
    /// account at `addr` is amenable with their account limits.
    /// Returns false if this deposit violates the account limits. Effectful.
    public fun update_deposit_limits<CoinType>(
        amount: u64,
        addr: address,
        _cap: &CallingCapability,
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
        _cap: &CallingCapability,
    ): bool acquires LimitsDefinition, Window {
        can_withdraw<CoinType>(
            amount,
            borrow_global_mut<Window<CoinType>>(addr),
        )
    }

    /// All accounts that could be subject to account limits will have a
    /// `Window` for each currency they can hold published at the top level.
    /// Root accounts for multi-account entities will hold this resource at
    /// their root/parent account.
    public fun publish_window<CoinType>(
        to_limit: &signer,
        _: &CallingCapability,
        limit_address: address,
    ) {
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

    /// Publishes a `LimitsDefinition` resource under `account`. The caller must have permission
    /// to publish this, represented by the `CallingCapability`.
    public fun publish_limits_definition<CoinType>(
        account: &signer,
        _: &CallingCapability,
        max_inflow: u64,
        max_outflow: u64,
        max_holding: u64,
        time_period: u64
    ) {
        move_to(
            account,
            LimitsDefinition<CoinType> {
                max_inflow,
                max_outflow,
                max_holding,
                time_period,
            }
        )
    }

    /// Unrestricted accounts are represented by setting all fields in the
    /// limits definition to u64 max.
    public fun publish_unrestricted_limits<CoinType>(account: &signer, cap: &CallingCapability) {
        publish_limits_definition<CoinType>(account, cap, U64_MAX, U64_MAX, U64_MAX, ONE_DAY)
    }

    /// Updates the `LimitsDefinition<CoinType>` resource at `limit_address`.
    /// If any of the field arguments is `0` the corresponding field is not updated.
    public fun update_limits_definition<CoinType>(
        tc_account: &signer,
        limit_address: address,
        new_max_inflow: u64,
        new_max_outflow: u64,
        new_max_holding_balance: u64,
    ) acquires LimitsDefinition {
        assert(has_treasury_compliance_role(tc_account), NOT_TREASURY_COMPLIANCE);
        // As we don't have Optionals for txn scripts, in update_unhosted_wallet_limits.move
        // we use 0 value to represent a None (ie no update to that variable)
        let limits_def = borrow_global_mut<LimitsDefinition<CoinType>>(limit_address);
        if (new_max_inflow > 0) { limits_def.max_inflow = new_max_inflow };
        if (new_max_outflow > 0) { limits_def.max_outflow = new_max_outflow };
        if (new_max_holding_balance > 0) { limits_def.max_holding = new_max_holding_balance };
    }

    /// Since we don't track balances of accounts before they are limited, once
    /// they do become limited the approximate balance in `CointType` held by
    /// the entity across all of its accounts will need to be set by the association.
    public fun set_current_holdings<CoinType>(
        tc_account: &signer,
        window_address: address,
        aggregate_balance: u64,
    ) acquires Window {
        assert(has_treasury_compliance_role(tc_account), NOT_TREASURY_COMPLIANCE);
        borrow_global_mut<Window<CoinType>>(window_address).tracked_balance = aggregate_balance;
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

    public fun limits_definition_address<CoinType>(addr: address): address acquires Window {
        borrow_global<Window<CoinType>>(addr).limit_address
    }

    public fun is_unlimited_account<CoinType>(addr: address): bool acquires LimitsDefinition {
        is_unrestricted(borrow_global<LimitsDefinition<CoinType>>(addr))
    }

    public fun has_limits_published<CoinType>(addr: address): bool {
        exists<LimitsDefinition<CoinType>>(addr)
    }

    fun current_time(): u64 {
        if (LibraTimestamp::is_not_initialized()) 0 else LibraTimestamp::now_microseconds()
    }
}

}
