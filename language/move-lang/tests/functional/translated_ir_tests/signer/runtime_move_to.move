module M {
    resource struct R1 { f: bool }
    resource struct R2<T> { f: T }

    public fun store(sender: &signer, f: bool) {
        move_to(sender, R1 { f })
    }

    public fun store_gen<T>(sender: &signer, f: T) {
        move_to(sender, R2<T> { f })
    }

    // TODO migrate to use signer
    public fun read(): bool acquires R1 {
        borrow_global<R1>(0x0::Transaction::sender()).f
    }

    // TODO migrate to use signer
    public fun read_gen<T: copyable>(): T acquires R2 {
        *&borrow_global<R2<T>>(0x0::Transaction::sender()).f
    }
}


//! new-transaction
script {
use {{default}}::M;
fun main(sender: &signer) {
    M::store(sender, false);
    0x0::Transaction::assert(M::read() == false, 42);

    M::store_gen<bool>(sender, true);
    0x0::Transaction::assert(M::read_gen<bool>() == true, 42);

    M::store_gen<u64>(sender, 112);
    0x0::Transaction::assert(M::read_gen<u64>() == 112, 42)
}
}

//! account: alice, 90000
//! new-transaction
//! sender: alice
script {
use {{default}}::M;
fun main(sender: &signer) {
    M::store(sender, false);
    M::store_gen<bool>(sender, true);
    M::store_gen<u64>(sender, 112)
}
}

//! new-transaction
//! sender: alice
script {
use {{default}}::M;
fun main(_sender: &signer) {
    0x0::Transaction::assert(M::read() == false, 42);
    0x0::Transaction::assert(M::read_gen<bool>() == true, 42);
    0x0::Transaction::assert(M::read_gen<u64>() == 112, 42)
}
}
