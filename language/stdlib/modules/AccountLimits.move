// Error codes:
// 0 -> INVALID_INITIALIZATION_SENDER
// 1 -> INVALID_LIMITS_DEFINITION_NOT_CERTIFIED
// 2 -> INVALID_ACCOUNT_MUTATION_CAPABILITY_SENDER
// 3 -> WITHDREW_INVALID_CURRENCY
address 0x1 {

module AccountLimits {
    use 0x1::CoreAddresses;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
    use 0x1::Roles::{Capability, TreasuryComplianceRole, AssociationRootRole};

    // An operations capability that restricts callers of this module since
    // the operations can mutate account states.
    resource struct CallingCapability { }

    // A resource specifying the account limits. There is a default
    // `LimitsDefinition` resource for unhosted accounts published at
    // `CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()`, but other not-unhosted accounts may have
    // different account limit definitons. In such cases, they will have a
    // `LimitsDefinition` published under their (root) account. Note that
    // empty accounts do _not_ have a published LimitsDefinition for
    // them--any operations (sending/receiving/storing) that would cause us
    // to look at it will cause the transaction to abort.
    // All monentary amounts specified in microLBR
    resource struct LimitsDefinition {
        // The maximum inflow + outflow allowed during the specified time period.
        max_total_flow: u64,
        // Time period, specified in microseconds
        time_period: u64,
        // The maximum that can be held
        max_holding: u64,
        // Certification flag to say whether this limits definition is approved.
        is_certified: bool,
    }

    // A struct holding account transaction information for the time window
    // starting at `window_start`.
    resource struct Window {
        // Time window start in microseconds
        window_start: u64,
        // The outflow + inflow during this time window
        window_total_flow: u64,
        // The balance that this account has held during this time period.
        tracked_balance: u64,
        // address storing the LimitsDefinition resource that governs this window
        limits_definition: address,
    }

    // Grant a capability to call this module. This does not necessarily
    // need to be a unique capability.
    public fun grant_calling_capability(_: &Capability<AssociationRootRole>): CallingCapability {
        CallingCapability{}
    }

    // Determine if the depositing of `amount` of `CoinType` coins into an
    // account with `receiving_window_info` is amenable with their limits.
    // Returns false if this violates the limits. Effectful.
    public fun update_deposit_limits<CoinType>(
        amount: u64,
        addr: address,
        _cap: &CallingCapability,
    ): bool acquires LimitsDefinition, Window {
        assert(0x1::Testnet::is_testnet(), 10047);
        can_receive<CoinType>(
            amount,
            borrow_global_mut<Window>(addr),
        )
    }

    // Determine if withdrawing `amount` of `CoinType` coins from
    // the account with `account_window_info` would violate the
    // LimitsDefinition held at the `limits_addr`. Returns false if this is
    // not permissible. Effectful.
    public fun update_withdrawal_limits<CoinType>(
        amount: u64,
        addr: address,
        _cap: &CallingCapability,
    ): bool acquires LimitsDefinition, Window {
        assert(0x1::Testnet::is_testnet(), 10048);
        can_withdraw<CoinType>(
            amount,
            borrow_global_mut<Window>(addr),
        )
    }

    // TODO: take limits_definition as input
    // All unhosted accounts will have this published at the top level. Root accounts for
    // multi-account entities will hold this resource in their account information.
    public fun publish(to_limit: &signer) {
        move_to(
            to_limit,
            Window {
                window_start: current_time(),
                window_total_flow: 0,
                tracked_balance: 0,
                limits_definition: CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()
            }
        )
    }

    // Anyone can publish a LimitsDefinition resource under their address. But
    // it does nothing until the association certifies the LimitsDefinition.
    public fun publish_limits_definition(
        account: &signer,
        max_total_flow: u64,
        max_holding: u64,
        time_period: u64
    ) {
        move_to(
            account,
            LimitsDefinition {
                max_total_flow,
                max_holding,
                time_period,
                is_certified: false,
            }
        )
    }

    // Unrestricted accounts are represented by setting all fields in the
    // limits definition to u64 max.
    public fun publish_unrestricted_limits(account: &signer) {
        let u64_max = 18446744073709551615u64;
        publish_limits_definition(account, u64_max, u64_max, u64_max)
    }

    // Removes the limits definition at the sender's address.
    public fun unpublish_limits_definition(account: &signer)
    acquires LimitsDefinition {
        LimitsDefinition {
            max_total_flow: _,
            max_holding: _,
            time_period: _,
            is_certified: _,
        } = move_from<LimitsDefinition>(Signer::address_of(account));
    }

    public fun update_limits_definition(
        _: &Capability<TreasuryComplianceRole>,
        new_max_total_flow: u64,
        new_max_holding_balance: u64,
    ) acquires LimitsDefinition {
        // As we don't have Optionals for txn scripts, in update_unhosted_wallet_limits.move
        // we use 0 value to represent a None (ie no update to that variable)
        if (new_max_total_flow != 0) {
            borrow_global_mut<LimitsDefinition>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()).max_total_flow = new_max_total_flow;
        };
        if (new_max_holding_balance != 0) {
            borrow_global_mut<LimitsDefinition>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()).max_holding = new_max_holding_balance;
        };
    }

    // Certify the limits definition published under the account at
    // `limits_addr`.
    public fun certify_limits_definition(_: &Capability<TreasuryComplianceRole>, limits_addr: address)
    acquires LimitsDefinition {
        borrow_global_mut<LimitsDefinition>(limits_addr).is_certified = true;
    }

    // Decertify the limits_definition published under the account at
    // `limits_addr`.
    public fun decertify_limits_definition(_: &Capability<TreasuryComplianceRole>, limits_addr: address)
    acquires LimitsDefinition {
        borrow_global_mut<LimitsDefinition>(limits_addr).is_certified = false;
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
            window.window_total_flow = 0;
        }
    }

    // Verify that the receiving account tracked by the `receiving` window
    // can receive `amount` funds without violating requirements
    // specified the `limits_definition` passed in.
    fun can_receive<CoinType>(
        amount: u64,
        receiving: &mut Window,
    ): bool acquires LimitsDefinition {
        let limits_definition = borrow_global_mut<LimitsDefinition>(receiving.limits_definition);
        // If the limits ares unrestricted then no more work needs to be done
        if (is_unrestricted(limits_definition)) return true;

        reset_window(receiving, limits_definition);
        // Check that the max total flow is OK
        let total_flow_ok = receiving.window_total_flow + amount <= limits_definition.max_total_flow;
        // Check that the holding after the deposit is OK
        let holding_ok = receiving.tracked_balance + amount <= limits_definition.max_holding;
        // The account with `receiving` window can receive the payment so record it.
        if (total_flow_ok && holding_ok) {
            receiving.window_total_flow = receiving.window_total_flow + amount;
            receiving.tracked_balance = receiving.tracked_balance + amount;
        };
        total_flow_ok && holding_ok
    }

    // Verify that `amount` can be withdrawn from the account tracked
    // by the `sending` window without violating any limits specified by
    // the passed-in `limits_definition`.
    fun can_withdraw<CoinType>(
        amount: u64,
        sending: &mut Window,
    ): bool acquires LimitsDefinition {
        let limits_definition = borrow_global_mut<LimitsDefinition>(sending.limits_definition);
        // If the limits are unrestricted then no more work is required
        if (is_unrestricted(limits_definition)) return true;

        reset_window(sending, limits_definition);
        // Check total flow OK
        let total_flow_ok = sending.window_total_flow + amount <= limits_definition.max_total_flow;
        // Flow is OK, so record it.
        if (total_flow_ok) {
            sending.window_total_flow = sending.window_total_flow + amount;
            sending.tracked_balance = if (amount >= sending.tracked_balance) 0
                                       else sending.tracked_balance - amount;
        };
        total_flow_ok
    }

    // Return whether the LimitsDefinition definition is unrestricted or
    // not.
    fun is_unrestricted(limits_def: &LimitsDefinition): bool {
        let u64_max = 18446744073709551615u64;
        limits_def.max_total_flow == u64_max &&
        limits_def.max_holding == u64_max &&
        limits_def.time_period == u64_max
    }

    public fun limits_definition_address(addr: address): address acquires Window {
        borrow_global<Window>(addr).limits_definition
    }

    public fun is_unlimited_account(addr: address): bool acquires LimitsDefinition {
        is_unrestricted(borrow_global<LimitsDefinition>(addr))
    }

    fun current_time(): u64 {
        if (LibraTimestamp::is_genesis()) 0 else LibraTimestamp::now_microseconds()
    }
}

}
