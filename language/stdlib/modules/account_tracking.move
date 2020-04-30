///////////////////////////////////////////////////////////////////////////
// Every account on chain has a default LimitsDefinition this is stored
// under a specified address. However, certain accounts may have specific
// limits definitions. In order to set these specific account limits, the
// account publishes a proposed LimitsDefinition resource under their root
// authority account. The association then certifies this LimitsDefinition
// resource. After that, it will be used for tracking account
// inflow/outflow/holding.
//
// With this information, when we verify that an account A can send or
// receive money there are two cases to consider:
// 1. This account is a "normal" account: in this case the
//    account limits window is held under A and the default limits definition
//    is used.
// 2. The account A is part of a multi-account on-chain entity.
//    In this case we trace back to the root authority for the account and
//    use the limit definition under the root account, and the window under
//    the root account.
///////////////////////////////////////////////////////////////////////////

// Error codes:
// 3000 -> INVALID_INITIALIZATION_SENDER
// 3001 -> UNKNOWN_ACCOUNT_TYPE
// 3002 -> ATTEMPTED_OPERATION_ON_EMPTY_ACCOUNT_TYPE
// 3003 -> ATTEMPTED_OPERATION_ON_VASP_ACCOUNT_TYPE

address 0x0:

module AccountTrack {
    use 0x0::Transaction;
    use 0x0::VASP;
    use 0x0::Unhosted;
    use 0x0::Empty;
    use 0x0::AccountType;
    use 0x0::AccountLimits;
    use 0x0::Sender;

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
    public fun initialize(sender: &Sender::T) {
        Transaction::assert(Sender::address_(sender) == singleton_addr(), 3000);
        let account_limits_cap = AccountLimits::grant_account_tracking(sender);
        let update_cap = AccountType::grant_account_tracking(sender);
        Sender::move_to(sender);
        move_to_sender(AccountLimitsCapability {
            account_limits_cap, update_cap
        });
    }

    // Grant a capability to call this module. This does not necessarily
    // need to be a unique capability.
    public fun grant_calling_capability(sender: &Sender::T): CallingCapability {
        Transaction::assert(Sender::address_(sender) == 0xA550C18, 3000);
        CallingCapability{}
    }

    // Determines if depositing `amount` into `receiving_addr` is valid
    // w.r.t. to LimitsDefinition. Updates them if it is a valid deposit
    // and returns `true`.  Otherwise it returns `false` and state is not
    // mutated.
    public fun update_deposit_limits<CoinType>(
        amount: u64,
        receiving_addr: address,
        _cap: &CallingCapability,
    ): bool acquires AccountLimitsCapability {
        if (is_unlimited_account(receiving_addr)) return true;
        let (receiving_limit_addr, receiving_info) = tracking_info(receiving_addr);
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
        if (is_unlimited_account(addr)) return true;
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
            abort 3003
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
            abort 3003
        } else if (AccountType::is_a<Empty::T>(addr)) {
            abort 3002
        } else { // Can add more logic here as we add more account types
            abort 3001
        }
    }

    fun is_unlimited_account(addr: address): bool {
        VASP::is_vasp(addr)
    }

    fun singleton_addr(): address {
        0xA550C18
    }
}
