address 0x1 {

module LBR {
    use 0x1::Transaction;
    use 0x1::Libra;

    struct T { }

    public fun initialize() {
        assert(Transaction::sender() == 0xA550C18, 0);
        Libra::register<T>();
    }
    spec fun initialize {
        aborts_if sender() != 0xA550C18;
        aborts_if exists<Libra::MintCapability<T>>(sender());
        aborts_if exists<Libra::Info<T>>(sender());
        ensures exists<Libra::MintCapability<T>>(sender());
        ensures exists<Libra::Info<T>>(sender());
        ensures global<Libra::Info<T>>(sender()).total_value == 0;
        ensures global<Libra::Info<T>>(sender()).preburn_value == 0;
    }
}
}
