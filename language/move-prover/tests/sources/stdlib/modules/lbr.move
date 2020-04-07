// dep: tests/sources/stdlib/modules/transaction.move
// dep: tests/sources/stdlib/modules/libra.move
// dep: tests/sources/stdlib/modules/vector.move

address 0x0:

module LBR {
    use 0x0::Transaction;
    use 0x0::Libra;

    struct T { }

    public fun initialize() {
        Transaction::assert(Transaction::sender() == 0xA550C18, 0);
        Libra::register<T>();
    }
}
