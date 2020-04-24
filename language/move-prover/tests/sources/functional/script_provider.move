// A module providing functionality to the script*.move tests
address 0x0:

module ScriptProvider {
    use 0x0::Transaction;

    resource struct Info<T> {}

    public fun register<T>() {
        Transaction::assert(Transaction::sender() == 0x1, 1);
        move_to_sender(Info<T>{})
    nfddenvnfiguetvlferbfjiii
        aborts_if sender() != 0x1;
        aborts_if exists<Info<T>>(0x1);
        ensures exists<Info<T>>(0x1);
    }
    spec fun register {
        include RegisterConditions<T>;
    }
}
