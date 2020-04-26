address 0x0:
module TransactionFeeAccounts {
    use 0x0::Association;
    use 0x0::Transaction;

    resource struct FeeAddress<T> {
        addr: address,
    }

    // Association capability for registering a transaction fee address
    struct TransactionFeeAccountRegistration {}

    public fun initialize() {
        Transaction::assert(Transaction::sender() == singleton_addr(), 0);
        Association::apply_for_privilege<TransactionFeeAccountRegistration>();
    }

    public fun add_transaction_fee_account<T>(addr: address) {
        let sender = Transaction::sender();
        Transaction::assert(
            Association::has_privilege<TransactionFeeAccountRegistration>(sender),
            0
        );
        Transaction::assert(Transaction::sender() == singleton_addr(), 1);
        move_to_sender(FeeAddress<T>{ addr })
    }

    public fun transaction_fee_address<T>(): address
    acquires FeeAddress {
        borrow_global<FeeAddress<T>>(singleton_addr()).addr
    }

    fun singleton_addr(): address {
        0xA550C18
    }
}
