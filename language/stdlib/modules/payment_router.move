// Errors:
// 0 -> ADDRESS_NOT_ALLOWED
// 1 -> CURRENCY_NOT_HELD
// 2 -> NON_EXCLUSIVE_WITHDRAWALS_NOT_PERMITTED
// 3 -> ADDRESS_IS_NOT_ROUTED
address 0x0 {
module PaymentRouter {
    use 0x0::LibraAccount;
    use 0x0::Transaction;
    use 0x0::Libra;
    use 0x0::Vector;

    // A resource that specifies the addresses that are allowed to be added
    // to this multi-currency holding.
    resource struct PaymentRouterInfo {
        // Whether the router account has exclusive withdrawal privileges to
        // its routed accounts. If false withdrawal privileges are shared
        // between router and child.
        exclusive_withdrawals_only: bool,
        allowed_addresses: vector<address>,
    }

    // Withdrawal capabilities for the added accounts, partitioned by their
    // currency type.
    resource struct AccountInfo<Token> {
        child_accounts: vector<address>,
    }

    // A delegated withdrawal capability that is published under the child
    // account and that allows the routing account to withdraw from it (and
    // the sender, depending on flags set in `PaymentRouterInfo`).
    resource struct RoutedAccount<Token> {
        router_account_addr: address,
        withdrawal_cap: LibraAccount::WithdrawalCapability,
    }

    // Initialize the sending account with `exclusive_withdrawals_only`
    // either turned on (true) or off (false).
    public fun initialize(exclusive_withdrawals_only: bool) {
        move_to_sender(PaymentRouterInfo {
            exclusive_withdrawals_only,
            allowed_addresses: Vector::empty(),
        })
    }

    // Set whether exclusive access is held to the routed accounts
    public fun set_exclusive_withdrawals(exclusive_withdrawals_only: bool)
    acquires PaymentRouterInfo {
        borrow_global_mut<PaymentRouterInfo>(Transaction::sender())
            .exclusive_withdrawals_only = exclusive_withdrawals_only
    }

    // Return whether the router at `router_account_addr` only allows
    // exclusive withdrawals
    public fun exclusive_withdrawals_only(router_account_addr: address): bool
    acquires PaymentRouterInfo {
        borrow_global_mut<PaymentRouterInfo>(router_account_addr).exclusive_withdrawals_only
    }

    // Allow the account at `addr` to delegate its withdrawal_capability to us.
    public fun allow_account_address(addr: address)
    acquires PaymentRouterInfo {
        let sender = Transaction::sender();
        let router_info = borrow_global_mut<PaymentRouterInfo>(sender);
        if (!Vector::contains(&router_info.allowed_addresses, &addr))
            Vector::push_back(&mut router_info.allowed_addresses, addr);
    }

    // Allow routing of currencies of `Token` type.
    public fun allow_currency<Token>() {
        move_to_sender(AccountInfo<Token>{ child_accounts: Vector::empty() })
    }

    // Add the sending account of currency type `Token` to the router at
    // `router_account_addr`.  The sending address must be in the
    // `allowed_addresses` held under the router account.
    public fun add_account_to<Token>(router_account_addr: address)
    acquires PaymentRouterInfo, AccountInfo {
        let sender = Transaction::sender();
        let router_info = borrow_global_mut<PaymentRouterInfo>(router_account_addr);
        let (has, index) = Vector::index_of(&router_info.allowed_addresses, &sender);
        Transaction::assert(has, 0);
        let account_info = borrow_global_mut<AccountInfo<Token>>(router_account_addr);
        Vector::swap_remove(&mut router_info.allowed_addresses, index);
        Vector::push_back(
            &mut account_info.child_accounts,
            Transaction::sender(),
        );
        move_to_sender(RoutedAccount<Token> {
            router_account_addr,
            withdrawal_cap: LibraAccount::extract_sender_withdrawal_capability()
        })
    }

    // Routes deposits to a sub-account that holds currencies of type `Token`.
    public fun deposit<Token>(router_account_addr: address, coin: Libra::T<Token>)
    acquires AccountInfo {
        let addrs = &borrow_global<AccountInfo<Token>>(router_account_addr).child_accounts;
        Transaction::assert(!Vector::is_empty(addrs), 1);
        // TODO: policy around how to rotate through different accounts
        let index = 0;
        LibraAccount::deposit(
            *Vector::borrow(addrs, index),
            coin
        );
    }

    // Withdraws `amount` of `Token` currency from the sending account
    // using the delegated withdrawal capability in the PaymentRouterInfo
    // at `router_account_addr`.
    public fun withdraw_through<Token>(amount: u64): Libra::T<Token>
    acquires PaymentRouterInfo, RoutedAccount {
        let routed_info = borrow_global<RoutedAccount<Token>>(Transaction::sender());
        let router_info = borrow_global<PaymentRouterInfo>(*&routed_info.router_account_addr);
        Transaction::assert(!router_info.exclusive_withdrawals_only, 2);
        LibraAccount::withdraw_with_capability(
            &routed_info.withdrawal_cap,
            amount
        )
    }

    // Routes withdrawal requests from the sending account to a sub-account
    // that holds currencies of type `Token`.
    public fun withdraw<Token>(amount: u64): Libra::T<Token>
    acquires AccountInfo, RoutedAccount {
        let addrs = &borrow_global<AccountInfo<Token>>(Transaction::sender()).child_accounts;
        Transaction::assert(!Vector::is_empty(addrs), 1);
        // TODO: policy around how to rotate through different accounts
        let index = 0;
        let addr = Vector::borrow(addrs, index);
        LibraAccount::withdraw_with_capability(
            &borrow_global<RoutedAccount<Token>>(*addr).withdrawal_cap,
            amount
        )
    }

    // Return the account addresses in this multi-currency holding that can
    // hold currency of `Token` type. The set of addresses returned can both be
    // empty, or non-empty.
    public fun addresses_for_currency<Token>(router_account_addr: address): vector<address>
    acquires AccountInfo {
        *&borrow_global<AccountInfo<Token>>(router_account_addr).child_accounts
    }

    // Return whether the account at `addr` is a routed address or not.
    public fun is_routed<Token>(addr: address): bool {
        exists<RoutedAccount<Token>>(addr)
    }

    // Return the router address for the account.
    public fun router_address<Token>(addr: address): address
    acquires RoutedAccount {
        Transaction::assert(is_routed<Token>(addr), 3);
        *&borrow_global<RoutedAccount<Token>>(addr).router_account_addr
    }
}

}
