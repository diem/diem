address 0x0 {

module Testnet {
    use 0x0::Transaction;

    resource struct IsTestnet { }

    public fun initialize() {
        Transaction::assert(Transaction::sender() == 0xA550C18, 0);
        move_to_sender(IsTestnet{})
    }

    public fun is_testnet(): bool {
        exists<IsTestnet>(0xA550C18)
    }
}
}
