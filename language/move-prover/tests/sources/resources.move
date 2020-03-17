// dep: ../stdlib/modules/transaction.move

module TestResources {
    use 0x0::Transaction;

    resource struct R {
        x: u64
    }

    fun create_resource() {
        move_to_sender<R>(R{x:1});
    }
    spec fun create_resource {
        aborts_if exists<R>(sender());
        ensures exists<R>(sender());
    }

    fun create_resource_bad() {
        if(exists<R>(Transaction::sender())) {
            abort 1
        };
    }
    spec fun create_resource_bad {
        aborts_if exists<R>(sender());
        ensures exists<R>(sender());
    }
}
