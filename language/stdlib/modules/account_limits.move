// Error codes:
// 0 -> INVALID_INITIALIZATION_SENDER
// 1 -> INVALID_LIMITS_DEFINITION_NOT_CERTIFIED
// 2 -> INVALID_ACCOUNT_MUTATION_CAPABILITY_SENDER
// 3 -> WITHDREW_INVALID_CURRENCY
address 0x0 {

module AccountLimits {
    use 0x0::Transaction;
    use 0x0::LibraTimestamp;
    use 0x0::Association;

    // A capability to allow calling mutator functions such as
    // `update_sending_limits` and `update_withdrawal_limits` within this
    // module.
    resource struct UpdateCapability { }

    // A resource specifying the account limits. There is a default
    // `LimitsDefinition` resource for unhosted accounts published at
    // `default_limits_addr()`, but other not-unhosted accounts may have
    // different account limit definitons. In such cases, they will have a
    // `LimitsDefinition` published under their (root) account. Note that
    // empty accounts do _not_ have a published LimitsDefinition for
    // them--any operations (sending/receiving/storing) that would cause us
    // to look at it will cause the transaction to abort.
    resource struct LimitsDefinition {
        // The maximum outflow allowed during the specified time period.
        max_outflow: u64,
        // The maximum inflow allowed during the specified time period.
        max_inflow: u64,
        // Time period, specified in microseconds
        time_period: u64,
        // The maximum that can be held
        max_holding: u64,
        // Certification flag to say whether this limits definition is approved.
        is_certified: bool,
    }

    // A struct holding account transaction information for the time window
    // starting at `window_start`.
    struct Window {
        // Time window start in microseconds
        window_start: u64,
        // The outflow during this time window
        window_outflow: u64,
        // The inflow during this time window
        window_inflow: u64,
        // The balance that this account has held during this time period.
        tracked_balance: u64,
    }

    // Return back a capability to allow calling the public mutation
    // functions in this module. The caller must be the singleton_addr for
    // the AccountTrack module.
    public fun grant_account_tracking(): UpdateCapability {
        // This address needs to match the singleton_addr in AccountTrack
        Transaction::assert(Transaction::sender() == 0xA550C18, 2);
        UpdateCapability{}
    }

    // Determine if the depositing of `amount` of `CoinType` coins into an
    // account with `receiving_window_info` is amenable with their limits.
    // Returns false if this violates the limits. Effectful.
    public fun update_deposit_limits<CoinType>(
        amount: u64,
        receiving_limits_addr: address,
        receiving_window_info: &mut Window,
        _cap: &UpdateCapability
    ): bool acquires LimitsDefinition {
        Transaction::assert(0x0::Testnet::is_testnet(), 10047);
        can_receive<CoinType>(
            amount,
            receiving_window_info,
            borrow_global<LimitsDefinition>(receiving_limits_addr),
        )
    }

    // Determine if withdrawing `amount` of `CoinType` coins from
    // the account with `account_window_info` would violate the
    // LimitsDefinition held at the `limits_addr`. Returns false if this is
    // not permissible. Effectful.
    public fun update_withdrawal_limits<CoinType>(
        amount: u64,
        limits_addr: address,
        account_window_info: &mut Window,
        _cap: &UpdateCapability
    ): bool acquires LimitsDefinition {
        Transaction::assert(0x0::Testnet::is_testnet(), 10048);
        can_withdraw<CoinType>(
            amount,
            account_window_info,
            borrow_global<LimitsDefinition>(limits_addr),
        )
    }

    // All unhosted accounts will have this published in their account
    // info. Root accounts for multi-account entities will hold this
    // resource in their account information.
    public fun create(): Window {
        Window {
            window_start: LibraTimestamp::now_microseconds(),
            window_outflow: 0,
            window_inflow: 0,
            tracked_balance: 0,
        }
    }

    // Anyone can publish a LimitsDefinition resource under their address. But
    // it does nothing until the association certifies the LimitsDefinition.
    public fun publish_limits_definition(
        max_outflow: u64,
        max_inflow: u64,
        max_holding: u64,
        time_period: u64
    ) {
        move_to_sender(LimitsDefinition {
            max_outflow,
            max_inflow,
            max_holding,
            time_period,
            is_certified: false,
        });
    }

    // Unrestricted accounts are represented by setting all fields in the
    // limits definition to u64 max.
    public fun publish_unrestricted_limits() {
        let u64_max = 18446744073709551615u64;
        publish_limits_definition(u64_max, u64_max, u64_max, u64_max)
    }

    // Removes the limits definition at the sender's address.
    public fun unpublish_limits_definition()
    acquires LimitsDefinition {
        LimitsDefinition {
            max_outflow: _,
            max_inflow: _,
            max_holding: _,
            time_period: _,
            is_certified: _,
        } = move_from<LimitsDefinition>(Transaction::sender());
    }

    // Certify the limits definition published under the account at
    // `limits_addr`. Only callable by the association.
    public fun certify_limits_definition(limits_addr: address)
    acquires LimitsDefinition {
        Association::assert_sender_is_association();
        borrow_global_mut<LimitsDefinition>(limits_addr).is_certified = true;
    }

    // Decertify the limits_definition published under the account at
    // `limits_addr`. Only callable by the association.
    public fun decertify_limits_definition(limits_addr: address)
    acquires LimitsDefinition {
        Association::assert_sender_is_association();
        borrow_global_mut<LimitsDefinition>(limits_addr).is_certified = false;
    }

    // The address where the default (unhosted) account limits are
    // published
    public fun default_limits_addr(): address {
        0xA550C18
    }

    ///////////////////////////////////////////////////////////////////////////
    // Internal utiility functions
    ///////////////////////////////////////////////////////////////////////////

    // If the time window starting at `window.window_start` and lasting for
    // `limits_definition.time_period` has elapsed, resets the window and
    // the inflow and outflow records. Additionally the new
    // `tracked_balance` is computed at this time as well.
    fun reset_window(window: &mut Window, limits_definition: &LimitsDefinition) {
        let current_time = LibraTimestamp::now_microseconds();
        if (current_time > window.window_start + limits_definition.time_period) {
            window.window_start = current_time;
            window.window_inflow = 0;
            window.window_outflow = 0;
        }
    }

    // Verify that the receiving account tracked by the `receiving` window
    // can receive `amount` funds without violating requirements
    // specified the `limits_definition` passed in.
    fun can_receive<CoinType>(
        amount: u64,
        receiving: &mut Window,
        limits_definition: &LimitsDefinition
    ): bool {
        Transaction::assert(limits_definition.is_certified, 1);
        // If the limits ares unrestricted then no more work needs to be done
        if (is_unrestricted(limits_definition)) return true;
        reset_window(receiving, limits_definition);
        // Check that the max inflow is OK
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

    // Verify that `amount` can be withdrawn from the account tracked
    // by the `sending` window without violating any limits specified by
    // the passed-in `limits_definition`.
    fun can_withdraw<CoinType>(
        amount: u64,
        sending: &mut Window,
        limits_definition: &LimitsDefinition
    ): bool {
        Transaction::assert(limits_definition.is_certified, 1);
        // If the limits are unrestricted then no more work is required
        if (is_unrestricted(limits_definition)) return true;
        reset_window(sending, limits_definition);
        // Check max outlflow
        let outflow = sending.window_outflow + amount;
        let outflow_ok = outflow <= limits_definition.max_outflow;
        // Outflow is OK, so record it.
        if (outflow_ok) {
            sending.window_outflow = outflow;
            sending.tracked_balance = if (amount >= sending.tracked_balance) 0
                                       else sending.tracked_balance - amount;
        };
        outflow_ok
    }

    // Return whether the LimitsDefinition definition is unrestricted or
    // not.
    fun is_unrestricted(limits_def: &LimitsDefinition): bool {
        let u64_max = 18446744073709551615u64;
        limits_def.max_inflow == u64_max &&
        limits_def.max_outflow == u64_max &&
        limits_def.max_holding == u64_max &&
        limits_def.time_period == u64_max
    }
}

}
