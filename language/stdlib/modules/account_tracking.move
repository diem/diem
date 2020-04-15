///////////////////////////////////////////////////////////////////////////
// Every account on chain has a default LimitsDefinition this is stored
// under a specified address. However, certain
// accounts (e.g. VASPs) may have specific limits definitions. In order to
// set these specific account limits, the account publishes a proposed
// LimitsDefinition resource under their root authority account (in the
// case of a VASP, under the root account). The association then certifies
// this LimitsDefinition resource. After that, it will be used for tracking
// account inflow/outflow/holding.
//
// With this information, when we verify that an account A can send or
// receive money there are two cases to consider:
// 1. This account is a "normal" account: in this case the
//    account limits window is held under A and the default limits definition
//    is used.
// 2. The account A is part of a multi-account on-chain entity (e.g. VASP).
//    In this case we trace back to the root authority for the account and
//    use the limit definition under the root account, and the window under
//    the root account.
///////////////////////////////////////////////////////////////////////////

// Error codes:
// 3000 -> INVALID_INITIALIZATION_SENDER
// 3001 -> UNKNOWN_ACCOUNT_TYPE
// 3002 -> ATTEMPTED_OPERATION_ON_EMPTY_ACCOUNT_TYPE

address 0x0:

module AccountTrack {
    use 0x0::Transaction;
    use 0x0::VASP;
    use 0x0::Unhosted;
    use 0x0::Empty;
    use 0x0::AccountType;
    use 0x0::AccountLimits;

    // An `AccountLimitsCapability` holds the capabilities needed in order
    // to call other privileged functions in other modules.
    resource struct AccountLimitsCapability {
        // Allows functions in this module to mutate
        // `AccountLimits::Window` values.
        account_limits_cap: AccountLimits::UpdateCapability,
        // Allows functions in this module to mutate the interior state of
        // account types
        update_cap: AccountType::UpdateCapability,
    }

    // An operations capability that restricts callers of this module since
    // the operations can mutate account states.
    resource struct CallingCapability { }

    // Publishes a singleton `AccountLimitsCapability` under the account at
    // `singleton_addr()`.
    public fun initialize() {
        Transaction::assert(Transaction::sender() == singleton_addr(), 3000);
        let account_limits_cap = AccountLimits::grant_account_tracking();
        let update_cap = AccountType::grant_account_tracking();
        move_to_sender(AccountLimitsCapability {
            account_limits_cap, update_cap
        });
    }

    // Grant a capability to call this module. This does not necessarily
    // need to be a unique capability.
    public fun grant_calling_capability(): CallingCapability {
        Transaction::assert(Transaction::sender() == 0xA550C18, 3000);
        CallingCapability{}
    }

    // Determines if sending `amount` from `sending_addr` to
    // `receiving_addr` is valid w.r.t. to their respective
    // LimitsDefinitions. Updates them if this is a valid transfer and
    // returns `true`.  Otherwise it returns `false` and state is not
    // mutated.  One key property that is encoded here: transfers between
    // accounts with the same root authority (i.e. intra-authority
    // transfers) do not count towards sending limits or other such updates
    // (e.g. in the case of intra-VASP transfers).
    public fun update_deposit_limits<CoinType>(
        amount: u64,
        sending_addr: address,
        receiving_addr: address,
        _cap: &CallingCapability,
    ): bool acquires AccountLimitsCapability {
        let (sender_limit_addr, _) = tracking_info(sending_addr);
        let (receiving_limit_addr, receiving_info) = tracking_info(receiving_addr);
        // We have the same specification, and these are _not_ unhosted
        // accounts. They therefore have the same root authority. Therefore
        // this is an intra-entity transfer, and we don't count this
        // towards anything.
        if (sender_limit_addr == receiving_limit_addr &&
            receiving_limit_addr != singleton_addr()) {
            return true
        };
        let can_send = AccountLimits::update_deposit_limits<CoinType>(
            amount,
            receiving_limit_addr,
            &mut receiving_info,
            &borrow_global<AccountLimitsCapability>(singleton_addr()).account_limits_cap
        );
        update_info(receiving_addr, receiving_info);
        can_send
    }

    // Determines if `amount` can be withdrawn from the account at
    // `addr`. Updates state and returns `true` if it can be updated.
    // Otherwise returns `false` and state is not updated.
    public fun update_withdrawal_limits<CoinType>(
        amount: u64,
        addr: address,
        _cap: &CallingCapability,
    ): bool acquires AccountLimitsCapability {
        let (limits_addr, account_metadata) = tracking_info(addr);
        let can_withdraw = AccountLimits::update_withdrawal_limits<CoinType>(
            amount,
            limits_addr,
            &mut account_metadata,
            &borrow_global<AccountLimitsCapability>(singleton_addr()).account_limits_cap
        );
        update_info(addr, account_metadata);
        can_withdraw
    }

    ///////////////////////////////////////////////////////////////////////////
    // Private/utility functions
    ///////////////////////////////////////////////////////////////////////////

    // Updates the `AccountLimits::Window` at `addr`. This traces back to
    // the root authority and updates the information there in case that
    // this is a child account, otherwise it updates the
    // `AccountLimits::Window` value at `addr` in the case that it's an
    // unhosted account.
    fun update_info(addr: address, info: AccountLimits::Window)
    acquires AccountLimitsCapability {
        if (AccountType::is_a<Unhosted::T>(addr)) {
            let unhosted_info = Unhosted::update_account_limits(info);
            AccountType::update_with_capability<Unhosted::T>(
                addr,
                unhosted_info,
                &borrow_global<AccountLimitsCapability>(singleton_addr()).update_cap
            )
        } else if (VASP::is_vasp(addr)) {
            let root_addr = VASP::root_vasp_address(addr);
            let root_vasp_info = VASP::update_account_limits(root_addr, info);
            AccountType::update_with_capability<VASP::RootVASP>(
                root_addr,
                root_vasp_info,
                &borrow_global<AccountLimitsCapability>(singleton_addr()).update_cap
            )
        } else if (AccountType::is_a<Empty::T>(addr)) {
            abort 3002
        } else { // Can add more logic here as we add more account types
            abort 3001
        }
    }

    // Returns the appropriate account limits window along with the address
    // for the limits definition that should be used for `addr`.
    fun tracking_info(addr: address): (address, AccountLimits::Window) {
        if (AccountType::is_a<Unhosted::T>(addr)) {
            (
                Unhosted::limits_addr(),
                Unhosted::account_limits(AccountType::account_metadata<Unhosted::T>(addr))
            )
        } else if (VASP::is_vasp(addr)) {
            let root_addr = VASP::root_vasp_address(addr);
            (root_addr, VASP::account_limits(root_addr))
        } else if (AccountType::is_a<Empty::T>(addr)) {
            abort 3002
        } else { // Can add more logic here as we add more account types
            abort 3001
        }
    }

    fun singleton_addr(): address {
        0xA550C18
    }
}
